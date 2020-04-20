use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub(crate) struct ApiError {
    pub(crate) detail: String,
}

#[derive(Serialize)]
pub(crate) struct LoginRequest {
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginResponse {
    pub(crate) auth_token: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginResponseError {
    pub(crate) non_field_errors: Vec<String>,
}
