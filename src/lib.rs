use tokio::task::JoinError;

use serde::{Deserialize, Serialize};

use chrono::offset::Utc;
use chrono::DateTime;

use std::io;
use std::num::ParseIntError;

#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    pub title: String,
    pub body_raw: String,
    pub body_cooked: String,
    pub created: DateTime<Utc>,
    pub username: String,
}

#[derive(Debug)]
pub enum Error {
    None,
    Reqwest(reqwest::Error),
    Json(serde_json::Error),
    IO(io::Error),
    ParseIntError(ParseIntError),
    JoinError(JoinError),
    CsvError(csv::Error),
}

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
