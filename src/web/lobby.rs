// use serde::{Serialize, Deserialize};
use super::players::WebAppPlayer;
use crate::game::Team;
use crate::errors::InvalidError;
use crate::game::FullGameInfoView;

use crate::game::InProgressGame;
use crate::game::InitialGame;
use crate::game::{Game, MoveResult, WinResult};
use crate::players::Player;
use crate::players::PlayerId;
use crate::players::SimplePlayer;
use crate::web::auth::{build_echo_challenge, AuthChallenge, InternalAuthChallenge};
use crate::web::errors::NotAllowedError;
use crate::web::players::PlayerModem;
use crate::web::ws::PlayerWebSocketConnection;
use crate::web::ws::PlayerWebSocketMsg;
use crate::web::wsproto::{AuthResponse, WSMessage};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Serialize)]
pub enum GameViewWrapper {
    InitialFullGameView(FullGameInfoView<InitialGame>),
    InProgressFullGameView(FullGameInfoView<InProgressGame>),
    // InitialGameView(FullGameInfoView<InitialGame>),
}

#[derive(Clone)]
pub enum GameWrapper {
    InitialGame(Game<InitialGame, SimplePlayer>),
    InProgressGame(Game<InProgressGame, SimplePlayer>),
}

impl GameWrapper {
    pub fn new(words: &Vec<String>) -> Result<GameWrapper, InvalidError> {
        let g = Game::new(words)?;
        Ok(GameWrapper::InitialGame(g))
    }

    pub fn is_initial(&self) -> bool {
        match &self {
            &GameWrapper::InitialGame(_g) => true,
            &GameWrapper::InProgressGame(_g) => false,
        }
    }

    pub fn begin(&self) -> Self {
        match self {
            Self::InitialGame(game) => {
                let tmp = game.clone();
                match tmp.begin(){
                    Ok(game) => {
                        Self::InProgressGame(game)
                    },
                    Err(e) => {
                        Self::InitialGame(e.take_old())
                    }
                }
            }
            g => g.clone(),
        }
    }

    pub fn get_full_game_info(
        &self,
        player: &SimplePlayer,
    ) -> Result<GameViewWrapper, InvalidError> {
        match &self {
            &GameWrapper::InitialGame(g) => match g.get_initial_full_game_info(&player) {
                Ok(g) => Ok(GameViewWrapper::InitialFullGameView(g)),
                Err(e) => Err(e),
            },
            &GameWrapper::InProgressGame(g) => match g.get_in_progress_full_game_info(&player) {
                Ok(g) => Ok(GameViewWrapper::InProgressFullGameView(g)),
                Err(e) => Err(e),
            },
        }
    }

    pub fn get_player_team_from_id(&self, pid: &PlayerId) -> Option<Team> {
        match &self {
            &GameWrapper::InitialGame(g) => g.get_player_team_from_id(pid),
            &GameWrapper::InProgressGame(g) => g.get_player_team_from_id(pid),
        }
    }
    
    pub fn transfer_player(&self, pid: &PlayerId, team: &Team) -> (Self, Result<(), InvalidError>) {
        let tmp = self.clone();
        match tmp {
            GameWrapper::InitialGame(mut g) => {
                let res = g.transfer_player(pid, team);
                return (GameWrapper::InitialGame(g), res);
            },
            GameWrapper::InProgressGame(mut g) => {
                let res = g.transfer_player(pid, team);
                
                return (GameWrapper::InProgressGame(g), res);
            },
        }
    }

    pub fn add_player_to_team(&self, player: SimplePlayer, team: &Team) -> (Self, Result<(), InvalidError>) {
        let tmp = self.clone();
        match tmp {
            GameWrapper::InitialGame(mut g) => {
                let res = g.add_player_to_team(player, team);
                return (GameWrapper::InitialGame(g), res);
            },
            GameWrapper::InProgressGame(mut g) => {
                let res = g.add_player_to_team(player, team);
                return (GameWrapper::InProgressGame(g), res);
            },
        }
    }
}

