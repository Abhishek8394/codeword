pub trait Player{
    fn get_name(&self) -> &str;
}

#[derive(Debug, Clone)]
pub struct SimplePlayer {
    name: String,
}

impl SimplePlayer{
    pub fn new(name: &str) -> Self{
        SimplePlayer{
            name: String::from(name)
        }
    }
}

impl Player for SimplePlayer{
    fn get_name(&self) -> &str {
        &self.name[..]
    }
}

