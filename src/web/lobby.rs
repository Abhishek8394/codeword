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
    ws_link_consumer: Receiver<PlayerWebSocketMsg>,
    ws_link_producer: Sender<PlayerWebSocketMsg>,
    unk_ws: HashMap<String, PlayerWebSocketConnection>,
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
            ws_link_consumer: rx,
            ws_link_producer: tx,
            unk_ws: HashMap::new(),
        }
    }

    pub fn add_player_id(&mut self, pid: &str) {
        self.player_ids.insert(String::from(pid));
    }

    pub fn get_num_players(&self) -> usize {
        self.player_ids.len()
    }

    pub fn handle_incoming_ws(&mut self, websocket: warp::ws::WebSocket) -> () {
        let ws_id = Uuid::new_v4().to_string();
        let pwsc = PlayerWebSocketConnection::new(
            &ws_id,
            Some(websocket),
            Some(self.ws_link_producer.clone()),
        );
        self.unk_ws.insert(ws_id, pwsc);
    }

    pub async fn start_ws_listening(&mut self) {
        loop {
            let pws_msg = self.ws_link_consumer.recv().await;
            match pws_msg {
                Some(pws_msg) => {
                    if let (uniq_id, Ok(msg)) = pws_msg {
                        println!("[{}] Got: {:?}", uniq_id, msg);
                        // TODO:
                        // - match uniq id
                        // - handle auth msg
                        // - handle game msg
                    }
                }
                // everyone has disconnected, drop out and delete lobby maybe?
                // If don't want to drop, then remove break.
                None => break,
            }
        }
    }

    pub fn get_num_unidentified_ws(&self) -> usize {
        self.unk_ws.len()
    }

    // pub async fn startup(self){
    //     tokio::task::spawn(self.clone().start_ws_listening());
    // }
}
