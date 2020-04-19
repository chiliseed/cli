use serde::{Deserialize, Serialize};

use crate::schemas::{Env, Project, Service};

#[derive(Debug, Deserialize)]
pub(crate) struct ApiError {
    pub(crate) detail: String,
}

#[derive(Serialize)]
pub(crate) struct LoginRequest {
    pub(crate) email: String,
    pub(crate) password: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginResponse {
    pub(crate) auth_token: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct LoginResponseError {
    pub(crate) non_field_errors: Vec<String>,
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
pub(crate) struct CreateEnvResponseError {
    pub(crate) name: Option<Vec<String>>,
    pub(crate) region: Option<Vec<String>>,
    pub(crate) domain: Option<Vec<String>>,
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
