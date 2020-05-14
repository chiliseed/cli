use crate::api_client::{
    ApiClient, CreateEnvironmentVariableRequest, DeleteEnvironmentVariableRequest,
};
use crate::schemas::Service;
use crate::utils::{add_row_to_output_table, get_output_table};

pub fn create(api_client: &ApiClient, service: Service, key_name: &str, key_value: &str) -> bool {
    match api_client.create_env_var(
        &service.slug,
        &CreateEnvironmentVariableRequest {
            key_name: key_name.to_string(),
            key_value: key_value.to_string(),
        },
    ) {
        Ok(resp) => {
            println!("Created new environment variable: {}", resp.key_name);
            true
        }
        Err(err) => {
            debug!("Error: {}", err.to_string());
            eprintln!("Server error. Please try again later.");
            false
        }
    }
}

pub fn create_env_var_in_project(
    api_client: &ApiClient,
    project_slug: &str,
    key_name: &str,
    key_value: &str,
) -> bool {
    match api_client.create_env_var_in_project(
        &project_slug,
        &CreateEnvironmentVariableRequest {
            key_name: key_name.to_string(),
            key_value: key_value.to_string(),
        },
    ) {
        Ok(resp) => {
            println!("Created new environment variables: ");
            for key in resp {
                println!("{}", key);
            }
            true
        }
        Err(err) => {
            debug!("Error: {}", err.to_string());
            false
        }
    }
}

pub fn list(api_client: &ApiClient, service: Service) {
    match api_client.list_env_vars(&service.slug) {
        Ok(env_vars) => {
            if env_vars.is_empty() {
                println!("Service {} has no environment variables.", service.name);
                return;
            }

            let mut table = get_output_table();
            for env_var in env_vars {
                add_row_to_output_table(
                    &mut table,
                    vec![&env_var.name, &env_var.value_from, &env_var.value],
                );
            }
            table.printstd();
        }

        Err(err) => {
            eprintln!("Error: {}", err.to_string());
        }
    }
}

pub fn delete_env_var(api_client: &ApiClient, service_slug: &str, key_name: &str) -> bool {
    let params = DeleteEnvironmentVariableRequest {
        key_name: key_name.to_string(),
    };
    match api_client.delete_env_var(service_slug, &params) {
        Ok(()) => true,
        Err(err) => {
            debug!("Server error: {}", err);
            false
        }
    }
}
