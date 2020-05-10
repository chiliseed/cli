use serde::{Deserialize, Serialize};

use crate::api_client::types::ApiResult;
use crate::api_client::utils::deserialize_body;
use crate::api_client::ApiClient;

impl ApiClient {
    pub fn create_db(
        &self,
        env_slug: &str,
        params: &CreateDbRequest,
    ) -> ApiResult<CreateDbResponse> {
        let (response, status) = self.post(
            &format!("/api/environment/{}/add-db/", env_slug),
            Some(params),
        )?;
        let db: CreateDbResponse = deserialize_body(&response, status)?;
        Ok(db)
    }

    pub fn add_db_to_service(
        &self,
        service_slug: &str,
        params: &AddDbRequest,
    ) -> ApiResult<AddDbResponse> {
        let (response, status) = self.post(
            &format!("/api/service/{}/add-db", service_slug),
            Some(params),
        )?;
        let log: AddDbResponse = deserialize_body(&response, status)?;
        Ok(log)
    }
}

#[derive(Debug, Serialize)]
pub struct CreateDbRequest {
    pub name: String,
    pub username: String,
    pub engine: String,
    pub preset: String,
    pub project: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDbResponse {
    pub log: String,
    pub resource: String,
}

#[derive(Debug, Serialize)]
pub struct AddDbRequest {
    pub db_slug: String,
}

#[derive(Debug, Deserialize)]
pub struct AddDbResponse {
    pub log: String,
}
