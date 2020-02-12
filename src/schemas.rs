use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Status {
    pub slug: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct Env {
    pub slug: String,
    pub name: String,
    pub region: String,
    pub domain: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_status: Status,
}

#[derive(Debug, Deserialize)]
pub struct ExecLog {
    pub slug: String,
    pub action: String,
    pub is_success: bool,
    pub component: String,
    pub component_id: String,
    pub ended_at: DateTime<Utc>,
}
