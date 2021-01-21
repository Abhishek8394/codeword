/// Player authentication
use crate::players::PlayerId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


/// Auth challenge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthChallenge{
    /// Player ID whom challenge identifies
    pub pid: PlayerId,
    /// Challenge string
    pub challenge: String,
}


/// Internal Auth challenge containing answer too.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalAuthChallenge{
    pub challenge: AuthChallenge,
    /// Expected response string
    pub exp_response: String,
}

impl InternalAuthChallenge{
    pub fn get_player_challenge(&self) -> &AuthChallenge {
        &self.challenge
    }

    pub fn response_matches(&self, response: &AuthResponse) -> bool {
        self.exp_response == response.response
    }
}



/// Player response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthResponse{
    /// Player responding
    pub pid: PlayerId,
    /// Player response.
    pub response: String,
}

/// Builds a simple echo challenge, where player must return the same string back to auth.
pub fn build_echo_challenge(pid: PlayerId, challenge: Option<String>) -> InternalAuthChallenge {
    let challenge = match challenge{
        Some(c) => c,
        None => Uuid::new_v4().to_string(),
    };
    return InternalAuthChallenge{
        challenge: AuthChallenge{
            pid: pid,
            challenge: challenge.clone(),
        },
        exp_response: challenge,
    }
}
