use crate::board::Board;
use crate::errors::{InvalidError, InvalidMoveError};
use crate::players::Player;
use std::collections::HashMap;
use std::convert::TryInto;

#[derive(Debug, PartialEq)]
pub enum Team {
    TeamOne,
    TeamTwo,
}

#[derive(Debug)]
pub struct Game<S, P: Player> {
    board: Board,
    team_one_players: HashMap<u32, P>,
    team_two_players: HashMap<u32, P>,
    team_one_spymaster_ind: Option<usize>,
    team_two_spymaster_ind: Option<usize>,
    team_one_score: u8,
    team_two_score: u8,
    next_turn: Option<Team>,
    state: S,
}

pub struct InitialGame {}

pub struct InProgressGame {}

impl<S, P: Player> Game<S, P> {
    pub fn get_team_one_score(&self) -> u8 {
        self.team_one_score.clone()
    }
    pub fn get_team_two_score(&self) -> u8 {
        self.team_two_score.clone()
    }

    pub fn has_enough_players(&self) -> bool {
        !(self.team_one_players.len() < 2
            || self.team_two_players.len() < 2
            || self.team_one_spymaster_ind.is_none()
            || self.team_one_spymaster_ind.is_none())
    }

    pub fn add_player_team_one(&mut self, player: P) {
        self.team_one_players.insert(*player.get_id(), player);
    }

    pub fn add_player_team_two(&mut self, player: P) {
        self.team_two_players.insert(*player.get_id(), player);
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

    pub fn get_player_team(&self, player: &P) -> Option<Team> {
        if self.team_one_players.contains_key(player.get_id()) {
            Some(Team::TeamOne)
        } else if self.team_two_players.contains_key(player.get_id()) {
            Some(Team::TeamTwo)
        } else {
            None
        }
    }
}

impl<P: Player> Game<InitialGame, P> {
    pub fn new(vocab: &Vec<String>) -> Result<Self, InvalidError> {
        let board = Board::new(vocab)?;
        let mut game = Game {
            board,
            team_one_players: HashMap::new(),
            team_two_players: HashMap::new(),
            team_one_spymaster_ind: None,
            team_two_spymaster_ind: None,
            team_one_score: 0,
            team_two_score: 0,
            next_turn: None,
            state: InitialGame {},
        };
        game.team_one_score = game.board.get_team_one_pending_size().try_into().unwrap();
        game.team_two_score = game.board.get_team_two_pending_size().try_into().unwrap();
        Ok(game)
    }

    pub fn can_begin(&self) -> bool {
        self.has_enough_players()
    }

