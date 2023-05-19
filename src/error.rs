use tokio::task::JoinError;

use std::error::Error as StdErrorTrait;
use std::fmt;
use std::io;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum Error {
    None,
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
    IO(io::Error),
    ParseIntError(ParseIntError),
    JoinError(JoinError),
    CsvError(csv::Error),
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl StdErrorTrait for Error {}

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}

impl From<JoinError> for Error {
    fn from(e: JoinError) -> Self {
        Self::JoinError(e)
    }
}

impl From<csv::Error> for Error {
    fn from(e: csv::Error) -> Self {
        Self::CsvError(e)
    }
}
