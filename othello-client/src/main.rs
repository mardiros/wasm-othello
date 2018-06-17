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
use yew::services::console::ConsoleService;
use yew::services::Task;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::format::Json;

/// This type is an expected response from a websocket connection.
#[derive(Serialize, Debug)]
pub struct WsConnectingParam<'a> {
    nickname: &'a str,
}

#[derive(Serialize, Debug)]
pub enum WsRequest<'a> {
    ConnectingParam(WsConnectingParam<'a>)
}


#[derive(Deserialize, Debug)]
pub struct WsConnectedParam {
    // a session id
    // pub id: usize,
    pub users_count: usize,
}


#[derive(Deserialize, Debug)]
pub enum WsResponse {
    /// `connect` command parameters
    ConnectedParam(WsConnectedParam)
}

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
    Connecting(String),
    /// The user is successfully connected
    Connected(String),
    /// The user try to connect but there is an issue
    ConnectionError(String),
}

pub struct AppModel {
    connected: ConnectionStatus,
    input_value: String,
    ws: Option<WebSocketTask>,
    wsdata: Option<WsResponse>,
}

pub enum Msg {
    Ignore,
    Connecting,
    Disconnecting,
    GotInput(String),
    WsAction(WsAction),
    WsReady(Result<WsResponse, Error>),
}

impl Component<Context> for AppModel {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: &mut Env<Context, Self>) -> Self {
        AppModel {
            connected: ConnectionStatus::Disconnected,
            input_value: "".to_string(),
            ws: None,
            wsdata: None,
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

                self.connected = ConnectionStatus::Connecting(self.input_value.clone());
                info!("disconnect");
            }
            Msg::Disconnecting => {
                self.connected = ConnectionStatus::Disconnected;
                info!("disconnected");
            }
            Msg::GotInput(value) => {
                self.input_value = value;
            }

            Msg::WsAction(action) => match action {
                WsAction::SendUser => {
                    let payload = WsConnectingParam {
                        nickname: self.input_value.as_str(),
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
                    return false
                }
                let response = response.unwrap();
                self.wsdata = Some(response);
                self.connected = ConnectionStatus::Connected(self.input_value.clone());
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
            </div>
        }
    }
}

impl AppModel {
    fn view_connection_button(&self) -> Html<Context, Self> {
        match self.connected {
            ConnectionStatus::Connected(ref username) => {
                html!{
                    <button onclick=|_| Msg::Disconnecting.into(),>{ format!("Disconnect {}", username) }</button>
                }
            }
            ConnectionStatus::Connecting(ref username) => {
                html!{
                    <span>
                        { format!("Connecting {}...", username) }
                    </span>
                }
            }
            ConnectionStatus::ConnectionError(ref message) => {
                html!{
                    <>
                    <input class="edit",
                        type="text",
                        value=&self.input_value,
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
                        value=&self.input_value,
                        oninput=|e| Msg::GotInput(e.value),
                        />
                    <button onclick=|_| Msg::Connecting.into(),>{ "Connect" }</button>
                    </>
                }
            }
        }
    }
}

pub struct Context {
    console: ConsoleService,
    ws: WebSocketService,
}

impl AsMut<ConsoleService> for Context {
    fn as_mut(&mut self) -> &mut ConsoleService {
        &mut self.console
    }
}

impl AsMut<WebSocketService> for Context {
    fn as_mut(&mut self) -> &mut WebSocketService {
        &mut self.ws
    }
}

fn main() {
    web_logger::init();
    yew::initialize();
    let context = Context {
        console: ConsoleService::new(),
        ws: WebSocketService::new(),
    };
    let app: App<_, AppModel> = App::new(context);
    app.mount_to_body();
    yew::run_loop();
}
