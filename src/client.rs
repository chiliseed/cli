use std::error::Error;
use std::{env, fmt};

use reqwest;
use reqwest::{blocking, header, StatusCode};
use rpassword::read_password_from_tty;
use serde::{Deserialize, Serialize};
use text_io::read;
use url::{ParseError, Url};

use crate::schemas::{Env, ExecLog, Project, Service, Worker};

const API_HOST: &str = "http://localhost:8000";

#[derive(Debug)]
pub enum ApiClientError {
    HTTPRequestError(String),
    HTTPTimeoutError(reqwest::Error),
    DeSerializerError(String),
    URLParseError(ParseError),
}

impl Error for ApiClientError {}

impl From<ParseError> for ApiClientError {
    fn from(err: ParseError) -> ApiClientError {
        ApiClientError::URLParseError(err)
    }
}

impl From<reqwest::Error> for ApiClientError {
    fn from(err: reqwest::Error) -> ApiClientError {
        ApiClientError::HTTPTimeoutError(err)
    }
}

impl fmt::Display for ApiClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ApiClientError::HTTPRequestError(ref cause) => write!(f, "{}", cause),
            ApiClientError::HTTPTimeoutError(ref err) => err.fmt(f),
            ApiClientError::DeSerializerError(ref cause) => write!(f, "{}", cause),
            ApiClientError::URLParseError(ref err) => err.fmt(f),
        }
    }
}

pub type ApiResult<T> = Result<T, ApiClientError>;
type ResponseBody = String;

pub struct ApiClient {
    api_host: String,
    pub client: blocking::Client,
}

fn get_url(base_url: &str, endpoint: &str) -> ApiResult<String> {
    let base = Url::parse(base_url)?;
    let url = base.join(endpoint)?;
    Ok(url.to_string())
}

