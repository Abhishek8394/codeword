use crate::board::{Board, MinimalBoardView, FullBoardView, PlayerView};
use crate::errors::{InvalidError, InvalidMoveError};
use crate::players::Player;
use crate::players::PlayerId;
use serde::Serialize;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::Display;

static TARGET_SCORE: u8 = 0;

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum Team {
    TeamOne,
    TeamTwo,
}

impl Team {
    pub fn get_id(&self) -> u8 {
        match self {
            &Self::TeamOne => 1,
            &Self::TeamTwo => 2,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum WinReason {
    ScoreReached,
    OpponentDangerDraw,
}

impl Display for WinReason {
    fn fmt(&self, fmtr: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let msg = match self {
            Self::ScoreReached => "Score reached.",
            Self::OpponentDangerDraw => "Other team drew the danger card",
        };
        write!(fmtr, "{}", msg)
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum MoveResult {
    Win(Team, WinReason),
    Continue,
}

/// Compact struct to contain dynamically changing data. Just the bits that change every turn.
/// `Board` is not included because it varies based on player.
/// TODO: Win/lose info.
#[derive(Serialize)]
pub struct DynamicGameInfoView {
    team_one_score: u8,
    team_two_score: u8,
    next_turn: Option<Team>,
    result: MoveResult,
}

/// Relatively less frequently changing data.
/// `Board` is not included because it varies based on player.
#[derive(Serialize)]
pub struct GameInfoView<S> {
    team_one_players: Vec<String>,
    team_two_players: Vec<String>,
    team_one_spymaster_ind: Option<usize>,
    team_two_spymaster_ind: Option<usize>,
    state: S,
    stats: DynamicGameInfoView,
}

#[derive(Serialize)]
pub struct MinimalGameInfoView<S> {
    game_info: GameInfoView<S>,
    board: MinimalBoardView,
}

#[derive(Serialize)]
pub struct FullGameInfoView<S> {
    game_info: GameInfoView<S>,
    board: FullBoardView,
}

pub type FullGameInfoViewResult<S> = Result<FullGameInfoView<S>, InvalidError>;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Serialize)]
pub struct InitialGame {}

#[derive(Debug, Clone, Serialize)]
pub struct InProgressGame {}

impl<S: Clone, P: Player> Game<S, P> {
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

    pub fn get_game_info(&self) -> GameInfoView<S> {
        GameInfoView {
            team_one_players: self
                .team_one_players
                .values()
                .map(|p| String::from(p.get_name()))
                .collect(),
            team_two_players: self
                .team_two_players
                .values()
                .map(|p| String::from(p.get_name()))
                .collect(),
            team_one_spymaster_ind: self.team_one_spymaster_ind.clone(),
            team_two_spymaster_ind: self.team_one_spymaster_ind.clone(),
            state: self.state.clone(),
            stats: self.get_dynamic_game_info(),
        }
    }

    pub fn has_player(&self, pid: &PlayerId) -> bool {
        self.team_one_players.contains_key(&pid) || self.team_two_players.contains_key(&pid)
    }

    pub fn is_team_one_spymaster(&self, pid: &PlayerId) -> bool {
        self.team_one_spymaster_ind.is_some()
            && *(self.team_one_spymaster_ind.as_ref().unwrap()) as u32 == *pid
    }

    pub fn is_team_two_spymaster(&self, pid: &PlayerId) -> bool {
        self.team_two_spymaster_ind.is_some()
            && *(self.team_two_spymaster_ind.as_ref().unwrap()) as u32 == *pid
    }

    pub fn get_full_game_info(&self, player: &P) -> Result<FullGameInfoView<S>, InvalidError> {
        let pid = player.get_id();
        let board_view: BoardView;
        if self.is_team_one_spymaster(&pid) || self.is_team_two_spymaster(&pid) {
            board_view = BoardView::FullSpyMasterView(self.board.get_full_spymaster_view());
        } else {
            // allow players not in any team too.
            board_view = BoardView::FullPlayerView(self.board.get_full_regular_player_view());
        }
        let game_info = self.get_game_info();
        return Ok(FullGameInfoView {
            game_info,
            board: board_view,
        });
    }

    pub fn get_dynamic_game_info(&self) -> DynamicGameInfoView {
        DynamicGameInfoView {
            team_one_score: self.team_one_score,
            team_two_score: self.team_two_score,
            next_turn: self.next_turn.clone(),
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

    pub fn get_initial_full_game_info(
        &self,
        player: &P,
    ) -> Result<FullGameInfoView<InitialGame>, InvalidError> {
        return self.get_full_game_info(&player);
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
    pub fn try_unravel(&mut self, player: &P, tile_id: u8) -> Result<MoveResult, InvalidMoveError> {
        let team_num = match self.get_player_team(player) {
            Some(team) => team,
            None => {
                return Err(InvalidMoveError::new("Player not in the team"));
            }
        };

        let mut move_result: MoveResult = MoveResult::Continue;

        if team_num == *self.next_turn.as_ref().unwrap() {
            match self.board.unravel_word(tile_id as usize) {
                Ok(_) => {
                    if tile_id == self.board.danger_index() {
                        // handle Game Over.
                        let win_team = if team_num == Team::TeamOne {
                            Team::TeamTwo
                        } else {
                            Team::TeamOne
                        };
                        move_result = MoveResult::Win(win_team, WinReason::OpponentDangerDraw);
                    } else if self.board.is_grey_index(tile_id.into()) {
                        // handle grey tile
                        if team_num == Team::TeamOne {
                            self.next_turn = Some(Team::TeamTwo);
                        } else if team_num == Team::TeamTwo {
                            self.next_turn = Some(Team::TeamOne);
                        } else {
                            panic!("unreachable code reached! No team to update for grey tile.");
                        }
                    } else if self.board.is_team_one_index(tile_id.into()) {
                        if team_num == Team::TeamTwo {
                            // team one guessed wrong.
                            self.next_turn = Some(Team::TeamTwo);
                        }
                    } else if self.board.is_team_two_index(tile_id.into()) {
                        if team_num == Team::TeamOne {
                            // team two guessed wrong.
                            self.next_turn = Some(Team::TeamTwo);
                        }
                    } else {
                        // This should be unreachable but covers future changes.
                        return Err(InvalidMoveError::new(
                            format!("Couldn't update score for unravelling: {}\n", tile_id)
                                .as_ref(),
                        ));
                    }
                }
                Err(e) => {
                    return Err(InvalidMoveError::new(format!("{:?}", e).as_ref()));
                }
            }

            self.team_one_score = self.board.get_team_one_pending_size().try_into().unwrap();
            self.team_two_score = self.board.get_team_two_pending_size().try_into().unwrap();

            if self.team_one_score == TARGET_SCORE {
                move_result = MoveResult::Win(Team::TeamOne, WinReason::ScoreReached);
            }

            if self.team_two_score == TARGET_SCORE {
                move_result = MoveResult::Win(Team::TeamTwo, WinReason::ScoreReached);
            }
            return Ok(move_result);
        }
        return Err(InvalidMoveError::new("Not the current team's turn"));
    }

    pub fn get_in_progress_full_game_info(
        &self,
        player: &P,
    ) -> Result<FullGameInfoView<InProgressGame>, InvalidError> {
        return self.get_full_game_info(&player);
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
    fn game_move_team_tracking() -> Result<(), InvalidError> {
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
    fn game_move_danger_open() -> Result<(), InvalidError> {
        let game = setup_valid_game()?;
        let mut game = game.begin()?;
        assert_eq!(*(game.next_turn.as_ref().unwrap()), Team::TeamOne);
        // maybe player changed their name.
        let p1 = SimplePlayer::new("p-whatever", 1);
        assert_eq!(game.get_player_team(&p1).unwrap(), Team::TeamOne);
        let res = game.try_unravel(&p1, game.board.danger_index());
        assert!(res.is_ok());
        assert_eq!(
            res.unwrap(),
            MoveResult::Win(Team::TeamTwo, WinReason::OpponentDangerDraw)
        );
        Ok(())
    }

    #[test]
    fn game_move_grey_open() -> Result<(), InvalidError> {
        let game = setup_valid_game()?;
        let mut game = game.begin()?;
        assert_eq!(*(game.next_turn.as_ref().unwrap()), Team::TeamOne);
        // maybe player changed their name.
        let p1 = SimplePlayer::new("p-whatever", 1);
        assert_eq!(game.get_player_team(&p1).unwrap(), Team::TeamOne);
        let grey_moves: Vec<usize> = (0..game.board.words().len())
            .filter(|x| game.board.is_grey_index(*x))
            .collect();
        let res = game.try_unravel(&p1, grey_moves[0] as u8);
        assert!(res.is_ok());
        assert_eq!(game.next_turn.unwrap(), Team::TeamTwo);
        Ok(())
    }

    #[test]
    fn game_move_correct_incorrect_open() -> Result<(), InvalidError> {
        let game = setup_valid_game()?;
        let mut game = game.begin()?;
        assert_eq!(*(game.next_turn.as_ref().unwrap()), Team::TeamOne);
        // maybe player changed their name.
        let p1 = SimplePlayer::new("p-whatever", 1);
        assert_eq!(game.get_player_team(&p1).unwrap(), Team::TeamOne);
        let t1_safe_moves: Vec<usize> = (0..game.board.words().len())
            .filter(|x| game.board.is_team_one_index(*x))
            .collect();
        let t2_safe_moves: Vec<usize> = (0..game.board.words().len())
            .filter(|x| game.board.is_team_two_index(*x))
            .collect();
        // open team-1 slot. Still team-1 turn. Team-1 score gets closer to target by 1.
        let old_t1_score = game.get_team_one_score();
        let old_t2_score = game.get_team_two_score();
        let res = game.try_unravel(&p1, t1_safe_moves[0].try_into().unwrap());
        assert!(res.is_ok());
        assert_eq!(game.next_turn.as_ref().unwrap(), &Team::TeamOne);
        assert_eq!(old_t1_score - 1, game.get_team_one_score());
        assert_eq!(old_t2_score, game.get_team_two_score());
        // open team-2 slot. Then team-2 turn. Team-2 score gets closer to target by 1.
        let old_t1_score = game.get_team_one_score();
        let old_t2_score = game.get_team_two_score();
        let res = game.try_unravel(&p1, t2_safe_moves[0].try_into().unwrap());
        assert!(res.is_ok());
        assert_eq!(game.next_turn.as_ref().unwrap(), &Team::TeamTwo);
        assert_eq!(old_t1_score, game.get_team_one_score());
        assert_eq!(old_t2_score - 1, game.get_team_two_score());
        Ok(())
    }

    #[test]
    fn game_winning() -> Result<(), InvalidError> {
        let game = setup_valid_game()?;
        let mut game = game.begin()?;
        assert_eq!(*(game.next_turn.as_ref().unwrap()), Team::TeamOne);
        // maybe player changed their name.
        let p1 = SimplePlayer::new("p-whatever", 1);
        assert_eq!(game.get_player_team(&p1).unwrap(), Team::TeamOne);
        let t1_safe_moves: Vec<usize> = (0..game.board.words().len())
            .filter(|x| game.board.is_team_one_index(*x))
            .collect();
        let mut res = Ok(MoveResult::Continue);
        for (i, safe_move) in t1_safe_moves.iter().enumerate() {
            res = game.try_unravel(&p1, *safe_move as u8);
            assert!(res.is_ok());
            if i + 1 < t1_safe_moves.len() {
                assert_eq!(res.as_ref().unwrap(), &MoveResult::Continue);
            }
            assert_eq!(game.next_turn.as_ref().unwrap(), &Team::TeamOne);
        }
        assert_eq!(0, game.get_team_one_score());
        assert_eq!(
            res.unwrap(),
            MoveResult::Win(Team::TeamOne, WinReason::ScoreReached)
        );
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
