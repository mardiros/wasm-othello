///
/// Request parameters
///

/// Connect parameter
#[derive(Deserialize, Debug)]
pub struct WsConnectingParam {
    /// a user nickname
    pub nickname: String,
}

/// This type handle type per command
#[derive(Deserialize, Debug)]
pub enum WsRequest {
    /// `connect` command parameters
    ConnectingParam(WsConnectingParam),
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


/// This type handle type per command
#[derive(Message, Serialize, Debug)]
pub enum WsResponse {
    /// `connect` command parameters
    ConnectedParam(WsConnectedParam)
}

