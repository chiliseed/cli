use std::env;

use reqwest::{blocking, header, StatusCode};
use serde::Serialize;

use super::errors::ApiClientError;
use super::schemas;
use super::types::{ApiResult, ResponseBody};
use super::utils::get_url;
use crate::schemas::ExecLog;

const API_HOST: &str = "http://localhost:8000";

pub struct ApiClient {
    api_host: String,
    pub client: blocking::Client,
}

impl ApiClient {
    pub fn new(username: &str, password: &str) -> ApiResult<ApiClient> {
        let api_host = match env::var("CHILISEED_API_HOST") {
            Ok(val) => val,
            Err(_err) => {
                warn!("Falling back to default api host");
                API_HOST.to_string()
            }
        };

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let api_client = blocking::ClientBuilder::new()
            .default_headers(headers.clone())
            .build()?;

        let login_url = get_url(&api_host, "/api/auth/login")?;

        debug!("Authenticating user");

        let resp = api_client
            .post(login_url.as_str())
            .json(&schemas::LoginRequest {
                email: username.to_string(),
                password: password.to_string(),
            })
            .send()?;

        let resp_body = resp.text().unwrap();

        debug!("server response {}", resp_body);

        let resp: schemas::LoginResponse = serde_json::from_str(&resp_body).map_err(|_err| {
            let err: schemas::LoginResponseError = serde_json::from_str(&resp_body)
                .map_err(|_err| {
                    return ApiClientError::DeSerializerError(
                        "Failed to understand server response".to_owned(),
                    );
                })
                .unwrap();
            debug!("{:?}", err.non_field_errors);
            return ApiClientError::HTTPRequestError(
                "Failed to login with provided credentials.".to_owned(),
            );
        })?;

        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Token {}", resp.auth_token)).unwrap(),
        );
        let api_client = blocking::ClientBuilder::new()
            .cookie_store(true)
            .default_headers(headers)
            .build()
            .unwrap();

        Ok(ApiClient {
            api_host: api_host.to_string(),
            client: api_client,
        })
    }

    pub(crate) fn get(&self, endpoint: &str) -> ApiResult<(ResponseBody, StatusCode)> {
        let url = get_url(&self.api_host, endpoint)?;
        let resp = self.client.get(&url).send()?;
        let status = resp.status();
        let body = resp.text().unwrap();
        Ok((body.to_owned(), status))
    }

    pub(crate) fn delete<T: Serialize>(
        &self,
        endpoint: &str,
        payload: Option<&T>,
    ) -> ApiResult<(ResponseBody, StatusCode)> {
        let url = get_url(&self.api_host, endpoint)?;
        if let Some(data) = payload {
            let resp = self.client.delete(&url).json(data).send()?;
            let status = resp.status();
            let body = resp.text().unwrap();
            Ok((body.to_owned(), status))
        } else {
            let resp = self.client.delete(&url).send()?;
            let status = resp.status();
            let body = resp.text().unwrap();
            Ok((body.to_owned(), status))
        }
    }

    pub(crate) fn get_with_query_params<T: Serialize>(
        &self,
        endpoint: &str,
        query: &T,
    ) -> ApiResult<(ResponseBody, StatusCode)> {
        let url = get_url(&self.api_host, endpoint)?;
        let resp = self.client.get(&url).query(&query).send()?;
        let status = resp.status();
        let body = resp.text().unwrap();
        Ok((body.to_owned(), status))
    }

    pub(crate) fn post<T: Serialize>(
        &self,
        endpoint: &str,
        payload: Option<&T>,
    ) -> ApiResult<(ResponseBody, StatusCode)> {
        let url = get_url(&self.api_host, endpoint)?;
        if let Some(data) = payload {
            let req = self.client.post(&url).json(data).send()?;
            let status = req.status();
            let body = req.text().unwrap();
            debug!("server response {}", body);
            Ok((body, status))
        } else {
            let req = self.client.post(&url).send()?;
            let status = req.status();
            let body = req.text().unwrap();
            debug!("server response {}", body);
            Ok((body, status))
        }
    }

    pub(crate) fn patch<T: Serialize>(
        &self,
        endpoint: &str,
        payload: Option<&T>,
    ) -> ApiResult<(ResponseBody, StatusCode)> {
        let url = get_url(&self.api_host, endpoint)?;
        if let Some(data) = payload {
            let req = self.client.patch(&url).json(data).send()?;
            let status = req.status();
            let body = req.text().unwrap();
            debug!("server response {}", body);
            Ok((body, status))
        } else {
            let req = self.client.patch(&url).send()?;
            let status = req.status();
            let body = req.text().unwrap();
            debug!("server response {}", body);
            Ok((body, status))
        }
    }

    pub fn get_exec_log(&self, slug: &str) -> ApiResult<ExecLog> {
        let (response_body, _) = self.get(&format!("/api/execution/status/{}", slug))?;
        let log: ExecLog = serde_json::from_str(&response_body).unwrap();
        Ok(log)
    }
}
