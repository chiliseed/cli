use structopt::StructOpt;

/// Chiliseed command line interface
/// AWS credentials are looked up first in environment variables with fallback
/// to config files in current directory and then in user directory
/// AWS_ACCESS_KEY_ID - for access key id
/// AWS_SECRET_ACCESS_KEY - for access secret key
/// AWS_SESSION_TOKEN - for session token, in case of mfa
/// AWS_CREDENTIAL_EXPIRATION
#[derive(Debug, StructOpt)]
#[structopt(name = "chiliseed", about = "Chiliseed command line interface")]
pub struct  Opt {
    /// The name of the project on which to take action.
    #[structopt(short = "p", long = "project", env = "CHILISEED_PROJECT_NAME")]
    pub project_name: String,

    /// Project environment on which to take action.
    #[structopt(short, long, env = "CHILISEED_ENVIRONMENT")]
    pub environment: String,

    #[structopt(subcommand)]
    pub cmd: Command
}


#[derive(Debug, StructOpt)]
pub enum Command {
    /// service sub commands
    #[structopt(name = "service", about = "Management commands for services")]
    Service {
        #[structopt(subcommand)]
        cmd: ServiceSubCommand
    }
}


#[derive(Debug, StructOpt)]
pub enum ServiceSubCommand {
    #[structopt(name = "add", about = "Add service")]
    Add {
        /// The name of the service to add
        service_name: String,
    }
}
