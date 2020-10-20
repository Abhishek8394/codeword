use crate::board::Board;
use crate::errors::InvalidError;
use std::convert::TryInto;

#[derive(Debug, Clone)]
struct Player {
    name: String,
}

#[derive(Debug)]
pub struct Game {
    board: Board,
    team_one_players: Vec<Player>,
    team_two_players: Vec<Player>,
    team_one_spymaster_ind: Option<usize>,
    team_two_spymaster_ind: Option<usize>,
    team_one_score: u8,
    team_two_score: u8,
    next_turn: Option<bool>,
}

impl Game {
    pub fn new(vocab: &Vec<String>) -> Result<Self, InvalidError> {
        let board = Board::new(vocab)?;
        let mut game = Game {
            board,
            team_one_players: Vec::new(),
            team_two_players: Vec::new(),
            team_one_spymaster_ind: None,
            team_two_spymaster_ind: None,
            team_one_score: 0,
            team_two_score: 0,
            next_turn: None,
        };
        game.team_one_score = game.board.get_team_one_pending_size().try_into().unwrap();
        game.team_two_score = game.board.get_team_two_pending_size().try_into().unwrap();
        Ok(game)
    }
}
