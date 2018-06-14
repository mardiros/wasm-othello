#![allow(unused_variables)]
extern crate byteorder;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;

#[macro_use]
extern crate actix;
extern crate actix_web;

use std::time::Instant;

use actix::{Addr, Syn, StreamHandler, Handler, Running, Arbiter, Actor, fut, prelude::*};
use actix_web::server::HttpServer;
use actix_web::{fs, http, ws, App, Error, HttpRequest, HttpResponse};

mod server;

/// This is our websocket route state, this state is shared with all route
/// instances via `HttpContext::state()`
struct WsOthelloSessionState {
    addr: Addr<Syn, server::OthelloServer>,
}

/// Entry point for our route
fn ws_route(req: HttpRequest<WsOthelloSessionState>) -> Result<HttpResponse, Error> {
    ws::start(
        req,
        WsOthelloSession {
            id: 0,
            hb: Instant::now(),
            room: "Main".to_owned(),
            name: None,
        },
    )
}

struct WsOthelloSession {
    /// unique session id
    id: usize,
    /// Client must send ping at least once per 10 seconds, otherwise we drop
    /// connection.
    hb: Instant,
    /// joined room
    room: String,
    /// peer name
    name: Option<String>,
}

impl Actor for WsOthelloSession {
    type Context = ws::WebsocketContext<Self, WsOthelloSessionState>;

    /// Method is called on actor start.
    /// We register ws session with OthelloServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // register self in chat server. `AsyncContext::wait` register
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
        ctx.state().addr.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::Message> for WsOthelloSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<ws::Message, ws::ProtocolError> for WsOthelloSession {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => {
                println!("Ping Received");
                println!("Sending Pong");
                ctx.pong(&msg)
            },
            ws::Message::Pong(msg) => {
                println!("Pong Received");
                println!("Sending Ping");
                self.hb = Instant::now()
            },
            ws::Message::Text(text) => {
                let m = text.trim();
                // we check for /sss type of messages
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(2, ' ').collect();
                    match v[0] {
                        "/list" => {
                            // Send ListRooms message to chat server and wait for
                            // response
                            println!("List rooms");
                            ctx.state()
                                .addr
                                .send(server::ListRooms)
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(rooms) => {
                                            for room in rooms {
                                                ctx.text(room);
                                            }
                                        }
                                        _ => println!("Something is wrong"),
                                    }
                                    fut::ok(())
                                })
                                .wait(ctx)
                            // .wait(ctx) pauses all events in context,
                            // so actor wont receive any new messages until it get list
                            // of rooms back
                        }
                        "/join" => {
                            if v.len() == 2 {
                                self.room = v[1].to_owned();
                                ctx.state().addr.do_send(server::Join {
                                    id: self.id,
                                    name: self.room.clone(),
                                });

                                ctx.text("joined");
                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/name" => {
                            if v.len() == 2 {
                                self.name = Some(v[1].to_owned());
                            } else {
                                ctx.text("!!! name is required");
                            }
                        }
                        _ => ctx.text(format!("!!! unknown command: {:?}", m)),
                    }
                } else {
                    let msg = if let Some(ref name) = self.name {
                        format!("{}: {}", name, m)
                    } else {
                        m.to_owned()
                    };
                    // send message to chat server
                    ctx.state().addr.do_send(server::ClientMessage {
                        id: self.id,
                        msg: msg,
                        room: self.room.clone(),
                    })
                }
            }
            ws::Message::Binary(bin) => println!("Unexpected binary"),
            ws::Message::Close(_) => {
                ctx.stop();
            }
        }
    }
}

fn main() {
    let _ = env_logger::init();
    let sys = actix::System::new("othello-server");

    // Start chat server actor in separate thread
    let server: Addr<Syn, _> = Arbiter::start(|_| server::OthelloServer::default());

    // Create Http server with websocket support
    HttpServer::new(move || {
        // Websocket sessions state
        let state = WsOthelloSessionState {
            addr: server.clone(),
        };

        App::with_state(state)
                // redirect to websocket.html
                .resource("/", |r| r.method(http::Method::GET).f(|_| {
                    HttpResponse::Found()
                        .header("LOCATION", "/static/websocket.html")
                        .finish()
                }))
                // websocket
                .resource("/ws/", |r| r.route().f(ws_route))
                // static resources
                .handler("/static/", fs::StaticFiles::new("static/"))
    }).bind("[::1]:8080")
        .unwrap()
        .start();

    println!("Started http server: [::1]:8080");
    let _ = sys.run();
}
