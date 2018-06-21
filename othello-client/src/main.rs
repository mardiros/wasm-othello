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

use wscommand::{Color, WsConnectingParam, WsJoinBoard, WsPlayBoard, WsRequest, WsResponse};

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

/// User connection status
pub enum ConnectionStatus {
    /// The useer is not connected
    Disconnected,
    /// The user trying to connect
    Connecting,
    /// The user is successfully connected
    Connected(String),
    /// The user try to connect but there is an issue
    ConnectionError(String),
}

pub struct AppModel {
    connected: ConnectionStatus,
    users_count: usize,
    nickname: String,

    board_id: String,
    opponent: Option<String>,
    opponent_move: Option<(usize, usize)>,
    color: Option<Color>,
    ws: Option<WebSocketTask>,

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
}

impl Component<Context> for AppModel {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: &mut Env<Context, Self>) -> Self {
        AppModel {
            connected: ConnectionStatus::Disconnected,
            nickname: "".to_string(),

            board_id: "".to_string(),
            opponent: None,
            opponent_move: None,
            color: None,

            users_count: 0,
            nickname_input: "".to_string(),
            ws: None,
        }
    }

    fn update(&mut self, msg: Self::Message, env: &mut Env<Context, Self>) -> ShouldRender {
        match msg {
            Msg::Connecting => {
                let callback = env.send_back(|Json(data)| Msg::WsReady(data));
                let notification = env.send_back(|status| match status {
                    WebSocketStatus::Opened => WsAction::SendUser.into(),
                    WebSocketStatus::Closed | WebSocketStatus::Error => WsAction::Lost.into(),
                });
                let ws_service: &mut WebSocketService = env.as_mut();
                let task = ws_service.connect("ws://[::1]:8080/ws/", callback, notification);
                self.ws = Some(task);

                self.connected = ConnectionStatus::Connecting;
                self.nickname = self.nickname_input.clone();
                info!("disconnect");
            }
            Msg::Disconnecting => {
                self.connected = ConnectionStatus::Disconnected;
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
                    self.ws.take().unwrap().cancel();
                }
                WsAction::Lost => {
                    self.ws = None;
                    self.connected = ConnectionStatus::ConnectionError(
                        "Service Temporarily Unavailable".to_string(),
                    );
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
                        let session_id = params.id.clone();
                        self.users_count = params.users_count;
                        self.connected = ConnectionStatus::Connected(session_id);
                    }
                    WsResponse::JoinedBoard(ref param) => {
                        self.board_id = param.id.clone();
                        self.color = Some(param.color.clone());
                        self.opponent = param.opponent.clone();
                    }
                    WsResponse::OpponentJoinedBoard(ref param) => {
                        if param.id == self.board_id {
                            self.opponent = Some(param.opponent.clone());
                        }
                    }
                    WsResponse::PlayedBoard(ref param) => match self.connected {
                        ConnectionStatus::Connected(ref session_id) => {
                            if param.board_id == self.board_id && session_id == &param.session_id {
                                self.opponent_move = Some(param.pos.clone());
                            }
                        }
                        _ => {}
                    },
                }
            }

            Msg::JoinBoard(()) => {
                info!("Join board");
                if let ConnectionStatus::Connected(ref session_id) = self.connected {
                    let payload = WsJoinBoard {
                        session_id: session_id.as_str(),
                    };
                    let command = WsRequest::JoinBoard(payload);
                    self.ws.as_mut().unwrap().send(Json(&command));
                }
            }
            Msg::BoardCellClicked((x, y)) => {
                info!("User play {} {}", x, y);
                if let ConnectionStatus::Connected(ref session_id) = self.connected {
                    let payload = WsPlayBoard {
                        session_id: session_id.as_str(),
                        board_id: self.board_id.as_str(),
                        pos: (x, y),
                    };
                    let command = WsRequest::PlayBoard(payload);
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
            ConnectionStatus::Connected(_) => {
                html!{
                    <span>
                        { format!("{} user(s) online", self.users_count) }
                    </span>
                    <button onclick=|_| Msg::Disconnecting.into(),>
                        { format!("Disconnect {}", self.nickname) }
                    </button>
                }
            }
            ConnectionStatus::Connecting => {
                html!{
                    <span>
                        { format!("Connecting {}...", self.nickname) }
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
            ConnectionStatus::Connected(_) => {
                html!{
                    <Board: nickname=&self.nickname,
                        opponent=&self.opponent,
                        opponent_move=&self.opponent_move,
                        color=&self.color,
                        onstart=Msg::JoinBoard,
                        onclick=Msg::BoardCellClicked, />
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
