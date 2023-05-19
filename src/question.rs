use serde::{Deserialize, Serialize};

use chrono::offset::Utc;
use chrono::DateTime;

#[derive(Debug, Serialize, Deserialize)]
pub struct Question {
    pub title: String,
    pub body_raw: String,
    pub body_cooked: String,
    pub created: DateTime<Utc>,
    pub username: String,
}
