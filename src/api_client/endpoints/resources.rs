use serde::{Deserialize, Serialize};

use crate::api_client::types::ApiResult;
use crate::api_client::utils::deserialize_body;
use crate::api_client::ApiClient;
use crate::schemas::{Bucket, Resource};

impl ApiClient {
    pub fn get_resource_details(&self, resource_slug: &str) -> ApiResult<Resource> {
        let (response, status) = self.get(&format!("/api/resource/{}", resource_slug))?;
        let resource: Resource = deserialize_body(&response, status)?;
        Ok(resource)
    }

    pub fn get_bucket_details(&self, resource_slug: &str) -> ApiResult<Bucket> {
        let (response, status) = self.get(&format!("/api/resource/{}", resource_slug))?;
        let resource: Bucket = deserialize_body(&response, status)?;
        Ok(resource)
    }

    pub fn list_resources(
        &self,
        project_slug: &str,
        filters: Option<&ResourceListFilter>,
    ) -> ApiResult<Vec<Resource>> {
        let endpoint = format!("/api/project/{}/resources/", project_slug);
        let (response, status) = match filters {
            Some(query) => self.get_with_query_params(&endpoint, query)?,
            None => self.get(&endpoint)?,
        };
        let resources: Vec<Resource> = deserialize_body(&response, status)?;
        Ok(resources)
    }

    pub fn remove_statics_bucket(&self, service_slug: &str) -> ApiResult<ExecLog> {
        let (response, status) = self.post(
            &format!("/api/service/{}/remove-statics-bucket", service_slug),
            None::<&ResourceListFilter>,
        )?;
        let log: ExecLog = deserialize_body(&response, status)?;
        Ok(log)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResourceKind {
    Database,
    Cache,
    Bucket,
}

#[derive(Debug, Serialize)]
pub struct ResourceListFilter {
    pub kind: ResourceKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExecLog {
    pub log: String,
}
