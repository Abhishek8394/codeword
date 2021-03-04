use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, Result};
use tokio::sync::RwLock;
use redis::aio::ConnectionManager;
use redis::RedisError;

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
pub struct InMemSessionStore{
    db: Arc<RwLock<HashMap<String, String>>>,
}

impl InMemSessionStore{
    pub fn new() -> Self{
        InMemSessionStore{
            db: Arc::new(RwLock::new(HashMap::new()))
        }
    }

    pub async fn get(&self, key: &str) -> Option<String>{
        let reader = self.db.read().await;
        if let Some(val) = (*reader).get(key){
            return Some(val.clone());
        }
        None
    }

    pub async fn insert(&self, key: String, val: String) -> Result<()> {
        let mut writer = self.db.write().await;
        (*writer).insert(key, val);
        Ok(())
    }
}

#[derive(Clone)]
pub struct RedisSessionStore{
    conn: ConnectionManager,
}

impl RedisSessionStore{
    pub async fn new(addr: &str) -> Result<Self, RedisError> {
        let client = redis::Client::open(addr)?;
        Ok(RedisSessionStore{
            conn: ConnectionManager::new(client).await?
        })
    }

    pub async fn get(&mut self, key: &str) -> Option<String> {
        let resp = redis::cmd("GET").arg(key).query_async(&mut self.conn).await;
        match resp{
            Ok(item) => {
                item
            },
            Err(e) => {
                eprintln!("Error fetching from redis: {:?}", e);
                None
            }
        }
    }

    pub async fn insert(&mut self, key: String, val: String) -> Result<()> {
        let resp: Result<(), RedisError> = redis::cmd("SET").arg(key).arg(val).query_async(&mut self.conn).await;
        match resp{
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Error inserting to redis: {:?}", e);
                let msg = format!("Error: {:?}", e);
                bail!(msg)
            },
        }
    }
}
