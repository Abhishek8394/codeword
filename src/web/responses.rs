/// API responses
use crate::web::auth::AuthChallenge;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OpStatus {
    Ok,
    Error,
}

/// Response for create player API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlayerResp {
    pub status: OpStatus,
    pub challenge: AuthChallenge,
}

/// Response for team change API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamChangeResponse {
    pub status: OpStatus,
}

impl warp::reject::Reject for TeamChangeResponse{}
