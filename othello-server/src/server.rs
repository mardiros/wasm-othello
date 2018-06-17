//! `OthelloActor` maintains list of connection client session.

use std::iter;
use std::cell::RefCell;
use std::collections::HashMap;

use rand::{self, Rng, ThreadRng};
use rand::distributions::Alphanumeric;
use actix::prelude::*;


use wscommand::{
    WsRequest,
    WsResponse,
    WsConnectedParam};

/// Message for Othello server communications

/// New Othello session is created on connection received
#[derive(Message)]
#[rtype(String)]
pub struct Connect {
    pub addr: Recipient<Syn, WsResponse>,
}

/// Session is disconnected
#[derive(Message)]
pub struct Disconnect {
    pub id: String,
}

#[derive(Message)]
pub struct ClientMessage {
    pub id: String,
    pub request: WsRequest,
}


pub struct OthelloActor {
    /// session_id to session address
    sessions: HashMap<String, Recipient<Syn, WsResponse>>,
    rng: RefCell<ThreadRng>,
}

impl Default for OthelloActor {
    fn default() -> OthelloActor {

        OthelloActor {
            sessions: HashMap::new(),
            rng: RefCell::new(rand::thread_rng()),
        }
    }
}

impl OthelloActor {
    /// Send message to all users in the room
    fn send_message(&self, resp: WsResponse, dest_id: &str) {
        if let Some(addr) = self.sessions.get(dest_id) {
            let _ = addr.do_send(resp);
        }
        else {
            warn!("Receive a message to an invalid id");
        }
    }
}

/// Make actor from `OthelloActor`
impl Actor for OthelloActor {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for OthelloActor {
    type Result = String;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {

        // register session with random id
        let id: String = iter::repeat(())
                .map(|()| self.rng.borrow_mut().sample(Alphanumeric))
                .take(40)
                .collect();
        self.sessions.insert(id.clone(), msg.addr);

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for OthelloActor {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.id).is_some() {
            debug!("Session {} closed", msg.id);
        }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for OthelloActor {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        let req = msg.request;
        let users_count = self.sessions.len();
        let resp = WsResponse::ConnectedParam(
            WsConnectedParam{
                id: msg.id.clone(),
                users_count: users_count
            }
        );
        self.send_message(resp, msg.id.as_str());
    }
}
