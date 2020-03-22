use crate::client::{ApiClient, CreateEnvironmentVariableRequest};
use crate::services::get_services;

pub fn create(
    api_client: &ApiClient,
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

pub fn list(api_client: &ApiClient, env_name: &str, project_name: &str, service_name: &str) {
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

    match api_client.list_env_vars(&service.slug) {
        Ok(env_vars) => {
            if env_vars.is_empty() {
                println!("Service {} has no environment variables.", service_name);
                return;
            }

            for env_var in env_vars {
                println!("{}", std::iter::repeat("=").take(60).collect::<String>());
                println!(
                    "Name {} {}",
                    std::iter::repeat(" ").take(5).collect::<String>(),
                    env_var.name
                );
                println!(
                    "Key path {} {}",
                    std::iter::repeat(" ").take(1).collect::<String>(),
                    env_var.value_from
                );
            }
        }

        Err(err) => {
            eprintln!("Error: {}", err.to_string());
        }
    }
}
