// use serde::{Serialize, Deserialize};
use crate::game::Game;
use crate::game::InProgressGame;
use crate::game::InitialGame;

use super::players::WebAppPlayer;

// #[derive(Serialize, Deserialize)]
pub enum GameWrapper {
    InitialGame(Game<InitialGame, WebAppPlayer>),
    InProgressGame(Game<InProgressGame, WebAppPlayer>),
}

// #[derive(Serialize, Deserialize)]
pub struct Lobby {
    id: String,
    player_ids: Vec<String>,
    game: GameWrapper,
}
