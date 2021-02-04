// use serde::{Serialize, Deserialize};
use crate::players::SimplePlayer;
use tokio::sync::RwLock;
use super::players::WebAppPlayer;
use crate::players::PlayerId;
use crate::web::auth::{AuthChallenge, InternalAuthChallenge, build_echo_challenge};
use std::collections::HashMap;
use std::sync::Arc;
use crate::web::players::PlayerModem;
use crate::errors::InvalidError;
use crate::game::{Game, MoveResult};
use crate::game::InProgressGame;
use crate::game::InitialGame;
use crate::web::ws::PlayerWebSocketConnection;
use crate::web::ws::PlayerWebSocketMsg;
use tokio::sync::mpsc::{self, Receiver, Sender};
use uuid::Uuid;
use crate::players::Player;
use crate::web::wsproto::{AuthResponse, WSMessage};

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
        match self{
            Self::InitialGame(game) => {
                let tmp = game.clone();
                if let Ok(game) = tmp.begin(){
                    Self::InProgressGame(game)
                }
                else{
                    self.clone()
                }
            },
            g => {g.clone()},
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
        match pwsc{
            Ok(pwsc) => {self.player_modem.add_orphan_conn(pwsc).await;},
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
            },
            Err(e) => {
                eprintln!("[{}] error closing ws (probably closed from user side): {:?}, {:?}", self.id, id, e);
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
    pub async fn handle_auth_resp(&self, ws_id: &str,  msg: AuthResponse){
        let pid = msg.pid;
        let mut ok = false;
        let server_resp;
        {
            let mut writer = self.auth_challenges.write().await;
            if let Some(challenge) = (*writer).get(&pid) {
                if challenge.response_matches(&msg){
                    ok = true;
                    (*writer).remove(&pid);
                }
            }
        }
        if ok {
            self.player_modem.relate_player_ws_conn(ws_id, pid.to_string().as_str()).await;
            server_resp = WSMessage::AuthOk;
            let _ = self.player_modem.ws_send_msg(ws_id, server_resp.into()).await;
        }
        else{
            server_resp = WSMessage::AuthReject;
            let _ = self.player_modem.ws_send_msg(ws_id, server_resp.into()).await;
            let _ = self.player_modem.close_ws(ws_id).await;
        }
    }

    pub async fn handle_tile_select_msg(&mut self, ws_id: &str, tile_num: u8) {
        if let Some(pid) = self.player_modem.get_ws_player_id(ws_id).await{
            if let Some(player) = self.player_modem.get_simple_player(&pid).await{
                if self.game.is_initial() {
                    self.game = self.game.begin();
                }
                match &mut self.game{
                    GameWrapper::InitialGame(_game) => {
                        let msg = WSMessage::InvalidMove{reason: Some("Game not ready to start yet".to_string())};
                        let send_res  = self.player_modem.send_player_msg(&pid, msg.into()).await;
                        if send_res.is_err(){
                            eprintln!("[{}] error sending response to ({}): {:?}", self.id, pid, send_res);
                        }
                    },
                    GameWrapper::InProgressGame(game) => {
                        let move_result = game.try_unravel(&player, tile_num);
                        match move_result{
                            Ok(move_result) => {
                                // TODO: Send confirmation to player
                                // Update internal state
                                self.move_update_id += 1;
                                // Handle result states
                                match move_result{
                                    MoveResult::Win(ref team, ref reason) => {
                                        let msg = WSMessage::TeamWinMessage{
                                            id: team.get_id(),
                                            reason: reason.to_string()
                                        };
                                        self.player_modem.broadcast(msg.into()).await;
                                        self.quit().await;
                                    },
                                    MoveResult::Continue => {
                                        // Game continues
                                        // Publish updates.
                                        let msg = WSMessage::UpdateState(self.move_update_id);
                                        self.player_modem.broadcast(msg.into()).await;
                                    }
                                }
                                println!("[{}] move result: {:?}", self.id, move_result);
                            },
                            Err(e) => {
                                eprintln!("[{}] move error: {:?}", self.id, e);
                                let msg = WSMessage::InvalidMove{reason: None};
                                let send_res  = self.player_modem.send_player_msg(&pid, msg.into()).await;
                                if send_res.is_err(){
                                    eprintln!("[{}] error sending response to ({}): {:?}", self.id, pid, send_res);
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
