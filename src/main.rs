mod client;
mod commands;
mod environments;

use std::process::exit;

#[macro_use]
extern crate log;
use structopt::StructOpt;

use client::APIClient;
use commands::{Command, EnvSubCommands, Opt};

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
                info!("Getting your environments");
                environments::list(&api_client).expect("Error listing environments");
            }
        },
    }
}
