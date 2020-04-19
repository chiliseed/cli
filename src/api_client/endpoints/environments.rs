use crate::api_client::types::ApiResult;
use crate::api_client::utils::deserialize_body;
use crate::api_client::ApiClient;
use crate::api_client::{schemas, ApiClientError};
use crate::schemas::Env;

impl ApiClient {
    pub fn list_envs(&self, filters: Option<&schemas::EnvListFilters>) -> ApiResult<Vec<Env>> {
        let endpoint = "/api/environments/";
        let (response_body, status_code) = match filters {
            Some(f) => self.get_with_query_params(endpoint, f)?,
            None => self.get(endpoint)?,
        };

        let envs: Vec<Env> = deserialize_body(&response_body, status_code)?;
        Ok(envs)
    }

    pub fn create_env(
        &self,
        env: &schemas::CreateEnvRequest,
    ) -> ApiResult<schemas::CreateEnvResponse> {
        let (response_body, status_code) = self.post("/api/environments/create", Some(env))?;

        let env: schemas::CreateEnvResponse = deserialize_body(&response_body, status_code)
            .map_err(|err| {
                if status_code.is_client_error() {
                    let api_err: schemas::CreateEnvResponseError =
                        serde_json::from_str(&response_body)
                            .map_err(|err| ApiClientError::DeSerializerError(err.to_string()))
                            .unwrap();
                    if let Some(name_err) = api_err.name {
                        return ApiClientError::HTTPRequestError(name_err[0].to_owned());
                    }
                    if let Some(domain_err) = api_err.domain {
                        return ApiClientError::HTTPRequestError(domain_err[0].to_owned());
                    }
                    if let Some(region_err) = api_err.region {
                        return ApiClientError::HTTPRequestError(region_err[0].to_owned());
                    }
                }
                error!("{}", err.to_string());
                ApiClientError::DeSerializerError("Failed to parse error response".to_string())
            })?;
        Ok(env)
    }
}
