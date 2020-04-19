use super::errors::ApiClientError;

pub type ApiResult<T> = Result<T, ApiClientError>;
pub type ResponseBody = String;
