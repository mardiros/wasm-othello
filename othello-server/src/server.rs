//! `OthelloActor` maintains list of connection client session.

use std::iter;
use std::cell::RefCell;
use std::collections::HashMap;

use rand::{self, Rng, ThreadRng};
use rand::distributions::Alphanumeric;
use actix::prelude::*;

use wscommand::{Color, WsConnectedParam, WsJoinedBoard, WsOpponentJoinedBoard, WsRequest,
                WsResponse};

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

pub struct SessionData {
    // where the user is joinable
    addr: Recipient<Syn, WsResponse>,
    // the nickname received in the ConnectionParam
    nickname: Option<String>,
    // a board where the user is actually
    board: Option<String>,
}

pub struct OthelloActor {
    /// session_id to session address
    sessions: HashMap<String, SessionData>,
    /// boards (black session_id, white session_id or empty string)
    boards: HashMap<String, (String, String)>,
    /// the list of boards waiting for a partner
    boarding: Vec<String>,
    rng: RefCell<ThreadRng>,
}

impl Default for OthelloActor {
    fn default() -> OthelloActor {
        OthelloActor {
            sessions: HashMap::new(),
            boards: HashMap::new(),
            boarding: Vec::new(),
            rng: RefCell::new(rand::thread_rng()),
        }
    }
}

impl OthelloActor {
    /// Send message to all users in the room
    fn send_message(&self, resp: WsResponse, dest_id: &str) {
        if let Some(ref sess) = self.sessions.get(dest_id) {
            let _ = sess.addr.do_send(resp);
        } else {
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
        self.sessions.insert(
            id.clone(),
            SessionData {
                addr: msg.addr,
                nickname: None,
                board: None,
            },
        );

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for OthelloActor {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        // cleaning boarding
        // TODO
        // cleaning boards
        // TODO

        if self.sessions.remove(&msg.id).is_some() {
            debug!("Session {} closed", msg.id);
        }
    }
}

/// Handler for message from the websocket.
impl Handler<ClientMessage> for OthelloActor {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        let req = msg.request;
        let resp = {
            match req {
                WsRequest::ConnectingParam(ref param) => {
                    let users_count = { self.sessions.len() };
                    let session = self.sessions.get_mut(&msg.id);
                    if session.is_none() {
                        error!("Receiving an invalid session id {}", msg.id);
                        return;
                    }
                    let session = session.unwrap();
                    session.nickname = Some(param.nickname.clone());
                    let resp = WsResponse::ConnectedParam(WsConnectedParam {
                        id: msg.id.clone(),
                        users_count: users_count,
                    });
                    Some(resp)
                }
                WsRequest::JoinBoard(ref param) => {
                    info!("Boarding: {:?}", self.boarding);
                    let joined = if self.boarding.len() > 0 {
                        // join the board as a white player
                        let board_id = self.boarding.remove(0);
                        let board = self.boards.get_mut(&board_id);
                        let opponent = if let Some(brd) = board {
                            let opponent_sess = self.sessions.get(&brd.0);
                            brd.1 = param.session_id.clone();
                            if let Some(ref opp_sess) = opponent_sess {
                                if let Some(ref name) = opp_sess.nickname {
                                    // notify the first user of the board he can play
                                    let self_sess = self.sessions.get(&param.session_id);
                                    let self_nick =
                                        self_sess.unwrap().nickname.as_ref().unwrap().as_str();
                                    let back =
                                        WsResponse::OpponentJoinedBoard(WsOpponentJoinedBoard {
                                            id: board_id.clone(),
                                            opponent: self_nick.to_string(),
                                        });
                                    info!("Sending back message");
                                    let _ = opp_sess.addr.do_send(back);

                                    Some(name.to_owned())
                                } else {
                                    error!("A user cannot join a board without a nick");
                                    None
                                }
                            } else {
                                error!(
                                    "No session on white user, the board should have been cleeaned"
                                );
                                None
                            }
                        } else {
                            error!(
                                "Invalid board id {} retrieved in the boarding queue",
                                board_id
                            );
                            None
                        };
                        WsJoinedBoard {
                            id: board_id,
                            color: Color::White,
                            opponent: opponent,
                        }
                    } else {
                        // create the board and join it as a black player
                        let board_id: String = iter::repeat(())
                            .map(|()| self.rng.borrow_mut().sample(Alphanumeric))
                            .take(12)
                            .collect();
                        self.boards
                            .insert(board_id.clone(), (param.session_id.clone(), "".to_owned()));
                        self.boarding.push(board_id.clone());
                        WsJoinedBoard {
                            id: board_id,
                            color: Color::Black,
                            opponent: None,
                        }
                    };
                    Some(WsResponse::JoinedBoard(joined))
                }
            }
        };
        if let Some(r) = resp {
            self.send_message(r, msg.id.as_str());
        }
    }
}
