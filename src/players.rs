use crate::errors::ParseError;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub trait Player {
    fn get_name(&self) -> &str;
    fn get_id(&self) -> &u32;
}

pub trait TryDeserialize {
    fn deserialize(s: String) -> Result<Self, ParseError>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplePlayer {
    name: String,
    id: u32,
}

impl SimplePlayer {
    pub fn new(name: &str, id: u32) -> Self {
        SimplePlayer {
            name: String::from(name),
            id,
        }
    }
}

impl Player for SimplePlayer {
    fn get_name(&self) -> &str {
        &self.name[..]
    }
    fn get_id(&self) -> &u32 {
        &self.id
    }
}

impl TryDeserialize for SimplePlayer {
    fn deserialize(s: String) -> Result<Self, ParseError> {
        let res = serde_json::from_str(&s);
        if res.is_ok() {
            return Ok(res.unwrap());
        }
        return Err(ParseError::new("cannot parse"));
    }
}
