/// API responses
use crate::web::auth::AuthChallenge;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpStatus{
    Ok,
    Error,
}

/// Response for create player API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlayerResp{
    pub status: OpStatus,
    pub challenge: AuthChallenge,
}
