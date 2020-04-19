use crate::api_client::schemas;
use crate::api_client::types::ApiResult;
use crate::api_client::utils::deserialize_body;
use crate::api_client::ApiClient;
use crate::schemas::Service;

impl ApiClient {
    pub fn list_services(
        &self,
        project_slug: &str,
        filters: Option<&schemas::ServiceListFilter>,
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
        service: &schemas::CreateServiceRequest,
        project_slug: &str,
    ) -> ApiResult<schemas::CreateServiceResponse> {
        let (response, status) = self.post(
            &format!("/api/project/{}/services/", project_slug),
            Some(service),
        )?;
        let service: schemas::CreateServiceResponse = deserialize_body(&response, status)?;
        Ok(service)
    }
}
