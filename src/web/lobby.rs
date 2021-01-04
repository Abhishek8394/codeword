// use serde::{Serialize, Deserialize};
use crate::errors::InvalidError;
use crate::game::Game;
use crate::game::InProgressGame;
use crate::game::InitialGame;
use std::collections::HashSet;

use super::players::WebAppPlayer;

// #[derive(Serialize, Deserialize)]
pub enum GameWrapper {
    InitialGame(Game<InitialGame, WebAppPlayer>),
    InProgressGame(Game<InProgressGame, WebAppPlayer>),
}

impl GameWrapper {
    pub fn new(words: &Vec<String>) -> Result<GameWrapper, InvalidError> {
        let g = Game::new(words)?;
        Ok(GameWrapper::InitialGame(g))
    }
}

// #[derive(Serialize, Deserialize)]
pub struct Lobby {
    pub id: String,
    player_ids: HashSet<String>,
    game: GameWrapper,
}

impl Lobby {
    pub fn new(id: &str, player_ids: &Vec<String>, game: GameWrapper) -> Self {
        Lobby {
            id: String::from(id),
            player_ids: player_ids
                .iter()
                .map(|x| String::from(x))
                .collect::<HashSet<String>>(),
            game,
        }
    }

    pub fn add_player_id(&mut self, pid: &str) {
        self.player_ids.insert(String::from(pid));
    }

    pub fn get_num_players(&self) -> usize {
        self.player_ids.len()
    }
}
