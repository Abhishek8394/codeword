use crate::game::Team;
use serde::{Serialize, Deserialize};


/// Request payload for a team change request.
#[derive(Serialize, Debug, Deserialize)]
pub struct TeamChangeRequest{
    pub team: Team,
}
