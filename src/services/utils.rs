use text_io::read;

use super::types::{ServiceError, ServiceResult};
use crate::client::{APIClient, ServiceListFilter};
use crate::environments::get_env;
use crate::projects::get_project;
use crate::schemas::Service;

pub fn get_services(
    api_client: &APIClient,
    env_name: &str,
    project_name: &str,
    service_name: Option<String>,
) -> ServiceResult<Vec<Service>> {
    let env = get_env(api_client, env_name)?;
    let project = get_project(api_client, &env.slug, project_name)?;
    let services = match service_name {
        Some(name) => api_client.list_services(&project.slug, Some(&ServiceListFilter { name }))?,
        None => api_client.list_services(&project.slug, None)?,
    };
    debug!("services are {:?}", services);
    if services.is_empty() {
        Err(ServiceError::ServicesNotFound(
            "No services found".to_string(),
        ))
    } else {
        Ok(services)
    }
}

pub fn get_service_name(maybe_service_name: Option<String>) -> String {
    maybe_service_name.unwrap_or_else(|| {
        println!("Service name: ");
        read!()
    })
}
