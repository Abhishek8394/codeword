use crate::players::Player;
use crate::players::SimplePlayer;
use crate::web::ws::PlayerWebSocketConnection;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// Constraints for players over a remote conn.
pub trait OnlinePlayer: Player + Clone + DeserializeOwned {}

/// A connection to player on web
pub type PlayerConnection = Arc<RwLock<PlayerWebSocketConnection>>;

/// A player in context of an web app
#[derive(Serialize, Deserialize, Clone)]
pub struct WebAppPlayer {
    /// Player info
    player: SimplePlayer,

    /// Player channel read connection
    #[serde(skip)]
    rx: Option<PlayerConnection>,

    /// Player channel write connection
    #[serde(skip)]
    wx: Option<PlayerConnection>,
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
            rx: None,
            wx: None,
        }
    }
}

impl OnlinePlayer for WebAppPlayer {}

mod test {
    use super::Player;
    use super::WebAppPlayer;

    #[test]
    fn player_ser_deser_test() {
        let player = WebAppPlayer::new("player 1", 1001);
        let json_player = serde_json::to_string(&player).unwrap();
        let deser_player: WebAppPlayer = serde_json::from_str(&json_player).unwrap();
        assert_eq!(player.get_id(), deser_player.get_id());
        assert_eq!(player.get_name(), deser_player.get_name());
        assert!(deser_player.rx.is_none());
        assert!(deser_player.wx.is_none());
    }
}
