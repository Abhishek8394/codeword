// use serde::{Serialize, Deserialize};
use crate::web::players::PlayerModem;
use super::players::WebAppPlayer;
use crate::errors::InvalidError;
use crate::game::Game;
use crate::game::InProgressGame;
use crate::game::InitialGame;
use crate::web::ws::PlayerWebSocketConnection;
use crate::web::ws::PlayerWebSocketMsg;
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
    // player_ids: HashSet<String>,
    game: GameWrapper,
    ws_link_consumer: Option<Receiver<PlayerWebSocketMsg>>,
    ws_link_producer: Option<Sender<PlayerWebSocketMsg>>,
    allow_conns: bool,
    player_modem: PlayerModem,
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
            player_modem: PlayerModem::new()
        }
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub async fn add_player(&self, player: WebAppPlayer) {
        self.player_modem.add_player(player).await;
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
        self.player_modem.add_orphan_conn(pwsc).await;
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

    pub async fn quit(&mut self) {
        self.allow_conns = false;
        self.ws_link_producer = None;
    }
}
