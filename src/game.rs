use crate::players::Player;
use crate::board::Board;
use crate::errors::InvalidError;
use std::convert::TryInto;

#[derive(Debug)]
pub struct Game <P: Player>{
    board: Board,
    team_one_players: Vec<P>,
    team_two_players: Vec<P>,
    team_one_spymaster_ind: Option<usize>,
    team_two_spymaster_ind: Option<usize>,
    team_one_score: u8,
    team_two_score: u8,
    next_turn: Option<bool>,
}

impl<P: Player> Game <P>{
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

    pub fn get_team_one_score(&self) -> u8{
        self.team_one_score.clone()
    }
    pub fn get_team_two_score(&self) -> u8{
        self.team_two_score.clone()
    }

    pub fn add_player_team_one(&mut self, player: P){
        self.team_one_players.push(player)
    }

    pub fn add_player_team_two(&mut self, player: P){
        self.team_two_players.push(player)
    }
}

#[cfg(test)]
mod tests {
    use crate::players::SimplePlayer;
    use super::*;

    #[test]
    fn new_game() -> Result<(), InvalidError>{
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let mut game: Game<SimplePlayer> = Game::new(&words)?;

        assert_eq!(game.get_team_one_score(), 8);
        assert_eq!(game.get_team_two_score(), 8);

        game.add_player_team_one(SimplePlayer::new("p1"));
        game.add_player_team_two(SimplePlayer::new("p2"));

        Ok(())
    }
}
