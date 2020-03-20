use crate::client::{APIClient, CreateEnvironmentVariableRequest};
use crate::services::get_services;

pub fn create(
    api_client: &APIClient,
    env_name: &str,
    project_name: &str,
    service_name: &str,
    key_name: &str,
    key_value: &str,
) {
    let service = match get_services(
        api_client,
        env_name,
        project_name,
        Some(service_name.to_string()),
    ) {
        Ok(services) => services[0].clone(),
        Err(err) => {
            debug!("Error: {}", err.to_string());
            eprintln!("Service not found. Please check service name and try again.");
            return;
        }
    };

    match api_client.create_env_var(
        &service.slug,
        &CreateEnvironmentVariableRequest {
            key_name: key_name.to_string(),
            key_value: key_value.to_string(),
        },
    ) {
        Ok(resp) => println!("Created new environment variable: {}", resp.key_name),
        Err(err) => {
            debug!("Error: {}", err.to_string());
        }
    }
}
