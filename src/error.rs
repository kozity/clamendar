use crossterm;
use serde_json;
use std::io;

#[derive(Debug)]
pub enum Error {
    Crossterm(crossterm::ErrorKind),
    DeletionWarning,
    InvalidIso,
    InvalidRecord,
    InvalidTime,
    Io(io::Error),
    NoInfo,
    Serde(serde_json::Error),
    YankWarning,
}

impl From<crossterm::ErrorKind> for Error {
    fn from(error: crossterm::ErrorKind) -> Self {
        Error::Crossterm(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Serde(error)
    }
}
