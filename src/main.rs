mod client;
mod commands;
mod environments;
mod projects;
mod schemas;

use std::process::exit;

#[macro_use]
extern crate log;
use structopt::StructOpt;

use client::APIClient;
use commands::{Command, EnvSubCommands, Opt, ProjectSubCommands};

fn main() {
    pretty_env_logger::try_init_custom_env("CHILISEED_LOG")
        .expect("Cannot initialize the logger that was already initialized.");

    info!("Firing up chiliseed CLI");
    let args = Opt::from_args();

    let api_client = match APIClient::new() {
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
                projects::list_projects(&api_client, environment_name);
            }
        },
    }
}
