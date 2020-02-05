mod client;
mod commands;

use std::process::exit;

#[macro_use]
extern crate log;
use structopt::StructOpt;

use crate::commands::{Command, EnvSubCommands, Opt};
use client::APIClient;

fn main() {
    pretty_env_logger::try_init_custom_env("CHILISEED_LOG")
        .expect("Cannot initialize the logger that was already initialized.");

    info!("Firing up chiliseed CLI");
    let api_client = match APIClient::new() {
        Ok(c) => c,
        Err(err) => {
            error!("{}", err);
            exit(1)
        }
    };
    let args = Opt::from_args();

    match args.cmd {
        Command::Environment { cmd } => match cmd {
            EnvSubCommands::List {} => {
                info!("Getting a list of environments");
                match api_client.list_envs() {
                    Ok(envs) => {
                        info!("{:?}", envs);
                    }
                    Err(err) => {
                        error!("Failed to list environments: {}", err);
                    }
                }
            }
        },
    }
}
