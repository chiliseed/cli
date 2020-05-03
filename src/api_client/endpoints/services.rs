use serde::{Deserialize, Serialize};

use crate::api_client::types::ApiResult;
use crate::api_client::utils::deserialize_body;
use crate::api_client::ApiClient;
use crate::schemas::Service;

impl ApiClient {
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
}

#[derive(Debug, Serialize)]
pub struct CreateServiceRequest {
    pub name: String,
    pub has_web_interface: bool,
    pub default_dockerfile_path: String,
    pub default_dockerfile_target: Option<String>,
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
