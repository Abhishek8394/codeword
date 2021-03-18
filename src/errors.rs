use std::fmt;

#[derive(Debug, Clone)]
pub struct InvalidError {
    msg: String,
}

impl InvalidError {
    pub fn new(msg: &str) -> Self {
        return InvalidError {
            msg: String::from(msg),
        };
    }
}

impl fmt::Display for InvalidError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid value: {}", self.msg)
    }
}

pub type InvalidMoveError = InvalidError;

pub type ParseError = InvalidError;

#[derive(Debug, Clone)]
pub struct GameBeginError<T: Clone>{
    old: T,
    msg: String,
}

impl<T: Clone> GameBeginError<T>{
    pub fn new(old: T, msg: &str) -> Self{
        GameBeginError{
            old,
            msg: msg.to_string()
        }
    }
}

impl<T: Clone> fmt::Display for GameBeginError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to begin game because: {}", self.msg)
    }
}

#[derive(Debug, Clone)]
pub struct GameFinishError<T: Clone>{
    old: T,
    msg: String,
}

impl<T: Clone> fmt::Display for GameFinishError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to finish game because: {}", self.msg)
    }
}
