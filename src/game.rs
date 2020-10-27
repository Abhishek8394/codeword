use crate::board::Board;
use crate::errors::InvalidError;
use crate::players::Player;
use std::convert::TryInto;

#[derive(Debug)]
pub enum GameState {
    Initial,
    Ready,
    End,
}

#[derive(Debug)]
pub struct Game<P: Player> {
    board: Board,
    team_one_players: Vec<P>,
    team_two_players: Vec<P>,
    team_one_spymaster_ind: Option<usize>,
    team_two_spymaster_ind: Option<usize>,
    team_one_score: u8,
    team_two_score: u8,
    next_turn: Option<bool>,
    state: GameState,
}

impl<P: Player> Game<P> {
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
            state: GameState::Initial,
        };
        game.team_one_score = game.board.get_team_one_pending_size().try_into().unwrap();
        game.team_two_score = game.board.get_team_two_pending_size().try_into().unwrap();
        Ok(game)
    }

    pub fn get_team_one_score(&self) -> u8 {
        self.team_one_score.clone()
    }
    pub fn get_team_two_score(&self) -> u8 {
        self.team_two_score.clone()
    }

    pub fn set_team_one_spymaster(&mut self, ind: usize) -> Result<(), InvalidError> {
        if ind >= self.team_one_players.len() {
            return Err(InvalidError::new("Spymaster index exceeds num players"));
        }
        self.team_one_spymaster_ind = Some(ind);
        Ok(())
    }
    pub fn set_team_two_spymaster(&mut self, ind: usize) -> Result<(), InvalidError> {
        if ind >= self.team_two_players.len() {
            return Err(InvalidError::new("Spymaster index exceeds num players"));
        }
        self.team_two_spymaster_ind = Some(ind);
        Ok(())
    }

    pub fn add_player_team_one(&mut self, player: P) {
        self.team_one_players.push(player)
    }

    pub fn add_player_team_two(&mut self, player: P) {
        self.team_two_players.push(player)
    }

    pub fn has_enough_players(&self) -> bool {
        !(self.team_one_players.len() < 2
            || self.team_two_players.len() < 2
            || self.team_one_spymaster_ind.is_none()
            || self.team_one_spymaster_ind.is_none())
    }

    pub fn begin(&mut self) -> Result<(), InvalidError> {
        match &self.state {
            GameState::Initial => {
                if self.has_enough_players() {
                    self.state = GameState::Ready;
                    return Ok(());
                }
                return Err(InvalidError::new(
                    "Not enough players. Each team needs 2 players atleast and a chosen spymaster.",
                ));
            }
            s => Err(InvalidError::new(&format!(
                "Game cannot begin from {:?} state",
                s
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::players::SimplePlayer;

    #[test]
    fn new_game() -> Result<(), InvalidError> {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let mut game: Game<SimplePlayer> = Game::new(&words)?;

        assert_eq!(game.get_team_one_score(), 8);
        assert_eq!(game.get_team_two_score(), 8);

        game.add_player_team_one(SimplePlayer::new("p1"));
        game.add_player_team_two(SimplePlayer::new("p2"));

        match &game.state {
            GameState::Initial => {}
            s => {
                assert!(false, "Game state must be Initial. Instead it was: {:?}", s);
            }
        };

        let res = game.begin();
        assert!(
            res.is_err(),
            "game shouldn't be ready without spymaster and 1 more player."
        );
        Ok(())
    }

    #[test]
    fn test_has_enough_players() -> Result<(), InvalidError> {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let mut game: Game<SimplePlayer> = Game::new(&words)?;
        game.add_player_team_one(SimplePlayer::new("p1"));
        game.add_player_team_two(SimplePlayer::new("p2"));
        assert!(
            !game.has_enough_players(),
            "each team has just 1 player. shouldnt be ready."
        );
        game.add_player_team_one(SimplePlayer::new("p1"));
        game.add_player_team_two(SimplePlayer::new("p2"));
        assert!(
            !game.has_enough_players(),
            "each team has just 2 players. But no spymaster elected. Shouldn't be ready."
        );
        let res = game.set_team_one_spymaster(0);
        assert!(res.is_ok());
        let res = game.set_team_two_spymaster(1);
        assert!(res.is_ok());
        assert!(
            game.has_enough_players(),
            "Game should have enough players now."
        );
        Ok(())
    }
}
