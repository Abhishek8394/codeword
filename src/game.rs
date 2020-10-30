use crate::board::Board;
use crate::errors::InvalidError;
use crate::players::Player;
use std::convert::TryInto;

#[derive(Debug)]
pub struct GameCore<P: Player> {
    board: Board,
    team_one_players: Vec<P>,
    team_two_players: Vec<P>,
    team_one_spymaster_ind: Option<usize>,
    team_two_spymaster_ind: Option<usize>,
    team_one_score: u8,
    team_two_score: u8,
    next_turn: Option<bool>,
}

// #[derive(Debug)]
// pub struct Game<P: Player> {
//     board: Board,
//     team_one_players: Vec<P>,
//     team_two_players: Vec<P>,
//     team_one_spymaster_ind: Option<usize>,
//     team_two_spymaster_ind: Option<usize>,
//     team_one_score: u8,
//     team_two_score: u8,
//     next_turn: Option<bool>,
//     state: GameState,
// }

impl<P: Player> GameCore<P> {
    pub fn new(vocab: &Vec<String>) -> Result<Self, InvalidError> {
        let board = Board::new(vocab)?;
        let mut game = GameCore {
            board,
            team_one_players: Vec::new(),
            team_two_players: Vec::new(),
            team_one_spymaster_ind: None,
            team_two_spymaster_ind: None,
            team_one_score: 0,
            team_two_score: 0,
            next_turn: None,
            // state: GameState::Initial,
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
}

pub struct InitialGame<P: Player> {
    game: GameCore<P>,
}

pub struct InProgressGame<P: Player> {
    game: GameCore<P>,
}
impl<P: Player> InProgressGame<P> {}
impl<P: Player> InitialGame<P> {
    pub fn new(words: &Vec<String>) -> Result<Self, InvalidError> {
        let game = GameCore::new(words)?;
        return Ok(InitialGame { game });
    }

    pub fn begin(self) -> (Option<Self>, Result<InProgressGame<P>, InvalidError>) {
        if self.game.has_enough_players() {
            return (None, Ok(InProgressGame { game: self.game }));
        }
        return (
            Some(self),
            Err(InvalidError::new(
                "Not enough players. Each team needs 2 players atleast and a chosen spymaster.",
            )),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::players::SimplePlayer;

    #[test]
    fn new_game_core() -> Result<(), InvalidError> {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let mut game: GameCore<SimplePlayer> = GameCore::new(&words)?;

        assert_eq!(game.get_team_one_score(), 8);
        assert_eq!(game.get_team_two_score(), 8);

        game.add_player_team_one(SimplePlayer::new("p1"));
        game.add_player_team_two(SimplePlayer::new("p2"));
        Ok(())
    }

    #[test]
    fn test_begin_game() -> Result<(), InvalidError> {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let mut game: InitialGame<SimplePlayer> = InitialGame::new(&words)?;

        assert_eq!(game.game.get_team_one_score(), 8);
        assert_eq!(game.game.get_team_two_score(), 8);

        game.game.add_player_team_one(SimplePlayer::new("p1"));
        game.game.add_player_team_two(SimplePlayer::new("p2"));

        let res = game.begin();
        // make sure it didn't transition to playing due to lack of players
        assert!(
            res.1.is_err(),
            "game shouldn't be ready without spymaster and 1 more player."
        );
        // get our game obj back.
        game = res.0.unwrap();

        game.game.add_player_team_one(SimplePlayer::new("p3"));
        game.game.add_player_team_two(SimplePlayer::new("p4"));
        // make sure it didn't transition to playing due to lack of spymasters
        let res = game.begin();
        // get our game obj back.
        game = res.0.unwrap();
        assert!(
            res.1.is_err(),
            "game shouldn't be ready without spymaster and 1 more player."
        );
        // it should work now.
        let res = game.game.set_team_one_spymaster(0);
        assert!(res.is_ok());
        let res = game.game.set_team_two_spymaster(1);
        assert!(res.is_ok());
        let res = game.begin();
        assert!(res.1.is_ok());
        let _game: InProgressGame<SimplePlayer> = res.1.unwrap();
        Ok(())
    }

    #[test]
    fn test_has_enough_players_core() -> Result<(), InvalidError> {
        let words: Vec<String> = (0..25).map(|x| format!("word-{}", x)).collect();
        let mut game: GameCore<SimplePlayer> = GameCore::new(&words)?;
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
