use std::fmt;
use std::error::Error as StdError;

#[derive(Debug)]
pub enum IrcClientError {
    LazyError,
}

impl StdError for IrcClientError {
    fn description(&self) -> &str {
        match *self {
            IrcClientError::LazyError => "Lazy error appeared",
//            IrcClientError::IdentifyError => "Error during identification",
        }
    }
}

impl fmt::Display for IrcClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IrcClientError::LazyError => f.write_str("LazyError"),
//            IrcClientError::IdentifyError => f.write_str("IdentifyError"),
        }
    }
}