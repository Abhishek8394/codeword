// use serde::{Serialize, Deserialize};
use tokio::sync::RwLock;
use super::players::WebAppPlayer;
use crate::players::PlayerId;
use crate::web::auth::{AuthChallenge, InternalAuthChallenge, build_echo_challenge};
use std::collections::HashMap;
use std::sync::Arc;
use crate::web::players::PlayerModem;
use crate::errors::InvalidError;
use crate::game::Game;
use crate::game::InProgressGame;
use crate::game::InitialGame;
use crate::web::ws::PlayerWebSocketConnection;
use crate::web::ws::PlayerWebSocketMsg;
use tokio::sync::mpsc::{self, Receiver, Sender};
use uuid::Uuid;
use crate::players::Player;
use crate::web::wsproto::{AuthResponse, WSMessage};

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
    // player_ids: HashSet<String>,
    game: GameWrapper,
    ws_link_consumer: Option<Receiver<PlayerWebSocketMsg>>,
    ws_link_producer: Option<Sender<PlayerWebSocketMsg>>,
    allow_conns: bool,
    player_modem: PlayerModem,
    auth_challenges: Arc<RwLock<HashMap<PlayerId, InternalAuthChallenge>>>,
}

impl Lobby {
    pub fn new(id: &str, _player_ids: &Vec<String>, game: GameWrapper) -> Self {
        let (tx, rx) = mpsc::channel(1024);
        Lobby {
            id: String::from(id),
            // player_ids: player_ids
            //     .iter()
            //     .map(|x| String::from(x))
            //     .collect::<HashSet<String>>(),
            game,
            ws_link_consumer: Some(rx),
            ws_link_producer: Some(tx),
            allow_conns: true,
            player_modem: PlayerModem::new(),
            auth_challenges: Arc::new(RwLock::new(HashMap::new()))
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
                eprintln!("[{}] error closing ws: {:?}, {:?}", self.id, id, e);
            }
        };
    }

    pub async fn quit(&mut self) {
        self.allow_conns = false;
        self.ws_link_producer = None;
    }

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
        // find challenge if found.
        // if passes, perform player mapping
        // else close this socket.
        // return response to socket.
    }
}
