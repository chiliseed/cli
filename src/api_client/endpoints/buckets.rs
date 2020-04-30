use serde::{Deserialize, Serialize};

use crate::api_client::types::ApiResult;
use crate::api_client::utils::deserialize_body;
use crate::api_client::ApiClient;

impl ApiClient {
    pub fn create_statics_bucket(
        &self,
        service_slug: &str,
        params: &CreateStaticsBucketRequest,
    ) -> ApiResult<CreateStaticsBucketResponse> {
        let (response, status) = self.post(
            &format!("/api/service/{}/add-statics-bucket", service_slug),
            Some(params),
        )?;
        let bucket: CreateStaticsBucketResponse = deserialize_body(&response, status)?;
        Ok(bucket)
    }
}

#[derive(Debug, Serialize)]
pub struct CreateStaticsBucketRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateStaticsBucketResponse {
    pub log: String,
    pub resource: String,
}
