use std::error::Error;
use std::{env, fmt};

use reqwest;
use reqwest::{blocking, header};
use rpassword::read_password_from_tty;
use serde::{Deserialize, Serialize};
use text_io::read;
use url::{ParseError, Url};

const API_HOST: &str = "http://localhost:8000";

#[derive(Debug)]
pub enum APIClientError {
    HTTPRequestError(String),
    HTTPTimeoutError(String),
    SerializerError(String),
    DeSerializerError(String),
    URLParseError(ParseError),
}

impl Error for APIClientError {
    fn description(&self) -> &str {
        match *self {
            APIClientError::HTTPRequestError(ref cause) => cause,
            APIClientError::HTTPTimeoutError(ref cause) => cause,
            APIClientError::SerializerError(ref cause) => cause,
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
                let val = read!();
                val
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

    pub fn list_envs(&self) -> APIResult<Vec<Env>> {
        let endpoint = get_url(&self.api_host, "/api/environments/")?;
        let resp = self.client.get(endpoint.as_str()).send()?;
        let response_body = resp.text().unwrap();

        let envs: Vec<Env> = serde_json::from_str(&response_body).map_err(|err| {
            let api_err: APIError = serde_json::from_str(&response_body)
                .map_err(|_err| APIClientError::DeSerializerError(err.to_string()))
                .unwrap();
            error!("{}", api_err.detail);
            return APIClientError::HTTPRequestError(api_err.detail);
        })?;
        Ok(envs)
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

#[derive(Debug, Deserialize)]
pub struct Env {
    pub slug: String,
    pub name: String,
    pub region: String,
    pub domain: String,
}

#[derive(Debug, Deserialize)]
struct APIError {
    detail: String,
}
