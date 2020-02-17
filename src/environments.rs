use rusoto_core::Region;
use rusoto_credential::{
    AwsCredentials, CredentialsError, EnvironmentProvider, ProvideAwsCredentials,
};
use text_io::read;
use tokio;

use crate::client::{APIClient, CreateEnvRequest};
use crate::utils::await_exec_result;

#[tokio::main]
async fn get_aws_credentials() -> Result<AwsCredentials, CredentialsError> {
    EnvironmentProvider::default().credentials().await
}

pub fn add(api_client: &APIClient, name: Option<String>, domain: Option<String>) {
    let env_name = name.unwrap_or_else(|| {
        println!("Environment name: ");
        read!()
    });

    let env_domain = domain.unwrap_or_else(|| {
        println!("Environment domain: ");
        read!()
    });

    let creds = match get_aws_credentials() {
        Ok(val) => val,
        Err(err) => {
            eprintln!("ERROR: {}", err.message);
            return;
        }
    };

    let req = CreateEnvRequest {
        name: env_name.to_owned(),
        domain: env_domain.to_string(),
        region: Region::default().name().to_string(),
        access_key: creds.aws_access_key_id().to_string(),
        access_key_secret: creds.aws_secret_access_key().to_string(),
    };

    match api_client.create_env(&req) {
        Ok(resp) => await_exec_result(api_client, &resp.log),
        Err(err) => {
            eprintln!("Error creating environment: {}", err.to_string());
        }
    }
}

pub fn list_envs(api_client: &APIClient) {
    match api_client.list_envs(None) {
        Ok(envs) => {
            if envs.is_empty() {
                println!("You have not created any environments yet.");
                return;
            }
            println!("Your environments: ");
            for env in envs {
                println!("{:?}", env);
            }
        }

        Err(_err) => eprintln!("Error getting envs"),
    }
}
