mod commands;
pub mod project;
mod service;

#[macro_use]
extern crate log;
use structopt::StructOpt;

use crate::commands::{Command, Opt, ServiceSubCommand};
use crate::project::Project;
use crate::service::AddECSServiceParams;

fn main() {
    pretty_env_logger::try_init_custom_env("CHILISEED_LOG")
        .expect("Cannot initialize the logger that was already initialized.");

    info!("Firing up chiliseed CLI");

    let args = Opt::from_args();
    let project = Project {
        name: args.project_name,
        env_name: args.environment,
    };

    match args.cmd {
        Command::Service { cmd } => match cmd {
            ServiceSubCommand::Add {
                service_name,
                cluster_name,
                port,
            } => {
                let params = AddECSServiceParams {
                    project,
                    cluster: cluster_name,
                    port,
                    service_name: String::from(service_name.as_str()),
                };

                match service::setup_deployment(params) {
                    Ok(()) => info!("Service ready to deploy: {}", String::from(service_name)),
                    Err(e) => error!("Something went wrong: {:?}", e),
                }
            }
        },
    }
}
