/// This type is an expected response from a websocket connection.
#[derive(Serialize, Debug)]
pub struct WsConnectingParam<'a> {
    pub nickname: &'a str,
}

#[derive(Serialize, Debug)]
pub struct WsJoinBoard<'a> {
    pub session_id: &'a str,
}

#[derive(Serialize, Debug)]
pub struct WsPlayBoard<'a> {
    pub session_id: &'a str,
    pub board_id: &'a str,
    pub pos: (usize, usize),
}

#[derive(Serialize, Debug)]
pub enum WsRequest<'a> {
    ConnectingParam(WsConnectingParam<'a>),
    JoinBoard(WsJoinBoard<'a>),
    PlayBoard(WsPlayBoard<'a>),
}

#[derive(Deserialize, Debug)]
pub struct WsConnectedParam {
    /// a session id
    pub id: String,
    /// the number of users that are connected to the game
    pub users_count: usize,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub enum Color {
    Black,
    White,
}

#[derive(Deserialize, Debug)]
pub struct WsJoinedBoard {
    /// a board_id to reuse while playing
    pub id: String,
    /// the color where the user play
    pub color: Color,
    // the nickname received in the ConnectionParam
    pub opponent: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct WsOpponentJoinedBoard {
    /// a board_id to reuse while playing
    pub id: String,
    // the nickname received in the ConnectionParam
    pub opponent: String,
}

#[derive(Deserialize, Debug)]
pub struct WsPlayedBoard {
    pub session_id: String,
    pub board_id: String,
    pub pos: (usize, usize),
}

#[derive(Deserialize, Debug)]
pub enum WsResponse {
    /// `connected` response parameters
    ConnectedParam(WsConnectedParam),
    /// `join board` command parameters
    JoinedBoard(WsJoinedBoard),
    /// when the opponent join the board someone created
    OpponentJoinedBoard(WsOpponentJoinedBoard),
    /// Reveiced the move from the opponent,
    PlayedBoard(WsPlayedBoard),
}
