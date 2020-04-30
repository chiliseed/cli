use serde::{Deserialize, Serialize};

use crate::api_client::types::ApiResult;
use crate::api_client::utils::{deserialize_body, handle_empty_response_or_error};
use crate::api_client::ApiClient;

impl ApiClient {
    pub fn list_env_vars(
        &self,
        service_slug: &str,
    ) -> ApiResult<Vec<ListEnvironmentVariableResponse>> {
        let (response, status) = self.get(&format!(
            "/api/service/{}/environment-variables/",
            service_slug
        ))?;
        let env_vars: Vec<ListEnvironmentVariableResponse> = deserialize_body(&response, status)?;
        Ok(env_vars)
    }

    pub fn create_env_var(
        &self,
        service_slug: &str,
        env_var: &CreateEnvironmentVariableRequest,
    ) -> ApiResult<CreateEnvironmentVariableResponse> {
        let (response, status) = self.post(
            &format!("/api/service/{}/environment-variables/", service_slug),
            Some(env_var),
        )?;

        let key: CreateEnvironmentVariableResponse = deserialize_body(&response, status)?;
        Ok(key)
    }

    pub fn create_env_var_in_project(
        &self,
        project_slug: &str,
        env_var: &CreateEnvironmentVariableRequest,
    ) -> ApiResult<Vec<String>> {
        let (response, status) = self.post(
            &format!("/api/project/{}/environment-variables/", project_slug),
            Some(env_var),
        )?;
        let keys: Vec<String> = deserialize_body(&response, status)?;
        Ok(keys)
    }

    pub fn delete_env_var(
        &self,
        service_slug: &str,
        payload: &DeleteEnvironmentVariableRequest,
    ) -> ApiResult<()> {
        let (response, status) = self.delete(
            &format!("/api/service/{}/environment-variables/", service_slug),
            Some(payload),
        )?;
        handle_empty_response_or_error(&response, status)
    }
}

#[derive(Debug, Serialize)]
pub struct CreateEnvironmentVariableRequest {
    pub key_name: String,
    pub key_value: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEnvironmentVariableResponse {
    pub key_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ListEnvironmentVariableResponse {
    pub name: String,
    pub value_from: String,
    pub value: String,
    pub arn: String,
    pub kind: String,
    pub last_modified: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteEnvironmentVariableRequest {
    pub key_name: String,
}
