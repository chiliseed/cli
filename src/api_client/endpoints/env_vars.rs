use crate::api_client::schemas;
use crate::api_client::types::ApiResult;
use crate::api_client::utils::{deserialize_body, handle_empty_response_or_error};
use crate::api_client::ApiClient;

impl ApiClient {
    pub fn list_env_vars(
        &self,
        service_slug: &str,
    ) -> ApiResult<Vec<schemas::ListEnvironmentVariableResponse>> {
        let (response, status) = self.get(&format!(
            "/api/service/{}/environment-variables/",
            service_slug
        ))?;
        let env_vars: Vec<schemas::ListEnvironmentVariableResponse> =
            deserialize_body(&response, status)?;
        Ok(env_vars)
    }

    pub fn create_env_var(
        &self,
        service_slug: &str,
        env_var: &schemas::CreateEnvironmentVariableRequest,
    ) -> ApiResult<schemas::CreateEnvironmentVariableResponse> {
        let (response, status) = self.post(
            &format!("/api/service/{}/environment-variables/", service_slug),
            Some(env_var),
        )?;

        let key: schemas::CreateEnvironmentVariableResponse = deserialize_body(&response, status)?;
        Ok(key)
    }

    pub fn delete_env_var(
        &self,
        service_slug: &str,
        payload: &schemas::DeleteEnvironmentVariableRequest,
    ) -> ApiResult<()> {
        let (response, status) = self.delete(
            &format!("/api/service/{}/environment-variables/", service_slug),
            Some(payload),
        )?;
        handle_empty_response_or_error(&response, status)
    }
}
