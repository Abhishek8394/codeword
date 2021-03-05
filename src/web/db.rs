use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, Result};
use redis::aio::ConnectionManager;
use redis::RedisError;
use tokio::sync::RwLock;

use super::lobby::Lobby;
use crate::web::errors::DuplicateLobbyError;

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
        if let Some(lobby) = (*w1).remove(lobby_id) {
            {
                let mut writer = lobby.write().await;
                (writer).quit().await;
            }
        }
    }
}

/// This is only for testing. A simple hashmap based util. Use with redis or something in prod.
#[derive(Clone)]
pub struct InMemSessionStore {
    db: Arc<RwLock<HashMap<String, HashMap<String, String>>>>,
}

impl InMemSessionStore {
    pub fn new() -> Self {
        InMemSessionStore {
            db: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn gen_internal_key(&self, sess_id: &str) -> String {
        format!("cookies_{}", sess_id)
    }

    pub async fn get(&self, sess_id: &str, key: &str) -> Option<String> {
        let reader = self.db.read().await;
        if let Some(cookie_data) = (*reader).get(&self.gen_internal_key(sess_id)) {
            if let Some(val) = cookie_data.get(key) {
                return Some(val.clone());
            };
        }
        None
    }

    pub async fn insert(&self, sess_id: &str, key: String, val: String) -> Result<()> {
        let mut writer = self.db.write().await;
        let cookie_data = (*writer)
            .entry(self.gen_internal_key(sess_id))
            .or_insert(HashMap::new());
        cookie_data.insert(key, val);
        Ok(())
    }
}

#[derive(Clone)]
pub struct RedisSessionStore {
    conn: ConnectionManager,
}

impl RedisSessionStore {
    pub async fn new(addr: &str) -> Result<Self, RedisError> {
        let client = redis::Client::open(addr)?;
        Ok(RedisSessionStore {
            conn: ConnectionManager::new(client).await?,
        })
    }

    fn gen_internal_key(&self, sess_id: &str) -> String {
        format!("cookies_{}", sess_id)
    }

    pub async fn get(&mut self, sess_id: &str, key: &str) -> Option<String> {
        let internal_key = self.gen_internal_key(sess_id);
        let resp = redis::cmd("HGET")
            .arg(internal_key)
            .arg(key)
            .query_async(&mut self.conn)
            .await;
        match resp {
            Ok(item) => item,
            Err(e) => {
                eprintln!("Error fetching from redis: {:?}", e);
                None
            }
        }
    }

    pub async fn insert(&mut self, sess_id: &str, key: String, val: String) -> Result<()> {
        let internal_key = self.gen_internal_key(sess_id);
        let resp: Result<(), RedisError> = redis::cmd("HSET")
            .arg(internal_key)
            .arg(key)
            .arg(val)
            .query_async(&mut self.conn)
            .await;
        match resp {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Error inserting to redis: {:?}", e);
                let msg = format!("Error: {:?}", e);
                bail!(msg)
            }
        }
    }
}
