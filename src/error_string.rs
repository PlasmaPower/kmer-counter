use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ErrorString {
    description: String,
}

impl ErrorString {
    pub fn new(description: String) -> ErrorString {
        ErrorString { description: description }
    }
}

impl fmt::Display for ErrorString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl Error for ErrorString {
    fn description(&self) -> &str {
        self.description.as_str()
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
