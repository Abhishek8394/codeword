use std::collections::HashMap;
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tokio::sync::{
    RwLock,
};

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
    pub fn new(name: &str, id: u32) -> Self {
        WebAppPlayer {
            player: SimplePlayer::new(name, id),
            conn: None,
        }
    }

    pub fn set_conn(&mut self, pwsc: PlayerWebSocketConnection) {
        self.conn = Some(Arc::new(pwsc));
    }
}

impl From<SimplePlayer> for WebAppPlayer{
    fn from(simple_player: SimplePlayer) -> Self {
        Self{
            player: simple_player,
            conn: None
        }
    }
}

impl OnlinePlayer for WebAppPlayer {}

/// This is a one point communication media for all players.
/// Messages from all players are available as a single queue.WebAppPlayer
/// TODO: Ability to broadcast messages to players
/// TODO: Ability to send message to a specific player.
pub struct PlayerModem {
    player_map: Arc<RwLock<HashMap<String, Arc<RwLock<WebAppPlayer>>>>>,
    ws_map: Arc<RwLock<HashMap<String, PlayerWebSocketConnection>>>,
    ws_player_map: Arc<RwLock<HashMap<String, String>>>,
}

impl PlayerModem {
    pub fn new() -> Self {
        Self {
            player_map: Arc::new(RwLock::new(HashMap::new())),
            ws_map: Arc::new(RwLock::new(HashMap::new())),
            ws_player_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_player(&self, player: WebAppPlayer) {
        let mut r1 = self.player_map.write().await;
        (*r1).insert(player.get_id().to_string(), Arc::new(RwLock::new(player)));
    }

    pub async fn add_orphan_conn(&self, player_conn: PlayerWebSocketConnection) {
        let mut writer = self.ws_map.write().await;
        (*writer).insert(player_conn.get_id().to_string(), player_conn);
    }

    pub async fn relate_player_ws_conn(&self, ws_id: &str, pid: &str) {
        let mut pwsc: Option<PlayerWebSocketConnection> = None;
        {
            let mut writer = self.ws_map.write().await;
            if let Some(conn) = (*writer).remove(ws_id){
                pwsc = Some(conn);
            }
        }
        if pwsc.is_none() {
            return;
        }
        {
            let mut writer = self.ws_player_map.write().await;
            (*writer).insert(ws_id.to_string(), pid.to_string());
        }
        {
            let reader = self.player_map.read().await;
            if let Some(player) = (*reader).get(pid){
                let mut player_writer = player.write().await;
                (*player_writer).set_conn(pwsc.unwrap());
            }
        }
    }

    pub async fn get_ws_player_id(&self, ws_id: &str) -> Option<String>{
        let reader = self.ws_player_map.read().await;
        if let Some(pid) =  (*reader).get(ws_id){
            return Some(pid.clone());
        }
        return None;
    }

    pub async fn get_num_players(&self) -> usize {
        let reader = self.player_map.read().await;
        return (*reader).len();
    }

    pub async fn get_num_orphan_conns(&self) -> usize {
        let reader = self.ws_map.read().await;
        return (*reader).len();
    }
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
