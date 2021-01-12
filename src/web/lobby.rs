// use serde::{Serialize, Deserialize};
use super::players::WebAppPlayer;
use crate::errors::InvalidError;
use crate::game::Game;
use crate::game::InProgressGame;
use crate::game::InitialGame;
use crate::web::ws::PlayerWebSocketConnection;
use crate::web::ws::PlayerWebSocketMsg;
use std::collections::HashMap;
use std::collections::HashSet;
use tokio::sync::mpsc::{self, Receiver, Sender};
use uuid::Uuid;

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
    player_ids: HashSet<String>,
    game: GameWrapper,
    ws_link_consumer: Option<Receiver<PlayerWebSocketMsg>>,
    ws_link_producer: Option<Sender<PlayerWebSocketMsg>>,
    unk_ws: HashMap<String, PlayerWebSocketConnection>,
    allow_conns: bool,
}

impl Lobby {
    pub fn new(id: &str, player_ids: &Vec<String>, game: GameWrapper) -> Self {
        let (tx, rx) = mpsc::channel(1024);
        Lobby {
            id: String::from(id),
            player_ids: player_ids
                .iter()
                .map(|x| String::from(x))
                .collect::<HashSet<String>>(),
            game,
            ws_link_consumer: Some(rx),
            ws_link_producer: Some(tx),
            unk_ws: HashMap::new(),
            allow_conns: true,
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn add_player_id(&mut self, pid: &str) {
        self.player_ids.insert(String::from(pid));
    }

    pub fn get_num_players(&self) -> usize {
        self.player_ids.len()
    }

    pub async fn handle_incoming_ws(&mut self, websocket: warp::ws::WebSocket) -> () {
        if !self.allow_conns {
            let _ = websocket.close().await;
            return;
        }
        let ws_id = Uuid::new_v4().to_string();
        let pwsc =
            PlayerWebSocketConnection::new(&ws_id, Some(websocket), self.ws_link_producer.clone());
        self.unk_ws.insert(ws_id, pwsc);
    }

    pub fn get_num_unidentified_ws(&self) -> usize {
        self.unk_ws.len()
    }

    pub fn get_ws_receiver(&mut self) -> Result<Receiver<PlayerWebSocketMsg>, InvalidError> {
        match self.ws_link_consumer.take() {
            Some(rcvr) => Ok(rcvr),
            None => Err(InvalidError::new("Receiver already taken")),
        }
    }

    pub async fn quit(mut self) {
        self.allow_conns = false;
        self.ws_link_producer = None;
        drop(self);
        // TODO: Disconnect everyone.
        todo!()
    }
}
