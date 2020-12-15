use serde::{Serialize, Deserialize};

use crate::players::SimplePlayer;
use serde::de::DeserializeOwned;
use crate::players::Player;
use warp::ws::WebSocket;


pub trait OnlinePlayer: Player + Clone + DeserializeOwned {}

#[derive(Serialize, Deserialize)]
pub struct WebAppPlayer{
    player: SimplePlayer,
    #[serde(skip)]
    rx: Option<WebSocket>,
    #[serde(skip)]
    wx: Option<WebSocket>,
}

impl Player for WebAppPlayer{

    fn get_name(&self) -> &str{
        self.player.get_name()
    }

    fn get_id(&self) -> &u32{
        self.player.get_id()
    }
}

impl WebAppPlayer{
    fn new(name: &str, id: u32) -> Self{
        WebAppPlayer{
            player: SimplePlayer::new(name, id),
            rx: None,
            wx: None,
        }
    }
}

mod test{
    use super::WebAppPlayer;
    use super::Player;

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

