pub mod notifiers;
pub mod providers;

use std::error::Error;
use std::fmt;

// Custom Error

#[derive(Debug, Clone)]
pub struct MyError {
    message: String,
}

impl MyError {
    pub fn new(msg: &str) -> MyError {
        MyError {
            message: msg.into(),
        }
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for MyError {}
