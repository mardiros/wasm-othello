#[macro_use]
extern crate log;
extern crate web_logger;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate failure;

extern crate stdweb;
#[macro_use]
extern crate yew;

use failure::Error;

use stdweb::web::window;
use yew::prelude::*;
use yew::services::Task;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::format::Json;

mod context;
mod board;
mod model;
mod wscommand;

use context::Context;
use board::Board;

use wscommand::{Color, WsConnectingParam, WsJoinBoard, WsPlayBoard, WsGameOver, WsRequest, WsResponse};

pub enum WsAction {
    SendUser,
    Disconnect,
    Lost,
}

impl From<WsAction> for Msg {
    fn from(action: WsAction) -> Self {
        Msg::WsAction(action)
    }
}

#[derive(PartialEq)]
struct Session {
    session_id: String,
    board_id: String,
    users_count: usize,
    nickname: String,
    opponent: Option<String>,
    color: Option<Color>,
}

/// User connection status
#[derive(PartialEq)]
enum ConnectionStatus {
    /// The useer is not connected
    Disconnected,
    /// The user trying to connect
    Connecting(String),
    /// The user is successfully connected
    Connected(Session),
    /// The user try to connect but there is an issue
    ConnectionError(String),
}

/// Main Component of the application
pub struct AppModel {
    /// Contains session information in case user is connected
    connected: ConnectionStatus,

    /// Received by the websocket, send back to the board via a property
    opponent_move: Option<(usize, usize)>,

    /// the web socket to communicate with the server
    ws: Option<WebSocketTask>,

    // inputs
    /// store the value of the nickname input
    nickname_input: String,
}

pub enum Msg {
    Ignore,
    Connecting,
    Disconnecting,
    GotInput(String),
    WsAction(WsAction),
    WsReady(Result<WsResponse, Error>),

    JoinBoard(()),
    BoardCellClicked((usize, usize)),
    BoardGameOver((usize, usize))
}

impl Component<Context> for AppModel {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: &mut Env<Context, Self>) -> Self {
        AppModel {
            connected: ConnectionStatus::Disconnected,
            opponent_move: None,
            nickname_input: "".to_string(),
            ws: None,
        }
    }

    fn update(&mut self, msg: Self::Message, env: &mut Env<Context, Self>) -> ShouldRender {
        self.opponent_move = None; // always reset the move

        match msg {
            Msg::Connecting => {
                if self.nickname_input.len() == 0 {
                    return false;
                }
                let callback = env.send_back(|Json(data)| Msg::WsReady(data));
                let notification = env.send_back(|status| match status {
                    WebSocketStatus::Opened => WsAction::SendUser.into(),
                    WebSocketStatus::Closed | WebSocketStatus::Error => WsAction::Lost.into(),
                });
                let ws_service: &mut WebSocketService = env.as_mut();

                let endpoint = {
                    let location = window().location().unwrap();
                    format!("{}://{}:{}/ws/",
                        if location.protocol().unwrap() == "https:" { "wss" } else { "ws" },
                        location.hostname().unwrap(),
                        location.port().unwrap(),
                    )
                };
                let task = ws_service.connect(endpoint.as_str(), callback, notification);
                self.ws = Some(task);

                self.connected = ConnectionStatus::Connecting(self.nickname_input.clone());
                info!("connecting {}", self.nickname_input);
            }
            Msg::Disconnecting => {
                self.connected = ConnectionStatus::Disconnected;
                self.ws.take().unwrap().cancel();
                info!("disconnected");
            }

            Msg::GotInput(value) => {
                self.nickname_input = value;
            }

            Msg::WsAction(action) => match action {
                WsAction::SendUser => {
                    let payload = WsConnectingParam {
                        nickname: self.nickname_input.as_str(),
                    };
                    let command = WsRequest::ConnectingParam(payload);
                    self.ws.as_mut().unwrap().send(Json(&command));
                }
                WsAction::Disconnect => {
                    self.connected = ConnectionStatus::Disconnected;
                    self.ws.take().unwrap().cancel();
                }
                WsAction::Lost => {
                    self.ws = None;
                    if self.connected != ConnectionStatus::Disconnected {
                        error!("Connection Closed from the server");
                        self.connected = ConnectionStatus::ConnectionError(
                            "Service Temporarily Unavailable".to_string(),
                        );
                    }
                }
            },

            Msg::WsReady(response) => {
                if let Err(err) = response {
                    error!("{}", err);
                    return false;
                }
                let response = response.unwrap();
                info!("{:?}", response);
                match response {
                    WsResponse::ConnectedParam(ref params) => {
                        let new_status =
                            if let ConnectionStatus::Connecting(ref nickname) = self.connected {
                                let session_id = params.session_id.clone();
                                let users_count = params.users_count;
                                let nickname = nickname.clone();
                                ConnectionStatus::Connected(Session {
                                    session_id,
                                    users_count,
                                    nickname,
                                    board_id: "".to_string(),
                                    color: None,
                                    opponent: None,
                                })
                            } else {
                                ConnectionStatus::Disconnected
                            };
                        self.connected = new_status;
                    }
                    WsResponse::JoinedBoard(ref param) => {
                        if let ConnectionStatus::Connected(ref mut session) = self.connected {
                            if param.session_id == session.session_id {
                                session.board_id = param.board_id.clone();
                                session.color = Some(param.color.clone());
                                session.opponent = param.opponent.clone();
                            } else {
                                error!(
                                    "Session id does not match {} != {}",
                                    param.session_id, session.session_id
                                );
                            }
                        }
                    }
                    WsResponse::OpponentJoinedBoard(ref param) => {
                        if let ConnectionStatus::Connected(ref mut session) = self.connected {
                            if param.session_id == session.session_id {
                                if param.board_id == session.board_id {
                                    session.opponent = Some(param.opponent.clone());
                                } else {
                                    error!(
                                        "Board id does not match {} != {}",
                                        param.board_id, session.board_id
                                    );
                                }
                            } else {
                                error!(
                                    "Session id does not match {} != {}",
                                    param.session_id, session.session_id
                                );
                            }
                        }
                    }
                    WsResponse::PlayedBoard(ref param) => {
                        if let ConnectionStatus::Connected(ref mut session) = self.connected {
                            if param.board_id == session.board_id
                                && session.session_id == param.session_id
                            {
                                self.opponent_move = Some(param.pos.clone());
                            }
                        }
                    }
                    WsResponse::OpponentDisconnected(ref param) => {
                        if let ConnectionStatus::Connected(ref mut session) = self.connected {
                            if param.session_id == session.session_id {
                                session.board_id = "".to_string();
                                session.color = None;
                                session.opponent = None;
                            } else {
                                error!("OpponentDisconnect reveived with an invalid session_id");
                            }
                        }
                    }
                }
            }

            Msg::JoinBoard(()) => {
                info!("Join board");
                if let ConnectionStatus::Connected(ref session) = self.connected {
                    let payload = WsJoinBoard {
                        session_id: session.session_id.as_str(),
                    };
                    let command = WsRequest::JoinBoard(payload);
                    self.ws.as_mut().unwrap().send(Json(&command));
                }
            }
            Msg::BoardCellClicked((x, y)) => {
                info!("User play {} {}", x, y);
                if let ConnectionStatus::Connected(ref session) = self.connected {
                    let payload = WsPlayBoard {
                        session_id: session.session_id.as_str(),
                        board_id: session.board_id.as_str(),
                        pos: (x, y),
                    };
                    let command = WsRequest::PlayBoard(payload);
                    self.ws.as_mut().unwrap().send(Json(&command));
                }
            }

            Msg::BoardGameOver((black_score, white_score)) => {
                if let ConnectionStatus::Connected(ref session) = self.connected {
                    let payload = WsGameOver {
                        session_id: session.session_id.as_str(),
                        board_id: session.board_id.as_str(),
                        score: (black_score, white_score),
                    };
                    let command = WsRequest::GameOver(payload);
                    self.ws.as_mut().unwrap().send(Json(&command));
                }
            }

            Msg::Ignore => info!("Received an ignored message"),
        }
        true
    }
}

