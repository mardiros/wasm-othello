use yew::services::websocket::WebSocketService;
use yew::services::interval::IntervalService;

pub struct Context {
    ws: WebSocketService,
    interval: IntervalService,
}

impl AsMut<IntervalService> for Context {
    fn as_mut(&mut self) -> &mut IntervalService {
        &mut self.interval
    }
}

impl AsMut<WebSocketService> for Context {
    fn as_mut(&mut self) -> &mut WebSocketService {
        &mut self.ws
    }
}

impl Context {
    pub fn new() -> Self {
        Context {
            interval: IntervalService::new(),
            ws: WebSocketService::new(),
        }
    }
}
