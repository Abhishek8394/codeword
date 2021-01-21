/// Websocket protocol
use serde::{Serialize, Deserialize};

pub use crate::web::auth::{AuthResponse, AuthChallenge};

/// Wrapper for all Websocket messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WSMessage{
    AuthResponse,
}

