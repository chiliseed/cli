use rusoto_core::Region;
use rusoto_credential::{AwsCredentials, EnvironmentProvider, ProvideAwsCredentials};
use text_io::read;
use tokio;

use crate::client::{APIClient, EnvRequest};

#[tokio::main]
async fn get_aws_credentials() -> AwsCredentials {
    EnvironmentProvider::default().credentials().await.unwrap()
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

    let creds = get_aws_credentials();

    let req = EnvRequest {
        name: env_name.to_owned(),
        domain: env_domain.to_string(),
        region: Region::default().name().to_string(),
        access_key: creds.aws_access_key_id().to_string(),
        access_key_secret: creds.aws_secret_access_key().to_string(),
    };

    match api_client.create_env(&req) {
        Ok(env) => {
            println!("Launched environment: {}", env.name);
        }
        Err(err) => {
            eprintln!("Error creating environment: {}", err.to_string());
        }
    }
}

pub fn list(api_client: &APIClient) {
    match api_client.list_envs() {
        Ok(envs) => {
            if envs.is_empty() {
                println!("You have not created any environments yet.");
                return;
            }
            println!("Your environments: ");
            for env in envs {
                println!("{}", env.name);
            }
        }

        Err(_err) => eprintln!("Error getting envs"),
    }
}
