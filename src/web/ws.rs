use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::stream::SplitStream;
use futures::stream::StreamExt;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc::Sender};
use warp::ws::Message;
use warp::ws::WebSocket;

/// websocket msg from a player. (player id, ws message.)
type PlayerWebSocketMsg = (Option<String>, Message);
type WebSocketStreamItem = Result<Message, warp::Error>;

pub struct PlayerWebSocketConnection {
    producer: Option<Arc<Mutex<UnboundedSender<WebSocketStreamItem>>>>,
    player_listener: Option<Arc<Mutex<SplitStream<WebSocket>>>>,
}

impl Debug for PlayerWebSocketConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let prod_str = if self.producer.is_none() {
            "none"
        } else {
            "<CHANNEL SET>"
        };
        let player_listener_str = if self.player_listener.is_none() {
            "none"
        } else {
            "<CHANNEL SET>"
        };
        let msg = format!(
            "PlayerWebSocketConnection{{ producer: {:?}, player_listener: {:?} }}",
            prod_str, player_listener_str
        );
        f.write_str(&msg)
    }
}

impl PlayerWebSocketConnection {
    pub fn new(ws: Option<WebSocket>, _rcv_fwd: Option<Sender<PlayerWebSocketMsg>>) -> Self {
        let mut pwsc = PlayerWebSocketConnection {
            // sock: None,
            producer: None,
            player_listener: None,
        };
        if ws.is_some() {
            pwsc.set_websocket(ws.unwrap());
        }
        return pwsc;
    }

    pub fn set_websocket(&mut self, ws: WebSocket) -> () {
        let (client_ws_sender, client_ws_rcv) = ws.split();
        let (tx, rx) = unbounded();
        tokio::task::spawn(rx.forward(client_ws_sender));
        self.producer = Some(Arc::new(Mutex::new(tx)));
        self.player_listener = Some(Arc::new(Mutex::new(client_ws_rcv)));

    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use warp::Filter;

    #[tokio::test]
    async fn test_connection() {
        let route = warp::ws().map(|ws: warp::ws::Ws| {
            ws.on_upgrade(|websocket| async {
                let pwsc = PlayerWebSocketConnection::new(Some(websocket), None);
                let mut rcvr = pwsc.player_listener.as_ref().unwrap().lock().await;
                let msg = (*rcvr).next().await.unwrap().unwrap();
                assert_eq!(Ok("hello"), msg.to_str());
                let sender = pwsc.producer.as_ref().unwrap().lock().await;
                (*sender)
                    .unbounded_send(Ok(Message::text("world")))
                    .unwrap();
            })
        });

        let mut client = warp::test::ws().handshake(route).await.expect("handshake");
        client.send_text("hello").await;
        let msg = client.recv().await;
        assert_eq!("world", msg.unwrap().to_str().unwrap());
    }
}