// #[derive(Serialize, Deserialize)]
pub struct Lobby {
    pub id: String,
    game: GameWrapper,
    ws_link_consumer: Option<Receiver<PlayerWebSocketMsg>>,
    ws_link_producer: Option<Sender<PlayerWebSocketMsg>>,
    allow_conns: bool,
    player_modem: PlayerModem,
    auth_challenges: Arc<RwLock<HashMap<PlayerId, InternalAuthChallenge>>>,
    move_update_id: u32,
}

impl Lobby {
    pub fn new(id: &str, _player_ids: &Vec<String>, game: GameWrapper) -> Self {
        let (tx, rx) = mpsc::channel(1024);
        Lobby {
            id: String::from(id),
            // game: Arc::new(RwLock::new(game)),
            game,
            ws_link_consumer: Some(rx),
            ws_link_producer: Some(tx),
            allow_conns: true,
            player_modem: PlayerModem::new(),
            auth_challenges: Arc::new(RwLock::new(HashMap::new())),
            move_update_id: 0,
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub async fn add_player(&self, player: WebAppPlayer) -> AuthChallenge {
        let pid = player.get_id().clone();
        self.player_modem.add_player(player).await;
        let int_auth_challenge = build_echo_challenge(pid, None);
        let player_challenge = int_auth_challenge.get_player_challenge().clone();
        let mut writer = self.auth_challenges.write().await;
        (*writer).insert(pid, int_auth_challenge.clone());
        return player_challenge;
    }

    pub async fn switch_or_join_team(&mut self, pid: PlayerId, team: &Team) -> Result<(), InvalidError> {
        let tmp;
        let result;
        if self.game.get_player_team_from_id(&pid).is_some(){
            tmp = self.game.transfer_player(&pid, team);
            self.game = tmp.0;
            result = tmp.1;
        }
        else{
            let player = self.player_modem.get_simple_player(&pid.to_string()).await;
            if player.is_none(){
                result = Err(InvalidError::new("Invalid pid!"));
            }
            else{
                tmp = self.game.add_player_to_team(player.unwrap(), team);
                self.game = tmp.0;
                result = tmp.1;
            }
        }
        let msg = WSMessage::PlayerUpdate;
        self.player_modem.broadcast(msg.into()).await;
        return result;
    }

    pub async fn get_num_players(&self) -> usize {
        self.player_modem.get_num_players().await
    }

    pub async fn handle_incoming_ws(&mut self, websocket: warp::ws::WebSocket) -> () {
        if !self.allow_conns {
            let _ = websocket.close().await;
            return;
        }
        let ws_id = Uuid::new_v4().to_string();
        let pwsc =
            PlayerWebSocketConnection::new(&ws_id, Some(websocket), self.ws_link_producer.clone());
        match pwsc {
            Ok(pwsc) => {
                self.player_modem.add_orphan_conn(pwsc).await;
            }
            Err(e) => {
                eprintln!("[{}] Error setting up ws ({}): {:?}", self.id, ws_id, e);
            }
        }
    }

    pub async fn get_num_unidentified_ws(&self) -> usize {
        return self.player_modem.get_num_orphan_conns().await;
    }

    pub fn get_ws_receiver(&mut self) -> Result<Receiver<PlayerWebSocketMsg>, InvalidError> {
        match self.ws_link_consumer.take() {
            Some(rcvr) => Ok(rcvr),
            None => Err(InvalidError::new("Receiver already taken")),
        }
    }

    /// Close a websocket
    pub async fn close_ws(&self, id: &str) {
        match self.player_modem.close_ws(id).await {
            Ok(_) => {
                eprintln!("[{}] closed ws: {:?}", self.id, id);
            }
            Err(e) => {
                eprintln!(
                    "[{}] error closing ws (probably closed from user side): {:?}, {:?}",
                    self.id, id, e
                );
            }
        };
    }

    pub async fn quit(&mut self) {
        self.allow_conns = false;
        self.ws_link_producer = None;
    }

    /// handles auth response message on websocket.
    /// - find challenge if found.
    /// - if passes, perform player mapping
    /// - else close this socket.
    /// - return response to socket.
    pub async fn handle_auth_resp(&self, ws_id: &str, msg: AuthResponse) {
        let pid = msg.pid;
        let mut ok = false;
        let server_resp;
        {
            let mut writer = self.auth_challenges.write().await;
            if let Some(challenge) = (*writer).get(&pid) {
                if challenge.response_matches(&msg) {
                    ok = true;
                    (*writer).remove(&pid);
                }
            }
        }
        if ok {
            self.player_modem
                .relate_player_ws_conn(ws_id, pid.to_string().as_str())
                .await;
            server_resp = WSMessage::AuthOk;
            let _ = self
                .player_modem
                .ws_send_msg(ws_id, server_resp.into())
                .await;
        } else {
            server_resp = WSMessage::AuthReject;
            let _ = self
                .player_modem
                .ws_send_msg(ws_id, server_resp.into())
                .await;
            let _ = self.player_modem.close_ws(ws_id).await;
        }
    }

    /// Get game view based on player id.0
    pub async fn get_player_full_game_view(
        &self,
        pid: &str,
    ) -> Result<GameViewWrapper, NotAllowedError> {
        if let Some(player) = self.player_modem.get_simple_player(&pid).await {
            let res = self.game.get_full_game_info(&player);
            if res.is_ok() {
                return Ok(res.unwrap());
            }
        }
        return Err(NotAllowedError::new("Not allowed"));
    }

    pub async fn handle_tile_select_msg(&mut self, ws_id: &str, tile_num: u8) {
        if let Some(pid) = self.player_modem.get_ws_player_id(ws_id).await {
            if let Some(player) = self.player_modem.get_simple_player(&pid).await {
                if self.game.is_initial() {
                    self.game = self.game.begin();
                }
                match &mut self.game {
                    GameWrapper::InitialGame(_game) => {
                        let msg = WSMessage::InvalidMove {
                            reason: Some("Game not ready to start yet".to_string()),
                        };
                        let send_res = self.player_modem.send_player_msg(&pid, msg.into()).await;
                        if send_res.is_err() {
                            eprintln!(
                                "[{}] error sending response to ({}): {:?}",
                                self.id, pid, send_res
                            );
                        }
                    }
                    GameWrapper::InProgressGame(game) => {
                        let move_result = game.try_unravel(&player, tile_num);
                        match move_result {
                            Ok(move_result) => {
                                // TODO: Send confirmation to player
                                // Update internal state
                                self.move_update_id += 1;
                                // Handle result states
                                match move_result {
                                    MoveResult::Win(WinResult{ref team, ref reason}) => {
                                        let msg = WSMessage::TeamWinMessage {
                                            id: team.get_id(),
                                            reason: reason.to_string(),
                                        };
                                        self.player_modem.broadcast(msg.into()).await;
                                        self.quit().await;
                                    }
                                    MoveResult::Continue => {
                                        // Game continues
                                        // Publish updates.
                                        let msg = WSMessage::UpdateState(self.move_update_id);
                                        self.player_modem.broadcast(msg.into()).await;
                                    }
                                }
                                println!("[{}] move result: {:?}", self.id, move_result);
                            }
                            Err(e) => {
                                eprintln!("[{}] move error: {:?}", self.id, e);
                                let msg = WSMessage::InvalidMove { reason: None };
                                let send_res =
                                    self.player_modem.send_player_msg(&pid, msg.into()).await;
                                if send_res.is_err() {
                                    eprintln!(
                                        "[{}] error sending response to ({}): {:?}",
                                        self.id, pid, send_res
                                    );
                                }
                            }
                        }
                    }
                }
            };
        };
    }

    // TODO: handle game moves + broadcast update.
}
