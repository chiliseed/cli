use reqwest::StatusCode;
use serde::Deserialize;
use url::Url;

use super::errors::ApiClientError;
use super::schemas;
use super::types::ApiResult;

pub(crate) fn get_url(base_url: &str, endpoint: &str) -> ApiResult<String> {
    let base = Url::parse(base_url)?;
    let url = base.join(endpoint)?;
    Ok(url.to_string())
}

pub(crate) fn deserialize_body<'de, T>(
    body: &'de str,
    status: StatusCode,
) -> Result<T, ApiClientError>
where
    T: Deserialize<'de>,
{
    serde_json::from_str(body).map_err(|err| {
        if status.is_server_error() {
            let api_err: Result<schemas::ApiError, _> = serde_json::from_str(body);
            match api_err {
                Ok(api_err) => {
                    error!("{}", api_err.detail);
                    ApiClientError::HTTPRequestError(api_err.detail)
                }
                Err(err) => ApiClientError::DeSerializerError(err.to_string()),
            }
        } else {
            error!("{}", err.to_string());
            ApiClientError::DeSerializerError(err.to_string())
        }
    })
}

pub(crate) fn handle_empty_response_or_error(
    response: &String,
    status: StatusCode,
) -> Result<(), ApiClientError> {
    if status.is_success() {
        Ok(())
    } else {
        match serde_json::from_str::<schemas::ApiError>(&response) {
            Ok(api_err) => {
                error!("{}", api_err.detail);
                return Err(ApiClientError::HTTPRequestError(api_err.detail));
            }
            Err(err) => Err(ApiClientError::DeSerializerError(err.to_string())),
        }
    }
}
