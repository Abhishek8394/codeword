pub trait Player {
    fn get_name(&self) -> &str;
    fn get_id(&self) -> &u32;
}

#[derive(Debug, Clone)]
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
