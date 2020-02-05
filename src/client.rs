use std::env;

use reqwest::{blocking, header};
use rpassword::read_password_from_tty;
use serde::{Deserialize, Serialize};
use text_io::read;
use url::Url;

const API_HOST: &str = "http://localhost:8000";

pub struct APIClient {
    username: String,
    password: String,
    api_host: String,
    pub client: blocking::Client,
}

fn get_url(base_url: &str, endpoint: &str) -> Result<String, &'static str> {
    let base = match Url::parse(base_url) {
        Ok(val) => val,
        Err(_err) => return Err("Failed to parse base url"),
    };
    let url = match base.join(endpoint) {
        Ok(val) => val,
        Err(_) => return Err("Failed to join endpoint"),
    };
    Ok(url.to_string())
}

impl APIClient {
    pub fn new() -> Result<APIClient, &'static str> {
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
                error!("Falling back to default api host");
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

        let login_url = match get_url(&api_host, "/api/auth/login") {
            Ok(val) => val,
            Err(err) => {
                error!("{}", err);
                return Err(err);
            }
        };

        let resp = match api_client
            .post(login_url.as_str())
            .json(&LoginRequest {
                email: username.clone(),
                password: password.clone(),
            })
            .send()
        {
            Ok(r) => r,
            Err(err) => {
                error!("{}", err);
                return Err("Couldn't send this request. Please check your network and try again.");
            }
        };
        let resp_body = resp.text().unwrap();
        debug!("server response {}", resp_body);
        let resp: LoginResponse = match serde_json::from_str(&resp_body) {
            Ok(r) => r,
            Err(_err) => {
                let err: LoginResponseError = match serde_json::from_str(&resp_body) {
                    Ok(data) => data,
                    Err(err) => {
                        error!("{}", err);
                        return Err("Failed to understand server response");
                    }
                };
                debug!("{:?}", err.non_field_errors);
                return Err("Failed to login with provided credentials.");
            }
        };

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

    pub fn list_envs(&self) -> Result<Vec<Env>, &'static str> {
        let endpoint = get_url(&self.api_host, "/api/environments/").unwrap();
        let resp = match self.client.get(endpoint.as_str()).send() {
            Ok(r) => r,
            Err(err) => {
                error!("{}", err);
                return Err("API error");
            }
        };
        let response_body = match resp.text() {
            Ok(val) => val,
            Err(err) => {
                error!("{}", err);
                return Err("Failed to deserializer response");
            }
        };

        let envs: Vec<Env> = match serde_json::from_str(&response_body) {
            Ok(vals) => vals,
            Err(_err) => {
                let api_err: APIError = match serde_json::from_str(&response_body) {
                    Ok(e) => e,
                    Err(err) => {
                        error!("{}", err);
                        return Err("Failed to deserializer");
                    }
                };
                error!("{}", api_err.detail);
                return Err("API Error.");
            }
        };
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
