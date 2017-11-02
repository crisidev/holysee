use std::fmt;
use std::error::Error as StdError;

#[derive(Debug)]
pub enum TelegramClientError {
    LazyError,
}

impl StdError for TelegramClientError {
    fn description(&self) -> &str {
        match *self {
            TelegramClientError::LazyError => "Lazy error appeared",
        }
    }
}

impl fmt::Display for TelegramClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TelegramClientError::LazyError => f.write_str("LazyError"),
        }
    }
}
