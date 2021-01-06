use futures::Stream;
use futures::task::Poll;
use std::pin::Pin;
use crate::web::errors::ForwardingError;
use tokio::sync::{Mutex, mpsc::{self, Receiver, Sender, error::{TryRecvError}}};
use futures::stream::SplitStream;
use futures::stream::StreamExt;
use std::fmt::Debug;
use std::sync::Arc;
use warp::ws::Message;
use warp::ws::WebSocket;
use serde::{Serialize, Deserialize};
use crate::players::PlayerId;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChallengeResponse{
    response: String,
    pid: PlayerId,
}


// pub trait ChallengeResolver {
//     fn reslove_challenge(&mut self, msg: &ChallengeResponse) -> Result<PlayerId, InvalidError>;
// }

/// websocket msg from a player. (player id, ws message.)
type WebSocketStreamItem = Result<Message, warp::Error>;
type PlayerWebSocketMsg = (Option<String>, WebSocketStreamItem);

pub struct PlayerWebSocketConnection {
    producer: Option<Arc<Mutex<Sender<WebSocketStreamItem>>>>,
    player_listener: Option<Arc<Mutex<SplitStream<WebSocket>>>>,
    fwd_pipe: Option<Sender<PlayerWebSocketMsg>>,
    pid: Option<String>,
    consumer: Option<Receiver<WebSocketStreamItem>>,
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

pub async fn forwarder(mut ws: SplitStream<WebSocket>, mut fd: tokio::sync::mpsc::Sender<WebSocketStreamItem>) -> Result<(), ForwardingError>{
    loop{
        let msg = ws.next().await;
        match msg {
            Some(ws_stream_item) => {
                fd.send(ws_stream_item).await.map_err(|e| {
                    let msg = format!("Error in send from ws to pipe: {:?}", e);
                    return ForwardingError::new(&msg);
                })?;
            },
            None => break,
        }

    }
    Ok(())
}

impl PlayerWebSocketConnection {
    pub fn new(ws: Option<WebSocket>, rcv_fwd: Option<Sender<PlayerWebSocketMsg>>) -> Self {
        let mut pwsc = PlayerWebSocketConnection {
            // sock: None,
            producer: None,
            player_listener: None,
            fwd_pipe: rcv_fwd,
            pid: None,
            consumer: None,
        };
        if ws.is_some() {
            pwsc.set_websocket(ws.unwrap());
        }
        return pwsc;
    }

    pub fn set_websocket(&mut self, ws: WebSocket) -> () {
        let (client_ws_sender, client_ws_rcv) = ws.split();
        let (tx, rx) = mpsc::channel(1024);
        tokio::task::spawn(rx.forward(client_ws_sender));
        self.producer = Some(Arc::new(Mutex::new(tx)));
        let (tx, rx) = mpsc::channel(1024);
        // tokio::task::spawn(client_ws_rcv.forward(tx.map_err(|e| {
        //         return warp::Error { inner: Box::new(e) };
        //     })));
        tokio::task::spawn(forwarder(client_ws_rcv, tx));
        self.player_listener = None; // Some(Arc::new(Mutex::new(client_ws_rcv)));
        self.consumer = Some(rx);

    }

    // pub async fn start_listening(&mut self) {
        
    // }

    // TODO: Plan
    // ws -> msgToPlayerMsgConverter -> publish to PlayerModelMpscProducer
    // handle disconnection.
}

// impl Stream for PlayerWebSocketConnection{

//     type Item = PlayerWebSocketMsg;
//     fn poll_next(self: Pin<&mut Self>, _: &mut std::task::Context<'_>) -> Poll<std::option::Option<<Self as Stream>::Item>> {
//         if self.consumer.is_some(){
//             match self.consumer.as_ref().unwrap().try_recv(){
//                 Ok(msg) => {
//                     return Poll::Ready(Some((self.pid, msg)));
//                 },
//                 Err(e) => {
//                     match e {
//                         TryRecvError::Empty => {
//                             // TODO: Spawn hanging read.
//                             return Poll::Pending;
//                         },
//                         TryRecvError::Closed => {
//                             self.consumer.unwrap().close();
//                             self.consumer = None;
//                             return Poll::Ready(None)
//                         }
//                     }
//                 }
//             }
//         }
//         return Poll::Ready(None);
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;

    use warp::Filter;

    #[tokio::test]
    async fn test_connection() {
        let route = warp::ws().map(|ws: warp::ws::Ws| {
            ws.on_upgrade(|websocket| async {
                let pwsc = PlayerWebSocketConnection::new(Some(websocket), None);
                let mut rcvr = pwsc.consumer.unwrap();
                let msg = (rcvr).next().await.unwrap().unwrap();
                assert_eq!(Ok("hello"), msg.to_str());
                let mut sender = pwsc.producer.as_ref().unwrap().lock().await;
                (*sender)
                    .send(Ok(Message::text("world")))
                    .await
                    .unwrap();
            })
        });

        let mut client = warp::test::ws().handshake(route).await.expect("handshake");
        client.send_text("hello").await;
        let msg = client.recv().await;
        assert_eq!("world", msg.unwrap().to_str().unwrap());
    }
}
