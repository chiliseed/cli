use rusoto_core::{Region, RusotoError};
use rusoto_ssm::{PutParameterError, PutParameterRequest, PutParameterResult, Ssm, SsmClient};
// use tokio;

use crate::client::APIClient;
use crate::services::get_services;

// #[tokio::main]
// fn create_parameter(
//     key_path: &str,
//     key_value: &str,
// ) -> Result<PutParameterResult, RusotoError<PutParameterError>> {
//     let ssm_client = SsmClient::new(Region::default());
//     ssm_client
//         .put_parameter(PutParameterRequest {
//             allowed_pattern: None,
//             description: None,
//             key_id: None,
//             name: key_path.to_string(),
//             overwrite: Some(true),
//             policies: None,
//             tags: None,
//             tier: None,
//             type_: "SecureString".to_string(),
//             value: key_value.to_string(),
//         })
//         .await
// }

pub fn create(
    api_client: &APIClient,
    project_name: &str,
    env_name: &str,
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
}
