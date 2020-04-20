use serde::{Deserialize, Serialize};

use crate::api_client::types::ApiResult;
use crate::api_client::utils::deserialize_body;
use crate::api_client::ApiClient;
use crate::schemas::Project;

impl ApiClient {
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