fn deserialize_body<'de, T>(body: &'de str, status: StatusCode) -> Result<T, ApiClientError>
where
    T: Deserialize<'de>,
{
    serde_json::from_str(body).map_err(|err| {
        if status.is_server_error() {
            let api_err: Result<APIError, _> = serde_json::from_str(body);
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

fn handle_empty_response_or_error(
    response: &String,
    status: StatusCode,
) -> Result<(), ApiClientError> {
    if status.is_success() {
        Ok(())
    } else {
        match serde_json::from_str::<APIError>(&response) {
            Ok(api_err) => {
                error!("{}", api_err.detail);
                return Err(ApiClientError::HTTPRequestError(api_err.detail));
            }
            Err(err) => Err(ApiClientError::DeSerializerError(err.to_string())),
        }
    }
}

impl ApiClient {
    pub fn new() -> ApiResult<ApiClient> {
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
            .build()?;

        let login_url = get_url(&api_host, "/api/auth/login")?;

        debug!("Authenticating user");

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

    fn get(&self, endpoint: &str) -> ApiResult<(ResponseBody, StatusCode)> {
        let url = get_url(&self.api_host, endpoint)?;
        let resp = self.client.get(&url).send()?;
        let status = resp.status();
        let body = resp.text().unwrap();
        Ok((body.to_owned(), status))
    }

    fn delete<T: Serialize>(
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

    fn get_with_query_params<T: Serialize>(
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

    fn post<T: Serialize>(
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

    // fn patch<T: Serialize>(
    //     &self,
    //     endpoint: &str,
    //     payload: &T,
    // ) -> APIResult<(ResponseBody, StatusCode)> {
    //     let url = get_url(&self.api_host, endpoint)?;
    //     let req = self.client.patch(&url).json(payload).send()?;
    //     let status = req.status();
    //     let body = req.text().unwrap();
    //     debug!("server response {}", body);
    //     Ok((body, status))
    // }

    pub fn list_envs(&self, filters: Option<&EnvListFilters>) -> ApiResult<Vec<Env>> {
        let endpoint = "/api/environments/";
        let (response_body, status_code) = match filters {
            Some(f) => self.get_with_query_params(endpoint, f)?,
            None => self.get(endpoint)?,
        };

        let envs: Vec<Env> = deserialize_body(&response_body, status_code)?;
        Ok(envs)
    }

    pub fn create_env(&self, env: &CreateEnvRequest) -> ApiResult<CreateEnvResponse> {
        let (response_body, status_code) = self.post("/api/environments/create", Some(env))?;

        let env: CreateEnvResponse =
            deserialize_body(&response_body, status_code).map_err(|err| {
                if status_code.is_client_error() {
                    let api_err: CreateEnvResponseError = serde_json::from_str(&response_body)
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

    pub fn get_exec_log(&self, slug: &str) -> ApiResult<ExecLog> {
        let (response_body, _) = self.get(&format!("/api/execution/status/{}", slug))?;
        let log: ExecLog = serde_json::from_str(&response_body).unwrap();
        Ok(log)
    }

    pub fn list_projects(
        &self,
        env_slug: &str,
        filter: Option<&ProjectListFilters>,
    ) -> ApiResult<Vec<Project>> {
        let url = format!("/api/environment/{}/projects", env_slug);
        let (response_body, status) = match filter {
            Some(query) => self.get_with_query_params(&url, query)?,
            None => self.get(&url)?,
        };

        let projects: Vec<Project> = deserialize_body(&response_body, status)?;
        Ok(projects)
    }

    pub fn create_project(
        &self,
        project: &ProjectRequest,
        env_slug: &str,
    ) -> ApiResult<CreateProjectResponse> {
        let (response_body, status) = self.post(
            &format!("/api/environment/{}/projects/", env_slug),
            Some(project),
        )?;

        let project: CreateProjectResponse = deserialize_body(&response_body, status)?;
        Ok(project)
    }

    pub fn list_services(
        &self,
        project_slug: &str,
        filters: Option<&ServiceListFilter>,
    ) -> ApiResult<Vec<Service>> {
        let endpoint = format!("/api/project/{}/services/", project_slug);
        let (response_body, status) = match filters {
            Some(query) => self.get_with_query_params(&endpoint, query)?,
            None => self.get(&endpoint)?,
        };
        debug!("server response: {}", response_body);
        let projects: Vec<Service> = deserialize_body(&response_body, status)?;
        Ok(projects)
    }

    pub fn create_service(
        &self,
        service: &CreateServiceRequest,
        project_slug: &str,
    ) -> ApiResult<CreateServiceResponse> {
        let (response, status) = self.post(
            &format!("/api/project/{}/services/", project_slug),
            Some(service),
        )?;
        let service: CreateServiceResponse = deserialize_body(&response, status)?;
        Ok(service)
    }

    pub fn launch_worker(
        &self,
        worker: &LaunchWorkerRequest,
        service_slug: &str,
    ) -> ApiResult<LaunchWorkerResponse> {
        let (response, status) = self.post(
            &format!("/api/service/{}/build", service_slug),
            Some(worker),
        )?;
        let worker: LaunchWorkerResponse = deserialize_body(&response, status)?;
        Ok(worker)
    }

    pub fn get_worker_details(&self, worker_slug: &str) -> ApiResult<Worker> {
        let (response, status) = self.get(&format!("/api/worker/{}", worker_slug))?;
        let worker: Worker = deserialize_body(&response, status)?;
        Ok(worker)
    }

    pub fn deploy_service(
        &self,
        service_slug: &str,
        payload: &ServiceDeployRequest,
    ) -> ApiResult<ServiceDeployResponse> {
        let (response, status) = self.post(
            &format!("/api/service/{}/deploy", service_slug),
            Some(payload),
        )?;
        let deployment: ServiceDeployResponse = deserialize_body(&response, status)?;
        Ok(deployment)
    }

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

#[derive(Debug, Deserialize)]
struct APIError {
    detail: String,
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

#[derive(Debug, Serialize)]
pub struct EnvListFilters {
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProjectRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectResponse {
    pub project: Project,
    pub log: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectListFilters {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateServiceRequest {
    pub name: String,
    pub has_web_interface: bool,
    pub default_dockerfile_path: String,
    pub subdomain: String,
    pub container_port: String,
    pub alb_port_http: String,
    pub alb_port_https: String,
    pub health_check_endpoint: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateServiceResponse {
    pub service: Service,
    pub log: String,
}

#[derive(Debug, Serialize)]
pub struct ServiceListFilter {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CanCreateServiceResponse {
    pub can_create: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LaunchWorkerRequest {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct LaunchWorkerResponse {
    pub build: String,
    pub log: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ServiceDeployRequest {
    pub version: String,
}

#[derive(Debug, Deserialize)]
pub struct ServiceDeployResponse {
    pub deployment: String,
    pub log: String,
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
    pub last_modified: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteEnvironmentVariableRequest {
    pub key_name: String,
}
