#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate byteorder;
extern crate bytes;
extern crate futures;
extern crate rand;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

extern crate tokio_core;
extern crate tokio_io;

#[macro_use]
extern crate actix;
extern crate actix_web;

use std::time::Instant;

use actix::{fut, Actor, Addr, Arbiter, Handler, Running, StreamHandler, Syn, prelude::*};
use actix_web::server::HttpServer;
use actix_web::{ws, App, Error, HttpRequest, HttpResponse};

mod server;
mod wscommand;

use wscommand::{WsRequest, WsResponse};

/// This is our websocket route state, this state is shared with all route
/// instances via `HttpContext::state()`
struct WsOthelloSessionState {
    addr: Addr<Syn, server::OthelloActor>,
}

/// Entry point for our route
fn ws_route(req: HttpRequest<WsOthelloSessionState>) -> Result<HttpResponse, Error> {
    ws::start(
        req,
        WsOthelloSession {
            id: "".to_string(),
            hb: Instant::now(),
            nickname: None,
        },
    )
}

struct WsOthelloSession {
    /// unique session id
    id: String,
    /// Client must send ping at least once per 10 seconds, otherwise we drop
    /// connection.
    hb: Instant,
    /// peer name
    nickname: Option<String>,
}

impl Actor for WsOthelloSession {
    type Context = ws::WebsocketContext<Self, WsOthelloSessionState>;

    /// Method is called on actor start.
    /// We register ws session with OthelloActor
    fn started(&mut self, ctx: &mut Self::Context) {
        // register self in othello server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsOthelloSessionState, state is shared
        // across all routes within application
        let addr: Addr<Syn, _> = ctx.address();
        ctx.state()
            .addr
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        // notify chat server
        ctx.state().addr.do_send(server::Disconnect { id: self.id.clone() });
        Running::Stop
    }
}

/// WebSocket message handler
/// Text message are json parsed and send to the OthelloActor
impl StreamHandler<ws::Message, ws::ProtocolError> for WsOthelloSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                debug!("Ping Received");
                debug!("Sending Pong");
                ctx.pong(&msg)
            }
            ws::Message::Pong(_) => {
                debug!("Pong Received");
                debug!("Sending Ping");
                self.hb = Instant::now()
            }
            ws::Message::Text(text) => {
                let req: Result<WsRequest,_> = serde_json::from_str(text.as_str());
                if req.is_err() {
                    ctx.stop();
                    return
                }
                let req = req.unwrap();
                ctx.state().addr.do_send(server::ClientMessage {
                    id: self.id.clone(),
                    request: req,
                })
            }
            ws::Message::Binary(_) => {
                error!("Unexpected binary, give up");
                ctx.stop();
            }
            ws::Message::Close(_) => {
                ctx.stop();
            }
        }
    }
}


/// Send the websocket Response to the peer websocket
impl Handler<WsResponse> for WsOthelloSession {
    type Result = ();

    fn handle(&mut self, resp: WsResponse, ctx: &mut Self::Context) {
        let resp = serde_json::to_string(&resp).unwrap();
        info!("Sending response back: {}", resp);
        ctx.text(resp);
    }
}


fn main() {
    let _ = pretty_env_logger::init();
    let sys = actix::System::new("othello-server");

    // Start chat server actor in separate thread
    let server: Addr<Syn, _> = Arbiter::start(|_| server::OthelloActor::default());

    // Create Http server with websocket support
    HttpServer::new(move || {
        // Websocket sessions state
        let state = WsOthelloSessionState {
            addr: server.clone(),
        };

        App::with_state(state)
                // websocket
                .resource("/ws/", |r| r.route().f(ws_route))
    }).bind("[::1]:8080")
        .unwrap()
        .start();

    info!("Started http server: [::1]:8080");
    let _ = sys.run();
}
