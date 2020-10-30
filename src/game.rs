use crate::board::Board;
use crate::errors::{InvalidError, InvalidMoveError};
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

pub struct FinishedGame<P: Player> {
    game: GameCore<P>,
}

/// # POSSIBLE Strats for modelling multi-class state transitions.
/// ## Strat 1: Wrap state in enum
/// Use enum to wrap game states. This requires `match` littered  everywhere
/// and potential invalid transitions.
enum StatefulGame<P: Player>{
    Initial (InitialGame<P>),
    InProgress (InProgressGame<P>),
    Finished (FinishedGame<P>),
}
type BeginResult<P: Player> = Result<StatefulGame<P>, InvalidError>;
/// ## Strat 2: Type & Trait spam
/// For a given combination of possible output state, define a trait that binds them.
/// Constrait state output to this new trait. This ensures only allowed valid states
/// and can enforce expectations at compile time.
/// However, if expectations cannot be specified at compile time, you've got more work than it solves.
/// Cleanest and best if have compile time expectations.
pub trait WillBePlayable {}
impl<P: Player> WillBePlayable for InitialGame<P>{}
impl<P: Player> WillBePlayable for InProgressGame<P>{}
type BeginResultV2<G: WillBePlayable> = Result<G, InvalidError>;
/// ## Strat 3: Group All potential states and wrap in result.
/// Either wrap possible states in a tuple of `Option<State>` or an enum on state. Wrap output in Result.
/// Can enforce expectation at runtime using match or indexing in tuple. For runtime expectations,
/// match and if-else on index will do the job. This improves on trait hack in-terms of being able 
/// to easily identify and operate on expected type at runtime. Because reversing from trait to 
/// concrete is not easy if possible.
// 3.1
type InitialOrInProgressGame<P: Player> = (Option<InitialGame<P>>, Option<InProgressGame<P>>);
type BeginResultV3p1<P: Player> = Result<InitialOrInProgressGame<P>, InvalidError>;
// 3.2
enum IniOrProgEnum<P:Player>{
    Initial(InitialGame<P>),
    InProgress (InProgressGame<P>),
}
type BeginResultV3p2<P: Player> = Result<IniOrProgEnum<P>, InvalidError>;
// ---

impl<P: Player> InitialGame<P> {
    pub fn new(words: &Vec<String>) -> Result<Self, InvalidError> {
        let game = GameCore::new(words)?;
        return Ok(InitialGame { game });
    }

    pub fn add_player_team_one(&mut self, player: P) {
        self.game.add_player_team_one(player)
    }

    pub fn add_player_team_two(&mut self, player: P) {
        self.game.add_player_team_two(player)
    }

    pub fn set_team_one_spymaster(&mut self, ind: usize) -> Result<(), InvalidError> {
        self.game.set_team_one_spymaster(ind)
    }

    pub fn set_team_two_spymaster(&mut self, ind: usize) -> Result<(), InvalidError> {
        self.game.set_team_two_spymaster(ind)
    }

    pub fn get_team_one_score(&self) -> u8 {
        self.game.get_team_one_score()
    }

    pub fn get_team_two_score(&self) -> u8 {
        self.game.get_team_two_score()
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

impl<P: Player> InProgressGame<P> {
    pub fn add_player_team_one(&mut self, player: P) {
        self.game.add_player_team_one(player)
    }

    pub fn add_player_team_two(&mut self, player: P) {
        self.game.add_player_team_two(player)
    }

    pub fn get_team_one_score(&self) -> u8 {
        self.game.get_team_one_score()
    }

    pub fn get_team_two_score(&self) -> u8 {
        self.game.get_team_two_score()
    }

    pub fn unravel(&mut self, ind: usize) -> Result<(Option<Self>, Result<FinishedGame<P>, InvalidError>), InvalidMoveError>{
        todo!()
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

        game.add_player_team_one(SimplePlayer::new("p1"));
        game.add_player_team_two(SimplePlayer::new("p2"));

        let res = game.begin();
        // make sure it didn't transition to playing due to lack of players
        assert!(
            res.1.is_err(),
            "game shouldn't be ready without spymaster and 1 more player."
        );
        // get our game obj back.
        game = res.0.unwrap();

        game.add_player_team_one(SimplePlayer::new("p3"));
        game.add_player_team_two(SimplePlayer::new("p4"));
        // make sure it didn't transition to playing due to lack of spymasters
        let res = game.begin();
        // get our game obj back.
        game = res.0.unwrap();
        assert!(
            res.1.is_err(),
            "game shouldn't be ready without spymaster and 1 more player."
        );
        // it should work now.
        let res = game.set_team_one_spymaster(0);
        assert!(res.is_ok());
        let res = game.set_team_two_spymaster(1);
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
