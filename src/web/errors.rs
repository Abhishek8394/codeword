use crate::errors::InvalidError;
use warp::reject::Reject;

impl Reject for InvalidError {}

#[derive(Debug, Clone)]
pub struct DuplicateLobbyError {
    dup_id: String,
}

impl DuplicateLobbyError {
    pub fn new(lobby_id: &str) -> Self {
        Self {
            dup_id: String::from(lobby_id),
        }
    }
}

impl Reject for DuplicateLobbyError {}
