use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Status {
    pub slug: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Env {
    pub slug: String,
    pub name: String,
    pub region: String,
    pub domain: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_status: Status,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExecLog {
    pub slug: String,
    pub action: String,
    pub is_success: Option<bool>,
    pub component: String,
    pub component_slug: String,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub slug: String,
    pub name: String,
    pub last_status: Status,
    pub environment: Env,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Service {
    pub slug: String,
    pub name: String,
    pub subdomain: String,
    pub container_port: String,
    pub alb_port_http: String,
    pub alb_port_https: String,
    pub health_check_endpoint: String,
}
