mod api_client;
mod commands;
mod db;
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

use crate::commands::{DbSubCommands, EnvVarSubCommands};
use api_client::ApiClient;
use commands::{Command, EnvSubCommands, Opt, ProjectSubCommands, ServiceSubCommands};

fn main() {
    pretty_env_logger::try_init_custom_env("CHILISEED_LOG")
        .expect("Cannot initialize the logger that was already initialized.");

    info!("Firing up chiliseed CLI");
    let args = Opt::from_args();

    let api_client = match ApiClient::new(&args.username, &args.password) {
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
                let env = utils::get_environment_or_exit(&api_client, environment_name);
                let project = utils::get_project_or_exit(&api_client, project_name, &env.slug);
                services::list_services(&api_client, project);
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

            ServiceSubCommands::Update { service_name } => {
                let env = utils::get_environment_or_exit(&api_client, environment_name);
                let project = utils::get_project_or_exit(&api_client, project_name, &env.slug);
                let service = services::get_service(&api_client, &project, &service_name);
                info!("Updating service: {}", service.name);
                services::update(&api_client, project, service);
            }

            ServiceSubCommands::Deploy {
                service_name,
                build_arg,
            } => {
                let env = utils::get_environment_or_exit(&api_client, environment_name);
                let project = utils::get_project_or_exit(&api_client, project_name, &env.slug);
                let service = services::get_service(&api_client, &project, &service_name);
                info!("Deploying service: {}", service.name);
                services::deploy(&api_client, service, build_arg);
            }

            ServiceSubCommands::AddStatics { service_name } => {
                let env = utils::get_environment_or_exit(&api_client, environment_name);
                let project = utils::get_project_or_exit(&api_client, project_name, &env.slug);
                let service = services::get_service(&api_client, &project, &service_name);
                info!("Adding static files bucket to service: {}", service.name);
                services::add_statics(&api_client, service, project);
            }

            ServiceSubCommands::RemoveStatics { service_name } => {
                let env = utils::get_environment_or_exit(&api_client, environment_name);
                let project = utils::get_project_or_exit(&api_client, project_name, &env.slug);
                let service = services::get_service(&api_client, &project, &service_name);
                info!("Removing static files bucket to service: {}", service.name);
                services::remove_statics(&api_client, service);
            }

            ServiceSubCommands::AddDb {
                identifier,
                service_name,
            } => {
                let env = utils::get_environment_or_exit(&api_client, environment_name);
                let project = utils::get_project_or_exit(&api_client, project_name, &env.slug);
                let service = services::get_service(&api_client, &project, &service_name);
                let db = utils::get_resource_or_exit(&api_client, &project, identifier);
                info!("Adding db {} to service {}", db.identifier, service.name);
                services::add_database(&api_client, service, db);
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
                let env = utils::get_environment_or_exit(&api_client, environment_name);
                let project = utils::get_project_or_exit(&api_client, project_name, &env.slug);
                let service = utils::get_service_or_exit(&api_client, &project, service_name);
                info!("Creating new environment variable: {}", key_name);
                env_vars::create(&api_client, service, &key_name, &key_value);
            }

            EnvVarSubCommands::List {} => {
                let env = utils::get_environment_or_exit(&api_client, environment_name);
                let project = utils::get_project_or_exit(&api_client, project_name, &env.slug);
                let service = utils::get_service_or_exit(&api_client, &project, service_name);
                info!("Listing environment variables for service: {} in project: {} in environment: {}", service.name, project.name, env.name);
                env_vars::list(&api_client, service);
            }
        },

        Command::Db {
            environment_name,
            project_name,
            cmd,
        } => match cmd {
            DbSubCommands::Create {} => {
                let env_name = projects::get_env_name(environment_name);
                let project_name = projects::get_project_name(project_name);
                info!("Adding new database in {} environment", env_name);
                db::create_db(&api_client, &env_name, &project_name);
            }

            DbSubCommands::List {} => {
                let env = utils::get_environment_or_exit(&api_client, environment_name);
                let project = utils::get_project_or_exit(&api_client, project_name, &env.slug);
                info!(
                    "Listing databases in project {} ({})",
                    project.name, env.name
                );
                db::list_databases(&api_client, &project);
            }
        },
    }
}
