use crate::players::PlayerId;
use warp::ws::Message;
use crate::web::errors::WebSocketError;
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
pub type PlayerConnection = Arc<RwLock<PlayerWebSocketConnection>>;

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

    fn get_id(&self) -> &PlayerId {
        self.player.get_id()
    }

}

impl WebAppPlayer {
    pub fn new(name: &str, id: PlayerId) -> Self {
        WebAppPlayer {
            player: SimplePlayer::new(name, id),
            conn: None,
        }
    }

    pub fn set_conn(&mut self, pwsc: PlayerWebSocketConnection) {
        self.conn = Some(Arc::new(RwLock::new(pwsc)));
    }

    pub async fn close_ws(&mut self) -> Result<(), WebSocketError> {
        if self.conn.is_none(){
            return Ok(());
        }
        let res;
        {
            let mut writer = self.conn.as_ref().unwrap().write().await;
            res = (*writer).close().await;
        }
        self.conn = None;
        return res;
    }

    pub async fn send_msg(&mut self, msg: Message) -> Result<(), WebSocketError> {
        if self.conn.is_none(){
            let err_msg = format!("websocket not bound for player: {}", self.player.get_id());
            return Err(WebSocketError::WSNotFoundError(err_msg));
        }
        let reader = self.conn.as_ref().unwrap().read().await;
        return (*reader).send_msg(msg).await;
    }

    pub fn get_player(&self) -> &SimplePlayer {
        return &self.player;
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
    ws_map: Arc<RwLock<HashMap<String, PlayerConnection>>>,
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
        (*writer).insert(player_conn.get_id().to_string(), Arc::new(RwLock::new(player_conn)));
    }

    pub async fn relate_player_ws_conn(&self, ws_id: &str, pid: &str) {
        let mut pwsc: Option<PlayerWebSocketConnection> = None;
        {
            let mut writer = self.ws_map.write().await;
            if let Some(conn) = (*writer).remove(ws_id){
                if let Ok(pswc_lone) = Arc::try_unwrap(conn) {
                    pwsc = Some(pswc_lone.into_inner());
                }
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
                eprintln!("player {} connected via {}", pid, ws_id);
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

    /// Get web app player associated with given player id. Returned entry wrapped in `RwLock`
    pub async fn get_web_player(&self, pid: &str) -> Option<Arc<RwLock<WebAppPlayer>>> {
        let reader = self.player_map.read().await;
        if let Some(player) = (*reader).get(pid) {
            return Some(player.clone());
        }
        return None;
    }

    /// Get simple player associated with given player id
    pub async fn get_simple_player(&self, pid: &str) -> Option<SimplePlayer> {
        if let Some(web_player) = self.get_web_player(pid).await{
            let reader = web_player.read().await;
            return Some((*reader).get_player().clone());
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

    pub async fn remove_ws_player_mapping(&self, id: &str) {
        let mut writer = self.ws_player_map.write().await;
        (*writer).remove(id);
    }

    /// Close a websocket
    pub async fn close_ws(&self, id: &str) -> Result<(), WebSocketError> {
        // if an orphan conn.
        let mut pwsc = None;
        {
            let mut writer = self.ws_map.write().await;
            match (*writer).remove(id){
                Some(tmp) => {
                    pwsc = Some(tmp);
                },
                None => {},
            };
        }
        {
            if pwsc.is_some(){
                let pwsc = pwsc.unwrap();
                let mut writer = pwsc.write().await;
                let res = (*writer).close().await;
                return res;
            }
        }
        // if on a player
        {
            match self.get_ws_player_id(id).await{
                Some(pid) => {
                    {
                        let rdr = self.player_map.read().await;
                        match (*rdr).get(&pid){
                            Some(player_rw) => {
                                let mut player = player_rw.write().await;
                                let res = player.close_ws().await;
                                // if ws closed, remove mapping.
                                if res.is_ok(){
                                    self.remove_ws_player_mapping(id).await;
                                }
                                return res;
                            },
                            None => {},
                        };
                    }
                },
                None => {},
            }
        }
        return Ok(());
    }

    pub async fn send_player_msg(&self, pid: &str, msg: Message) -> Result<(), WebSocketError> {
        {
            let reader = self.player_map.read().await;
            if let Some(player) = (*reader).get(pid){
                let mut writer = player.write().await;
                return (*writer).send_msg(msg).await;
            }
        }
        let err_msg = format!("websocket not found for player: {}", pid);
        return Err(WebSocketError::WSNotFoundError(err_msg));
    }

    /// send message to a websocket.
    pub async fn ws_send_msg(&self, ws_id: &str, msg: Message) -> Result<(), WebSocketError> {

        {
            let reader = self.ws_map.read().await;
            if let Some(pwsc) = (*reader).get(ws_id).as_ref() {
                let reader = pwsc.read().await;
                return (*reader).send_msg(msg).await;
            }
        }
        {
            match self.get_ws_player_id(ws_id).await{
                Some(pid) => {
                    return self.send_player_msg(&pid, msg).await;
                },
                None => {}
            }
        }
        let err_msg = format!("websocket not found: {}", ws_id);
        return Err(WebSocketError::WSNotFoundError(err_msg));
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
