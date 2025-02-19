use text_io::read;

use super::types::{ServiceError, ServiceResult};
use crate::api_client::{ApiClient, ServiceListFilter};
use crate::schemas::{Project, Service};
use std::process::exit;

pub fn get_services(
    api_client: &ApiClient,
    project: &Project,
    service_name: Option<String>,
) -> ServiceResult<Vec<Service>> {
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

pub fn get_service(api_client: &ApiClient, project: &Project, service_name: &str) -> Service {
    match get_services(api_client, project, Some(service_name.to_string())) {
        Ok(services) => services[0].clone(),
        Err(err) => {
            debug!("Error: {}", err.to_string());
            eprintln!("Service not found. Please check service name and try again.");
            exit(1);
        }
    }
}

pub(crate) fn ask_for_value(question: String, default: String) -> String {
    println!("\n{}", question);
    let new_value: String = read!("{}\n");
    if !new_value.is_empty() {
        new_value
    } else {
        default
    }
}

pub(crate) fn ask_yes_no(question: String, default: bool) -> bool {
    println!("\n{}", question);
    let yes: String = read!("{}\n");
    if yes.is_empty() {
        default
    } else {
        vec!["y", "ye", "yes", "yep"].contains(&yes.to_lowercase().as_str())
    }
}
