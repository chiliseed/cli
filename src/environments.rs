use std::error::Error;
use std::fmt;

use rusoto_core::Region;
use rusoto_credential::{
    AwsCredentials, CredentialsError, EnvironmentProvider, ProvideAwsCredentials,
};
use text_io::read;
use tokio;

use crate::api_client::{ApiClient, CreateEnvRequest, EnvListFilters};
use crate::schemas::Env;
use crate::utils::{add_row_to_output_table, await_exec_result, get_output_table};

#[derive(Debug)]
pub enum EnvError {
    EnvNotFound(String),
    ErrorGettingEnv(String),
}

impl Error for EnvError {}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EnvError::EnvNotFound(ref cause) => write!(f, "{}", cause),
            EnvError::ErrorGettingEnv(ref cause) => write!(f, "{}", cause),
        }
    }
}

pub type EnvResult<T> = Result<T, EnvError>;

#[tokio::main]
async fn get_aws_credentials() -> Result<AwsCredentials, CredentialsError> {
    EnvironmentProvider::default().credentials().await
}

pub fn add(api_client: &ApiClient, name: Option<String>, domain: Option<String>) {
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
        _ => {}
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
            await_exec_result(api_client, &resp.log, None);
        }
        Err(err) => {
            eprintln!("Error creating environment: {}", err.to_string());
        }
    }
}

pub fn list_envs(api_client: &ApiClient) {
    match api_client.list_envs(None) {
        Ok(envs) => {
            if envs.is_empty() {
                println!("You have not created any environments yet.");
                return;
            }
            println!("Your environments: ");
            for env in envs {
                println!();
                println!("{}", env.name);
                println!("{}", std::iter::repeat("=").take(60).collect::<String>());

                let mut table = get_output_table();
                add_row_to_output_table(&mut table, vec!["AWS Region", env.region.as_str()]);
                add_row_to_output_table(&mut table, vec!["Domain", env.domain.as_str()]);
                add_row_to_output_table(
                    &mut table,
                    vec!["Created at", env.created_at.to_rfc2822().as_str()],
                );
                add_row_to_output_table(
                    &mut table,
                    vec!["Status", env.last_status.status.as_str()],
                );
                table.printstd();
            }
        }

        Err(_err) => eprintln!("Error getting envs"),
    }
}

pub fn get_env(api_client: &ApiClient, env_name: &str) -> EnvResult<Env> {
    match api_client.list_envs(Some(&EnvListFilters {
        name: Some(env_name.to_string()),
    })) {
        Ok(envs) => {
            if envs.is_empty() {
                println!("Environment not found. Please check the name and try again.");
                return Err(EnvError::EnvNotFound(format!(
                    "Environment {} not found",
                    env_name
                )));
            }
            return Ok(envs[0].clone());
        }

        Err(_err) => Err(EnvError::ErrorGettingEnv("Error getting envs".to_string())),
    }
}
