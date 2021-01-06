use crate::errors::ParseError;
use anyhow::Result;
use serde::{Deserialize, Serialize};

pub type PlayerId = u32;

pub trait Player {
    fn get_name(&self) -> &str;
    fn get_id(&self) -> &PlayerId;
}

pub trait TryDeserialize {
    fn deserialize(s: String) -> Result<Self, ParseError>
    where
        Self: Sized;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplePlayer {
    /// Visible name
    name: String,

    /// Internal ID
    #[serde(default)]
    id: u32,
}

impl SimplePlayer {
    pub fn new(name: &str, id: u32) -> Self {
        SimplePlayer {
            name: String::from(name),
            id,
        }
    }

    pub fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = String::from(name);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partial_simple_player_from_json() {
        let json_str = "{\"name\": \"foouser\"}";
        let p: SimplePlayer = serde_json::from_str(json_str).unwrap();
        assert_eq!(p.name, "foouser");
        // assert_eq!(p.id, "foouser");
    }
}
