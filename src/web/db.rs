use crate::web::errors::DuplicateLobbyError;
use crate::web::ws::PlayerWebSocketMsg;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;

use super::lobby::Lobby;
use anyhow::{bail, Result};

#[derive(Clone)]
pub struct RedisGameDB {}

type ArcLobbyWrapper = Arc<RwLock<Lobby>>;

#[derive(Clone)]
pub struct InMemGameDB {
    db: Arc<RwLock<HashMap<String, ArcLobbyWrapper>>>,
}

impl InMemGameDB {
    pub fn new() -> Self {
        Self {
            db: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_num_lobbies(&self) -> Result<usize> {
        let r1 = self.db.read().await;
        Ok((*r1).len())
    }

    pub async fn add_new_lobby(
        &self,
        lobby: Lobby,
    ) -> std::result::Result<(), DuplicateLobbyError> {
        eprintln!("Adding game lobby: {}", lobby.id);
        let mut w1 = self.db.write().await;
        if let Some(_) = (*w1).get(&lobby.id) {
            let msg = format!("Trying to insert duplicate id: {}", lobby.id);
            eprintln!("{}", msg);
            return Err(DuplicateLobbyError::new(&lobby.id));
        }
        (*w1).insert(lobby.id.to_string(), Arc::new(RwLock::new(lobby)));
        Ok(())
    }

    pub async fn get_lobby(&self, lobby_id: &str) -> Result<ArcLobbyWrapper> {
        let r1 = self.db.read().await;
        if let Some(lobby) = (*r1).get(lobby_id) {
            return Ok(lobby.clone());
        } else {
            bail!(format!("No lobby found for: {:}", lobby_id));
        }
    }

    pub async fn drop_lobby(&mut self, lobby_id: &str) {
        let mut w1 = self.db.write().await;
        if let Some(lobby) = (*w1).remove(lobby_id){
            {
                let mut writer = lobby.write().await;
                (writer).quit().await;
            }
        }
    }
}

pub fn spawn_lobby_ws_listen_task(
    db: InMemGameDB,
    game_id: &str,
    mut lobby_ws_rcvr: Receiver<PlayerWebSocketMsg>,
) {
    let game_id: String = game_id.to_string();
    tokio::task::spawn(async move {
        eprintln!("[{:?}] Starting websocket loop", game_id);
        loop {
            if let Ok(lobby) = db.get_lobby(&game_id).await {
                let pws_msg = lobby_ws_rcvr.recv().await;
                match pws_msg {
                    Some(pws_msg) => {
                        if let (uniq_id, Ok(msg)) = pws_msg {
                            let lobby_rdr = lobby.read().await;
                            println!("[{}] Got ({}): {:?}", (*lobby_rdr).get_id(), uniq_id, msg);
                            // TODO:
                            // - match uniq id
                            // - handle auth msg
                            // - handle game msg
                            // for early quit, poll for num players connected?
                        }
                    }
                    // everyone has disconnected, drop out and delete lobby maybe?
                    // If don't want to drop, then remove break.
                    None => break,
                }
            }
        }
        eprintln!("[{:?}] Ended websocket loop", game_id);
    });
}

pub fn spawn_lobby_death_timer(mut db: InMemGameDB, game_id: &str, duration: Duration) {
    let game_id: String = game_id.to_string();
    tokio::task::spawn(async move {
        eprintln!("[{:?}] Starting death timer: {:?}", game_id, duration);
        tokio::time::delay_for(duration).await; // called sleep in 1.0+
        db.drop_lobby(&game_id).await;
    });
}
