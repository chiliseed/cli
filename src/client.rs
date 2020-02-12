use std::error::Error;
use std::{env, fmt};

use chrono::{DateTime, Utc};
use reqwest;
use reqwest::{blocking, header, StatusCode};
use rpassword::read_password_from_tty;
use serde::{Deserialize, Serialize};
use text_io::read;
use url::{ParseError, Url};

use crate::schemas::{Env, ExecLog};

const API_HOST: &str = "http://localhost:8000";

#[derive(Debug)]
pub enum APIClientError {
    HTTPRequestError(String),
    HTTPTimeoutError(String),
    DeSerializerError(String),
    URLParseError(ParseError),
}

impl Error for APIClientError {
    fn description(&self) -> &str {
        match *self {
            APIClientError::HTTPRequestError(ref cause) => cause,
            APIClientError::HTTPTimeoutError(ref cause) => cause,
            APIClientError::DeSerializerError(ref cause) => cause,
            APIClientError::URLParseError(ref err) => Error::description(err),
        }
    }
}

impl From<ParseError> for APIClientError {
    fn from(err: ParseError) -> APIClientError {
        APIClientError::URLParseError(err)
    }
}

impl From<reqwest::Error> for APIClientError {
    fn from(err: reqwest::Error) -> APIClientError {
        APIClientError::HTTPTimeoutError(err.to_string())
    }
}

impl fmt::Display for APIClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

pub type APIResult<T> = Result<T, APIClientError>;
type ResponseBody = String;

pub struct APIClient {
    username: String,
    password: String,
    api_host: String,
    pub client: blocking::Client,
}

fn get_url(base_url: &str, endpoint: &str) -> APIResult<String> {
    let base = Url::parse(base_url)?;
    let url = base.join(endpoint)?;
    Ok(url.to_string())
}

impl APIClient {
    pub fn new() -> APIResult<APIClient> {
        let username = match env::var("CHILISEED_USERNAME") {
            Ok(val) => val,
            Err(_err) => {
                println!("Username: ");
                read!()
            }
        };

        let password = match env::var("CHILISEED_PASSWORD") {
            Ok(val) => val,
            Err(_err) => {
                let val = read_password_from_tty(Some("Password: ")).unwrap();
                val
            }
        };

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
            .build()
            .unwrap();

        let login_url = get_url(&api_host, "/api/auth/login")?;

        let resp = api_client
            .post(login_url.as_str())
            .json(&LoginRequest {
                email: username.clone(),
                password: password.clone(),
            })
            .send()?;

        let resp_body = resp.text().unwrap();

        debug!("server response {}", resp_body);

        let resp: LoginResponse = serde_json::from_str(&resp_body).map_err(|_err| {
            let err: LoginResponseError = serde_json::from_str(&resp_body)
                .map_err(|_err| {
                    return APIClientError::DeSerializerError(
                        "Failed to understand server response".to_owned(),
                    );
                })
                .unwrap();
            debug!("{:?}", err.non_field_errors);
            return APIClientError::HTTPRequestError(
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

        Ok(APIClient {
            username,
            password,
            api_host: api_host.to_string(),
            client: api_client,
        })
    }

    fn get(&self, endpoint: &str) -> APIResult<(ResponseBody, StatusCode)> {
        let url = get_url(&self.api_host, endpoint)?;
        let resp = self.client.get(&url).send()?;
        let status = resp.status();
        let body = resp.text().unwrap();
        Ok((body.to_owned(), status))
    }

    fn post<T: Serialize>(
        &self,
        endpoint: &str,
        payload: Option<&T>,
    ) -> APIResult<(ResponseBody, StatusCode)> {
        let url = get_url(&self.api_host, endpoint)?;
        if let Some(data) = payload {
            let req = self.client.post(&url).json(data).send()?;
            let status = req.status();
            let body = req.text().unwrap();
            Ok((body, status))
        } else {
            let req = self.client.post(&url).send()?;
            let status = req.status();
            let body = req.text().unwrap();
            Ok((body, status))
        }
    }

    pub fn list_envs(&self) -> APIResult<Vec<Env>> {
        let (response_body, status_code) = self.get("/api/environments/")?;

        let envs: Vec<Env> = serde_json::from_str(&response_body).map_err(|err| {
            if status_code.is_server_error() {
                let api_err: APIError = serde_json::from_str(&response_body)
                    .map_err(|err| APIClientError::DeSerializerError(err.to_string()))
                    .unwrap();
                error!("{}", api_err.detail);
                return APIClientError::HTTPRequestError(api_err.detail);
            }
            error!("{}", err.to_string());
            APIClientError::DeSerializerError("Failed to parse error response".to_string())
        })?;
        Ok(envs)
    }

    pub fn create_env(&self, env: &CreateEnvRequest) -> APIResult<CreateEnvResponse> {
        let (response_body, status_code) = self.post("/api/environments/create", Some(env))?;

        debug!("response body {}", response_body);

        let env: CreateEnvResponse = serde_json::from_str(&response_body).map_err(|err| {
            if status_code.is_server_error() {
                let api_err: APIError = serde_json::from_str(&response_body)
                    .map_err(|_err| APIClientError::DeSerializerError(err.to_string()))
                    .unwrap();
                error!("{}", api_err.detail);
                return APIClientError::HTTPRequestError(api_err.detail);
            }
            if status_code.is_client_error() {
                let api_err: CreateEnvResponseError = serde_json::from_str(&response_body)
                    .map_err(|err| APIClientError::DeSerializerError(err.to_string()))
                    .unwrap();
                if let Some(name_err) = api_err.name {
                    return APIClientError::HTTPRequestError(name_err[0].to_owned());
                }
                if let Some(domain_err) = api_err.domain {
                    return APIClientError::HTTPRequestError(domain_err[0].to_owned());
                }
                if let Some(region_err) = api_err.region {
                    return APIClientError::HTTPRequestError(region_err[0].to_owned());
                }
            }
            error!("{}", err.to_string());
            APIClientError::DeSerializerError("Failed to parse error response".to_string())
        })?;
        Ok(env)
    }

    pub fn get_exec_log(&self, slug: String) -> APIResult<ExecLog> {
        let (response_body, status) = self.get(&format!("/api/execution/status/{}", slug))?;
        let log: ExecLog = serde_json::from_str(&response_body).unwrap();
        Ok(log)
    }
}

#[derive(Serialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    auth_token: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponseError {
    non_field_errors: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateEnvRequest {
    pub name: String,
    pub domain: String,
    pub region: String,
    pub access_key: String,
    pub access_key_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEnvResponse {
    pub env: Env,
    pub log: String,
}

#[derive(Debug, Deserialize)]
struct CreateEnvResponseError {
    name: Option<Vec<String>>,
    region: Option<Vec<String>>,
    domain: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct APIError {
    detail: String,
}
