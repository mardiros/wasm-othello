///
/// Request parameters
///

/// Connect parameter
#[derive(Deserialize, Debug)]
pub struct WsConnectingParam {
    /// a user nickname
    pub nickname: String,
}

/// Connect parameter
#[derive(Deserialize, Debug)]
pub struct WsJoinBoard {
    /// a user nickname
    pub session_id: String,
}

/// User is sending a move
#[derive(Serialize, Deserialize, Debug)]
pub struct WsPlayBoard {
    /// a board id
    pub session_id: String,
    /// a board id
    pub board_id: String,
    /// a position
    pub pos: (usize, usize),
}

/// This type handle type per command
#[derive(Deserialize, Debug)]
pub enum WsRequest {
    /// `connect` command parameters
    ConnectingParam(WsConnectingParam),
    JoinBoard(WsJoinBoard),
    PlayBoard(WsPlayBoard),
}

///
/// Responses
///

/// Connected parameter
#[derive(Serialize, Debug)]
pub struct WsConnectedParam {
    /// a session id
    pub id: String,
    pub users_count: usize,
}

#[derive(Serialize, Debug)]
pub enum Color {
    Black,
    White,
}

/// Connected parameter
#[derive(Serialize, Debug)]
pub struct WsJoinedBoard {
    /// a board id
    pub id: String,
    /// the user color
    pub color: Color,
    /// the nick of the opponnent user
    pub opponent: Option<String>,
}

/// Connected parameter
#[derive(Serialize, Debug)]
pub struct WsOpponentJoinedBoard {
    /// a board id
    pub id: String,
    /// the nick of the opponnent user
    pub opponent: String,
}

/// Connected parameter
#[derive(Serialize, Debug)]
pub struct WsOpponentDisconnected {
    /// a session id of the user that will be ejected
    pub session_id: String,
    /// a board id
    pub board_id: String,
}

/// This type handle type per command
#[derive(Message, Serialize, Debug)]
pub enum WsResponse {
    /// `connect` command parameters
    ConnectedParam(WsConnectedParam),
    JoinedBoard(WsJoinedBoard),
    OpponentJoinedBoard(WsOpponentJoinedBoard),
    PlayedBoard(WsPlayBoard),
    OpponentDisconnected(WsOpponentDisconnected),
}
