mod client;
mod commands;
mod env_vars;
mod environments;
mod projects;
mod schemas;
mod services;
mod utils;

use std::process::exit;

#[macro_use]
extern crate log;
use structopt::StructOpt;

use crate::commands::EnvVarSubCommands;
use client::ApiClient;
use commands::{Command, EnvSubCommands, Opt, ProjectSubCommands, ServiceSubCommands};

fn main() {
    pretty_env_logger::try_init_custom_env("CHILISEED_LOG")
        .expect("Cannot initialize the logger that was already initialized.");

    info!("Firing up chiliseed CLI");
    let args = Opt::from_args();

    let api_client = match ApiClient::new() {
        Ok(c) => c,
        Err(err) => {
            error!("{}", err);
            exit(1)
        }
    };

    match args.cmd {
        Command::Environment { cmd } => match cmd {
            EnvSubCommands::List {} => {
                info!("Getting your environments");
                environments::list_envs(&api_client);
            }

            EnvSubCommands::Create { name, domain } => {
                info!("Creating new environment");
                environments::add(&api_client, name, domain);
            }
        },

        Command::Project {
            environment_name,
            cmd,
        } => match cmd {
            ProjectSubCommands::List {} => {
                info!("Getting list of project");
                let env_name = projects::get_env_name(environment_name);
                projects::list_projects(&api_client, &env_name);
            }

            ProjectSubCommands::Create { name } => {
                info!("Creating project");
                let env_name = projects::get_env_name(environment_name);
                projects::create_project(&api_client, &env_name, name);
            }
        },

        Command::Service {
            environment_name,
            project_name,
            cmd,
        } => match cmd {
            ServiceSubCommands::List {} => {
                info!("Getting services for project");
                let env_name = projects::get_env_name(environment_name);
                let project_name = projects::get_project_name(project_name);
                services::list_services(&api_client, &env_name, &project_name);
            }

            ServiceSubCommands::Create {} => {
                let env_name = projects::get_env_name(environment_name);
                let project_name = projects::get_project_name(project_name);
                info!(
                    "Creating service for project: {}({})",
                    project_name, env_name
                );
                services::create_service(&api_client, &env_name, &project_name);
            }

            ServiceSubCommands::Deploy { service_name } => {
                let env_name = projects::get_env_name(environment_name);
                let project_name = projects::get_project_name(project_name);
                info!("Deploying service: {}", service_name);
                services::deploy(&api_client, &env_name, &project_name, &service_name);
            }
        },

        Command::EnvVar {
            environment_name,
            project_name,
            service_name,
            cmd,
        } => match cmd {
            EnvVarSubCommands::Create {
                key_name,
                key_value,
            } => {
                let env_name = projects::get_env_name(environment_name);
                let project_name = projects::get_project_name(project_name);
                let service_name = services::get_service_name(service_name);
                info!("Creating new environment variable: {}", key_name);
                env_vars::create(
                    &api_client,
                    &env_name,
                    &project_name,
                    &service_name,
                    &key_name,
                    &key_value,
                );
            }

            EnvVarSubCommands::List {} => {
                let env_name = projects::get_env_name(environment_name);
                let project_name = projects::get_project_name(project_name);
                let service_name = services::get_service_name(service_name);
                info!("Listing environment variables for service: {} in project: {} in environment: {}", service_name, project_name, env_name);
                env_vars::list(&api_client, &env_name, &project_name, &service_name);
            }
        },
    }
}
