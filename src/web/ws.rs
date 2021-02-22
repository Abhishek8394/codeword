use crate::web::errors::{ForwardingError, WebSocketError};
use futures::stream::SplitStream;
use futures::StreamExt;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{
    mpsc::{self, Sender},
    Mutex,
};
use tokio_stream::wrappers::ReceiverStream;
use warp::ws::Message;
use warp::ws::WebSocket;

/// websocket msg from a player. (websocket id, ws message.)
pub type WebSocketStreamItem = Result<Message, warp::Error>;
pub type PlayerWebSocketMsg = (String, WebSocketStreamItem);

pub struct PlayerWebSocketConnection {
    /// Send msg to ws
    producer: Option<Arc<Mutex<Sender<WebSocketStreamItem>>>>,
    /// Hold player ws receiver if not in use atm
    player_listener: Option<Arc<Mutex<SplitStream<WebSocket>>>>,
    /// The sending end for forwarding ws msgs received from player side. player -> ws -> fwd_pipe (Sender) -> whatever rcvr for Sender.
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

pub async fn forwarder(
    id: String,
    ws: Arc<Mutex<SplitStream<WebSocket>>>,
    fd: tokio::sync::mpsc::Sender<PlayerWebSocketMsg>,
) -> Result<(), ForwardingError> {
    loop {
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
            }
            None => break,
        }
    }
    eprintln!("[{}] Quitting forward loop", id);
    Ok(())
}

impl PlayerWebSocketConnection {
    pub fn new(
        id: &str,
        ws: Option<WebSocket>,
        rcv_fwd: Option<Sender<PlayerWebSocketMsg>>,
    ) -> Result<Self, WebSocketError> {
        let mut pwsc = PlayerWebSocketConnection {
            // sock: None,
            producer: None,
            player_listener: None,
            fwd_pipe: rcv_fwd,
            id: String::from(id),
        };
        if ws.is_some() {
            pwsc.set_websocket(ws.unwrap())?;
        }
        return Ok(pwsc);
    }

    pub fn set_websocket(&mut self, ws: WebSocket) -> Result<(), WebSocketError> {
        let (client_ws_sender, client_ws_rcv) = ws.split();
        let (tx, rx) = mpsc::channel(1024);
        let rx = ReceiverStream::new(rx);
        tokio::task::spawn(rx.forward(client_ws_sender));
        self.producer = Some(Arc::new(Mutex::new(tx)));
        // let (tx, rx) = mpsc::channel(1024);
        // tokio::task::spawn(client_ws_rcv.forward(tx.map_err(|e| {
        //         return warp::Error { inner: Box::new(e) };
        //     })));
        self.player_listener = Some(Arc::new(Mutex::new(client_ws_rcv)));
        if self.fwd_pipe.is_some() {
            self.setup_ws_forwarding(self.fwd_pipe.as_ref().unwrap().clone())?;
        }
        Ok(())
    }

    pub fn setup_ws_forwarding(
        &mut self,
        rcv_fwd: Sender<PlayerWebSocketMsg>,
    ) -> Result<(), WebSocketError> {
        if self.player_listener.is_none() {
            return Err(WebSocketError::PipeSetupError(
                "Forwarding already setup elsewhere".to_string(),
            ));
        }
        self.fwd_pipe = Some(rcv_fwd);
        tokio::task::spawn(forwarder(
            self.id.clone(),
            self.player_listener.as_ref().unwrap().clone(),
            self.fwd_pipe.as_ref().unwrap().clone(),
        ));
        self.player_listener = None;
        Ok(())
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Close the websocket.
    pub async fn close(&mut self) -> Result<(), WebSocketError> {
        let res = self
            .send_msg(Message::close())
            .await
            .map_err(|e| WebSocketError::CloseError(format!("{:?}", e)));
        self.producer = None;
        self.fwd_pipe = None;
        self.player_listener = None;
        return res;
        // TODO: Other cleanup.
    }

    /// Send a message to websocket.
    pub async fn send_msg(&self, msg: Message) -> Result<(), WebSocketError> {
        if self.producer.is_some() {
            let producer = self.producer.as_ref().unwrap().lock().await;
            return match (*producer).send(Ok(msg)).await {
                Ok(_) => Ok(()),
                Err(e) => Err(WebSocketError::SendError(e.to_string())),
            };
        }
        Ok(())
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
                let pwsc = PlayerWebSocketConnection::new("uNiQiD", Some(websocket), None).unwrap();
                let lstr = pwsc.player_listener.as_ref().unwrap();
                let mut rcvr = lstr.lock().await;
                let msg = (*rcvr).next().await.unwrap().unwrap();
                assert_eq!(Ok("hello"), msg.to_str());
                let sender = pwsc.producer.as_ref().unwrap().lock().await;
                (*sender).send(Ok(Message::text("world"))).await.unwrap();
            })
        });

        let mut client = warp::test::ws().handshake(route).await.expect("handshake");
        client.send_text("hello").await;
        let msg = client.recv().await;
        assert_eq!("world", msg.unwrap().to_str().unwrap());
    }
}
