


use crate::errors::InvalidError;
use crate::web::errors::ForwardingError;
use tokio::sync::{Mutex, mpsc::{self, Sender}};
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

/// websocket msg from a player. (websocket id, ws message.)
type WebSocketStreamItem = Result<Message, warp::Error>;
type PlayerWebSocketMsg = (String, WebSocketStreamItem);

pub struct PlayerWebSocketConnection {
    producer: Option<Arc<Mutex<Sender<WebSocketStreamItem>>>>,
    player_listener: Option<Arc<Mutex<SplitStream<WebSocket>>>>,
    fwd_pipe: Option<Sender<PlayerWebSocketMsg>>,
    id: String,
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

pub async fn forwarder(id: String, ws: Arc<Mutex<SplitStream<WebSocket>>>, mut fd: tokio::sync::mpsc::Sender<PlayerWebSocketMsg>) -> Result<(), ForwardingError>{
    loop{
        let msg: Option<WebSocketStreamItem>;
        {
            let mut rdr = ws.lock().await;
            msg = (*rdr).next().await;
        }
        match msg {
            Some(ws_stream_item) => {
                fd.send((id.clone(), ws_stream_item)).await.map_err(|e| {
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
    pub fn new(id: &str, ws: Option<WebSocket>, rcv_fwd: Option<Sender<PlayerWebSocketMsg>>) -> Self {
        let mut pwsc = PlayerWebSocketConnection {
            // sock: None,
            producer: None,
            player_listener: None,
            fwd_pipe: rcv_fwd,
            id: String::from(id),
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
        // let (tx, rx) = mpsc::channel(1024);
        // tokio::task::spawn(client_ws_rcv.forward(tx.map_err(|e| {
        //         return warp::Error { inner: Box::new(e) };
        //     })));
        self.player_listener = Some(Arc::new(Mutex::new(client_ws_rcv)));
        if self.fwd_pipe.is_some(){
            self.setup_ws_forwarding(self.fwd_pipe.as_ref().unwrap().clone());
        }
    }

    pub fn setup_ws_forwarding(&mut self, rcv_fwd: Sender<PlayerWebSocketMsg>) -> Result<(), InvalidError> {
        if self.player_listener.is_none(){
            InvalidError::new("Forwarding already setup elsewhere");
        }
        self.fwd_pipe = Some(rcv_fwd);
        tokio::task::spawn(forwarder(self.id.clone(), self.player_listener.as_ref().unwrap().clone(), self.fwd_pipe.as_ref().unwrap().clone()));
        self.player_listener = None;
        Ok(())
    }

    pub fn get_id(&self) -> &str {
        &self.id
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
                let pwsc = PlayerWebSocketConnection::new("uNiQiD", Some(websocket), None);
                let lstr = pwsc.player_listener.unwrap();
                let mut rcvr = lstr.lock().await;
                let msg = (*rcvr).next().await.unwrap().unwrap();
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
