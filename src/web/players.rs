use std::collections::HashMap;
use std::sync::{Arc};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc::{Sender, Receiver}};
use warp::ws::Message;

use crate::players::Player;
use crate::players::SimplePlayer;
use crate::web::ws::PlayerWebSocketConnection;

/// Constraints for players over a remote conn.
pub trait OnlinePlayer: Player + Clone + DeserializeOwned {}

/// A connection to player on web
pub type PlayerConnection = Arc<PlayerWebSocketConnection>;

/// A player in context of an web app
#[derive(Serialize, Deserialize, Clone)]
pub struct WebAppPlayer {
    /// Player info
    player: SimplePlayer,

    /// Player channel connection
    #[serde(skip)]
    conn: Option<PlayerConnection>,
}

impl Player for WebAppPlayer {
    fn get_name(&self) -> &str {
        self.player.get_name()
    }

    fn get_id(&self) -> &u32 {
        self.player.get_id()
    }
}

impl WebAppPlayer {
    fn new(name: &str, id: u32) -> Self {
        WebAppPlayer {
            player: SimplePlayer::new(name, id),
            conn: None,
        }
    }
}

impl OnlinePlayer for WebAppPlayer {}

/// This is a one point communication media for all players.
/// Messages from all players are available as a single queue.WebAppPlayer
/// TODO: Ability to broadcast messages to players
/// TODO: Ability to send message to a specific player.
pub struct PlayerModem{
    player_map: Arc<RwLock<HashMap<String, WebAppPlayer>>>,
    player_msgq_consumer: Option<Receiver<Message>>,
    player_reader: Option<Sender<Message>>,
}

impl PlayerModem{
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        Self {
            player_map: Arc::new(RwLock::new(HashMap::new())),
            player_msgq_consumer: Some(rx),
            player_reader: Some(tx),
        }
    }

    // pub async fn add_player(&mut self, player: WebAppPlayer) {
    //     let mut r1 = self.player_map.write().await;
    //     (*r1).insert(player.get_id().to_string(), player);
    //     if player.conn.is_some() {
    //         tokio::task::spawn(player.conn.as_ref().unwrap()
    //             .forward(self.player_reader.as_mut_ref().unwrap())
    //         );
    //     }
    // }

    // pub async fn add_player_conn(&mut self, player_conn: PlayerWebSocketConnection) {

    // }
}

mod test {
    use super::Player;
    use super::WebAppPlayer;

    #[test]
    fn player_deser_test() {
        let player = WebAppPlayer::new("player 1", 1001);
        let json_player = serde_json::to_string(&player).unwrap();
        let deser_player: WebAppPlayer = serde_json::from_str(&json_player).unwrap();
        assert_eq!(player.get_id(), deser_player.get_id());
        assert_eq!(player.get_name(), deser_player.get_name());
        assert!(deser_player.conn.is_none());
    }
}

