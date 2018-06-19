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

/// This type handle type per command
#[derive(Deserialize, Debug)]
pub enum WsRequest {
    /// `connect` command parameters
    ConnectingParam(WsConnectingParam),
    JoinBoard(WsJoinBoard),
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
}

/// This type handle type per command
#[derive(Message, Serialize, Debug)]
pub enum WsResponse {
    /// `connect` command parameters
    ConnectedParam(WsConnectedParam),
    JoinedBoard(WsJoinedBoard),
}
