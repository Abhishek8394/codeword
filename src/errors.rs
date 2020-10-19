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