impl Renderable<Context, AppModel> for AppModel {
    fn view(&self) -> Html<Context, Self> {
        html! {
            <div id="main",>
                { self.view_connection_button() }
                <br/>
                { self.view_board() }
            </div>
        }
    }
}

impl AppModel {
    fn view_connection_button(&self) -> Html<Context, Self> {
        match self.connected {
            ConnectionStatus::Connected(ref session) => {
                html!{
                    <span>
                        { format!("{} user(s) online", session.users_count) }
                    </span>
                    <button onclick=|_| Msg::Disconnecting.into(),>
                        { format!("Disconnect {}", session.nickname) }
                    </button>
                }
            }
            ConnectionStatus::Connecting(ref nickname) => {
                html!{
                    <span>
                        { format!("Connecting {}...", nickname) }
                    </span>
                }
            }
            ConnectionStatus::ConnectionError(ref message) => {
                html!{
                    <>
                    <input class="edit",
                        type="text",
                        value=&self.nickname_input,
                        oninput=|e| Msg::GotInput(e.value),
                        />
                    <button onclick=|_| Msg::Connecting.into(),>{ "Connect" }</button>
                    <p class="error",>{ message }</p>
                    </>
                }
            }
            ConnectionStatus::Disconnected => {
                html!{
                    <>
                    <input class="edit",
                        type="text",
                        value=&self.nickname_input,
                        oninput=|e| Msg::GotInput(e.value),
                        />
                    <button onclick=|_| Msg::Connecting.into(),>{ "Connect" }</button>
                    </>
                }
            }
        }
    }

    fn view_board(&self) -> Html<Context, Self> {
        match self.connected {
            ConnectionStatus::Connected(ref session) => {
                html!{
                    <Board: nickname=&session.nickname,
                        opponent=&session.opponent,
                        color=&session.color,
                        opponent_move=&self.opponent_move,
                        onstart=Msg::JoinBoard,
                        onclick=Msg::BoardCellClicked, 
                        ongameover=Msg::BoardGameOver, />
                }
            }
            _ => {
                html!{
                    <>
                    </>
                }
            }
        }
    }
}

fn main() {
    web_logger::init();
    yew::initialize();
    let context = Context::new();
    let app: App<_, AppModel> = App::new(context);
    app.mount_to_body();
    yew::run_loop();
}
