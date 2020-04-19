use crate::api_client::schemas;
use crate::api_client::types::ApiResult;
use crate::api_client::utils::deserialize_body;
use crate::api_client::ApiClient;
use crate::schemas::Worker;

impl ApiClient {
    pub fn launch_worker(
        &self,
        worker: &schemas::LaunchWorkerRequest,
        service_slug: &str,
    ) -> ApiResult<schemas::LaunchWorkerResponse> {
        let (response, status) = self.post(
            &format!("/api/service/{}/build", service_slug),
            Some(worker),
        )?;
        let worker: schemas::LaunchWorkerResponse = deserialize_body(&response, status)?;
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
        payload: &schemas::ServiceDeployRequest,
    ) -> ApiResult<schemas::ServiceDeployResponse> {
        let (response, status) = self.post(
            &format!("/api/service/{}/deploy", service_slug),
            Some(payload),
        )?;
        let deployment: schemas::ServiceDeployResponse = deserialize_body(&response, status)?;
        Ok(deployment)
    }
}
