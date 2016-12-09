use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ErrorStr<'a> {
    description: &'a str
}

impl<'a> fmt::Display for ErrorStr<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl<'a> ErrorStr<'a> {
    pub fn new(description: &'a str) -> ErrorStr<'a> {
        ErrorStr {
            description: description
        }
    }
}

impl<'a> Error for ErrorStr<'a> {
    fn description(&self) -> &str {
        self.description
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
