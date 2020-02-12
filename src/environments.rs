use std::thread::sleep;
use std::time::Duration;

use rusoto_core::Region;
use rusoto_credential::{
    AwsCredentials, CredentialsError, EnvironmentProvider, ProvideAwsCredentials,
};
use text_io::read;
use tokio;

use crate::client::{APIClient, CreateEnvRequest};

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
        Ok(resp) => {
            println!("Launching environment: {}", resp.env.name);
            let timeout_minutes = 30;
            let mut waited = 0;
            loop {
                if waited >= timeout_minutes * 60 {
                    eprintln!("TIMING OUT after 30 minutes. Please contact support for help");
                    return;
                }

                sleep(Duration::from_secs(30));
                waited += 30;

                println!("Checking create status");

                match api_client.get_exec_log(resp.log.clone()) {
                    Ok(exec_log) => {
                        debug!("{:?}", exec_log);

                        if exec_log.is_success {
                            println!("Environment is ready after {}s", waited);
                            return;
                        }

                        println!("Still creating [{}s]", waited);
                        continue;
                    }
                    Err(err) => {
                        eprintln!("Error checking status");
                        return;
                    }
                }
            }
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
                println!("{:?}", env);
            }
        }

        Err(_err) => eprintln!("Error getting envs"),
    }
}
