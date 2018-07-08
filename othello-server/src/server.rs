//! `OthelloActor` maintains list of connection client session.

use std::iter;
use std::cell::RefCell;
use std::collections::HashMap;

use rand::{self, Rng, ThreadRng};
use rand::distributions::Alphanumeric;
use actix::prelude::*;

use wscommand::{Color, WsConnectedParam, WsJoinedBoard, WsOpponentDisconnected,
                WsOpponentJoinedBoard, WsPlayBoard, WsRequest, WsResponse};

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
    board_id: Option<String>,
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
                board_id: None,
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
        let boarding = self.boarding.clone(); // cannot move out of borrowed content
        if let Some(session) = self.sessions.remove(&msg.id) {
            info!("Closing session {}", msg.id);
            if let Some(ref board_id) = session.board_id {
                if let Some(brd) = self.boards.remove(board_id) {
                    info!("Closing board {}", board_id);
                    // if the board where waiing for someone
                    self.boarding = boarding.into_iter().filter(|b| b != board_id).collect();
                    if brd.1 == msg.id {
                        let opponent_sess = self.sessions.get(&brd.0);
                        if let Some(opp_sess) = opponent_sess {
                            let back = WsResponse::OpponentDisconnected(WsOpponentDisconnected {
                                session_id: brd.0.clone(),
                                board_id: board_id.clone(),
                            });
                            let _ = opp_sess.addr.do_send(back);
                        }
                    } else if brd.0 == msg.id {
                        let opponent_sess = self.sessions.get(&brd.1);
                        if let Some(opp_sess) = opponent_sess {
                            let back = WsResponse::OpponentDisconnected(WsOpponentDisconnected {
                                session_id: brd.1.clone(),
                                board_id: board_id.clone(),
                            });
                            let _ = opp_sess.addr.do_send(back);
                        }
                    }
                }
            }
            info!("Session {} closed", msg.id);
        } else {
            error!("Unregistered session has disconnect");
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
                        session_id: msg.id.clone(),
                        users_count: users_count,
                    });
                    Some(resp)
                }
                WsRequest::JoinBoard(ref param) => {
                    info!("Boarding: {:?}", self.boarding);
                    if self.boarding.len() > 0 {
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
                                    if let Some(ref sess) = self_sess {
                                        let self_nick = sess.nickname.as_ref().unwrap().as_str();
                                        let back = WsResponse::OpponentJoinedBoard(
                                            WsOpponentJoinedBoard {
                                                session_id: brd.0.clone(),
                                                board_id: board_id.clone(),
                                                opponent: self_nick.to_string(),
                                            },
                                        );
                                        info!("Sending back message");
                                        let _ = opp_sess.addr.do_send(back);
                                    }

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

                        // register the board_id
                        if opponent.is_some() {
                            let mut self_sess = self.sessions.get_mut(&param.session_id);
                            if let Some(ref mut sess) = self_sess {
                                sess.board_id = Some(board_id.clone());
                            }
                        }

                        Some(WsResponse::JoinedBoard(WsJoinedBoard {
                            session_id: param.session_id.clone(),
                            board_id: board_id,
                            color: Color::White,
                            opponent: opponent,
                        }))
                    } else {
                        let board_id: String = iter::repeat(())
                            .map(|()| self.rng.borrow_mut().sample(Alphanumeric))
                            .take(12)
                            .collect();
                        let self_sess = self.sessions.get_mut(&param.session_id);
                        if let Some(sess) = self_sess {
                            // create the board and join it as a black player
                            self.boards.insert(
                                board_id.clone(),
                                (param.session_id.clone(), "".to_owned()),
                            );
                            self.boarding.push(board_id.clone());
                            // register the user on the created board
                            sess.board_id = Some(board_id.clone());

                            Some(WsResponse::JoinedBoard(WsJoinedBoard {
                                session_id: param.session_id.clone(),
                                board_id: board_id,
                                color: Color::Black,
                                opponent: None,
                            }))
                        } else {
                            error!("Unknown session id receided to join the board");
                            None
                        }
                    }
                }
                WsRequest::PlayBoard(ref param) => {
                    let sess_id = param.session_id.as_str();
                    let board = self.boards.get(&param.board_id);
                    if let Some(ref brd) = board {
                        let opponent_msg = if brd.0.as_str() == sess_id {
                            // black played, send the move to the white
                            let opponent_sess = self.sessions.get(&brd.1);
                            if let Some(ref opp_sess) = opponent_sess {
                                Some((
                                    &opp_sess.addr,
                                    WsResponse::PlayedBoard(WsPlayBoard {
                                        session_id: brd.1.clone(),
                                        board_id: param.board_id.clone(),
                                        pos: param.pos.clone(),
                                    }),
                                ))
                            } else {
                                // Should not happen
                                // the session is in error, should drop the board
                                None
                            }
                        } else if brd.1.as_str() == sess_id {
                            // white played, send the move to the black
                            let opponent_sess = self.sessions.get(&brd.0);
                            if let Some(ref opp_sess) = opponent_sess {
                                Some((
                                    &opp_sess.addr,
                                    WsResponse::PlayedBoard(WsPlayBoard {
                                        session_id: brd.0.clone(),
                                        board_id: param.board_id.clone(),
                                        pos: param.pos.clone(),
                                    }),
                                ))
                            } else {
                                // Should not happen
                                // the session is in error, should drop the board
                                None
                            }
                        } else {
                            // Should not happen
                            // DROP THE BOARD
                            // Send End Board to players
                            None
                        };
                        if let Some((addr, msg)) = opponent_msg {
                            info!("Forwarding the move");
                            let _ = addr.do_send(msg);
                        }
                        None
                    } else {
                        None
                    }
                }
                WsRequest::GameOver(ref param) => {

                    let self_sess = self.sessions.get(&param.session_id);
                    if let Some(sess) = self_sess {
                        let del_board = if sess.board_id.as_ref() == Some(&param.board_id) {
                            let mut board = self.boards.get_mut(&param.board_id);
                            if let Some(ref mut brd) = board {
                                //
                                let sess_id = param.session_id.as_str();
                                if brd.0.as_str() == sess_id {
                                    info!("Closing black board");
                                    brd.0 = "".to_string();
                                } else if brd.1.as_str() == sess_id {
                                    info!("Closing white board");
                                    brd.0 = "".to_string();
                                }
                                if brd.0.len() == 0 && brd.1.len() == 0 {
                                    Some(&param.board_id)
                                }
                                else {
                                    None
                                }
                            }
                            else {
                                error!(
                                    "User {} is pointing to an deleted board id",
                                    sess.nickname.as_ref().unwrap_or(&"UNKNWOWN".to_string()));
                                None
                            }
                        }
                        else {
                            error!(
                                "User {} is hijacking a board id",
                                sess.nickname.as_ref().unwrap_or(&"UNKNWOWN".to_string()));
                            None
                        };
                        if let Some(ref board_id) = del_board {
                            info!("Removing the board {}", board_id);
                            let _ = self.boards.remove(*board_id);
                        }
                    }
                    None
                }
            }
        };
        if let Some(r) = resp {
            self.send_message(r, msg.id.as_str());
        }
    }
}