    pub fn begin(self) -> Result<Game<InProgressGame, P>, InvalidError> {
        if self.can_begin() {
            // return (None, Ok(InProgressGame { game: self.game }));
            let mut game = Game::<InProgressGame, P>::from(self);
            game.next_turn = Some(Team::TeamOne);
            return Ok(game);
        }
        return Err(InvalidError::new(
            "Not enough players. Each team needs 2 players atleast and a chosen spymaster.",
        ));
    }
}

impl<P: Player> From<Game<InitialGame, P>> for Game<InProgressGame, P> {
    fn from(value: Game<InitialGame, P>) -> Game<InProgressGame, P> {
        Game {
            state: InProgressGame {},
            board: value.board,
            team_one_players: value.team_one_players,
            team_two_players: value.team_two_players,
            team_one_spymaster_ind: value.team_one_spymaster_ind,
            team_two_spymaster_ind: value.team_two_spymaster_ind,
            team_one_score: value.team_one_score,
            team_two_score: value.team_two_score,
            next_turn: value.next_turn,
        }
    }
}

impl<P: Player> Game<InProgressGame, P> {
    pub fn try_unravel(&mut self, player: &P, tile_id: u8) -> Result<(), InvalidMoveError> {
        let team_num = match self.get_player_team(player) {
            Some(team) => team,
            None => {
                return Err(InvalidMoveError::new("Player not in the team"));
            }
        };
        if team_num == *self.next_turn.as_ref().unwrap() {
            match self.board.unravel_word(tile_id as usize) {
                Ok(_) => {
                    if tile_id == self.board.danger_index() {
                        // handle Game Over.
                    } else if self.board.is_grey_index(tile_id.into()) {
                        // handle grey tile
                    } else if self.board.is_team_one_index(tile_id.into())
                        && team_num == Team::TeamOne
                    {
                        self.team_one_score =
                            self.board.get_team_one_pending_size().try_into().unwrap();
                    } else if self.board.is_team_two_index(tile_id.into())
                        && team_num == Team::TeamTwo
                    {
                        self.team_two_score =
                            self.board.get_team_two_pending_size().try_into().unwrap();
                    } else {
                        // This should be unreachable but covers future changes.
                        return Err(InvalidMoveError::new(
                            format!("Couldn't update score for unravelling: {}", tile_id).as_ref(),
                        ));
                    }
                }
                Err(e) => {
                    return Err(InvalidMoveError::new(format!("{:?}", e).as_ref()));
                }
            }
            return Ok(());
        }
        return Err(InvalidMoveError::new("Not the current team's turn"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::players::SimplePlayer;

    fn setup_valid_game() -> Result<Game<InitialGame, SimplePlayer>, InvalidError> {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let mut game = Game::new(&words)?;

        game.add_player_team_one(SimplePlayer::new("p1", 1));
        game.add_player_team_two(SimplePlayer::new("p2", 2));

        game.add_player_team_one(SimplePlayer::new("p3", 3));
        game.add_player_team_two(SimplePlayer::new("p4", 4));

        let res = game.set_team_one_spymaster(0);
        assert!(res.is_ok());
        let res = game.set_team_two_spymaster(1);
        assert!(res.is_ok());
        assert!(game.can_begin());
        Ok(game)
    }

    #[test]
    fn game_move_tracking() -> Result<(), InvalidError> {
        let game = setup_valid_game()?;
        let mut game = game.begin()?;
        assert_eq!(*(game.next_turn.as_ref().unwrap()), Team::TeamOne);
        // maybe player changed their name.
        let p1 = SimplePlayer::new("p-whatever", 1);
        let p2 = SimplePlayer::new("p2", 2);
        assert_eq!(game.get_player_team(&p1).unwrap(), Team::TeamOne);
        assert_eq!(game.get_player_team(&p2).unwrap(), Team::TeamTwo);
        let safe_move = (game.board.danger_index() + 1) % game.board.words().len() as u8;
        let res = game.try_unravel(&p2, safe_move);
        assert!(res.is_err());
        let res = game.try_unravel(&p1, safe_move);
        assert!(res.is_ok());

        Ok(())
    }

    #[test]
    fn new_game_core() -> Result<(), InvalidError> {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let mut game: Game<InitialGame, SimplePlayer> = Game::new(&words)?;

        assert_eq!(game.get_team_one_score(), 8);
        assert_eq!(game.get_team_two_score(), 8);

        game.add_player_team_one(SimplePlayer::new("p1", 1));
        game.add_player_team_two(SimplePlayer::new("p2", 2));
        Ok(())
    }

    #[test]
    fn test_begin_game() -> Result<(), InvalidError> {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let mut game = Game::new(&words)?;

        assert_eq!(game.get_team_one_score(), 8);
        assert_eq!(game.get_team_two_score(), 8);

        game.add_player_team_one(SimplePlayer::new("p1", 1));
        game.add_player_team_two(SimplePlayer::new("p2", 2));
        assert!(!game.can_begin());

        game.add_player_team_one(SimplePlayer::new("p3", 3));
        game.add_player_team_two(SimplePlayer::new("p4", 4));
        assert!(!game.can_begin());

        let res = game.set_team_one_spymaster(0);
        assert!(res.is_ok());
        let res = game.set_team_two_spymaster(1);
        assert!(res.is_ok());

        // it should work now.
        assert!(game.can_begin());
        let res = game.begin();
        assert!(res.is_ok());
        let game: Game<InProgressGame, SimplePlayer> = res.unwrap();
        assert!(game.next_turn.is_some());
        Ok(())
    }

    #[test]
    fn test_has_enough_players_core() -> Result<(), InvalidError> {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let mut game: Game<InitialGame, SimplePlayer> = Game::new(&words)?;
        game.add_player_team_one(SimplePlayer::new("p1", 1));
        game.add_player_team_two(SimplePlayer::new("p2", 2));
        assert!(
            !game.has_enough_players(),
            "each team has just 1 player. shouldnt be ready."
        );
        game.add_player_team_one(SimplePlayer::new("p1", 3));
        game.add_player_team_two(SimplePlayer::new("p2", 4));
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
