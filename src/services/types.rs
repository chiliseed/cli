use std::error::Error;
use std::{fmt, io};

use crate::api_client::ApiClientError;
use crate::environments::EnvError;
use crate::projects::ProjectError;

#[derive(Debug)]
pub enum ServiceError {
    EnvError(EnvError),
    ProjectError(ProjectError),
    ServicesNotFound(String),
    APIError(ApiClientError),
    DeploymentError(String),
}

pub type ServiceResult<T> = Result<T, ServiceError>;

impl Error for ServiceError {}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<EnvError> for ServiceError {
    fn from(err: EnvError) -> ServiceError {
        ServiceError::EnvError(err)
    }
}

impl From<ApiClientError> for ServiceError {
    fn from(err: ApiClientError) -> ServiceError {
        ServiceError::APIError(err)
    }
}

impl From<ProjectError> for ServiceError {
    fn from(err: ProjectError) -> ServiceError {
        ServiceError::ProjectError(err)
    }
}

impl From<io::Error> for ServiceError {
    fn from(err: io::Error) -> ServiceError {
        ServiceError::DeploymentError(err.to_string())
    }
}

impl From<globset::Error> for ServiceError {
    fn from(err: globset::Error) -> ServiceError {
        ServiceError::DeploymentError(err.to_string())
    }
}

impl From<ssh2::Error> for ServiceError {
    fn from(err: ssh2::Error) -> ServiceError {
        ServiceError::DeploymentError(err.to_string())
    }
}
