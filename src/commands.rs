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
pub struct Opt {
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Environment sub commands
    #[structopt(name = "environment", about = "Management commands for environments")]
    Environment {
        #[structopt(subcommand)]
        cmd: EnvSubCommands,
    },
    #[structopt(name = "project", about = "Management commands for projects")]
    Project {
        /// Name of the environment hosting the project(s)
        #[structopt(short, long = "environment")]
        environment_name: Option<String>,
        #[structopt(subcommand)]
        cmd: ProjectSubCommands,
    },

    #[structopt(name = "service", about = "Management commands for services")]
    Service {
        /// Name of the environment hosting the project
        #[structopt(short, long = "environment")]
        environment_name: Option<String>,
        /// Name of the project to which the service(s) is(are) related
        #[structopt(short, long = "project")]
        project_name: Option<String>,
        #[structopt(subcommand)]
        cmd: ServiceSubCommands,
    },
}

#[derive(Debug, StructOpt)]
pub enum EnvSubCommands {
    #[structopt(name = "list", about = "List all environments.")]
    List {},
    /// Create new environment
    Create {
        name: Option<String>,
        domain: Option<String>,
    },
}

#[derive(Debug, StructOpt)]
pub enum ProjectSubCommands {
    /// List projects for environment
    List {},
    Create {
        /// Name for the project infra to create. Example: backend
        name: Option<String>,
    },
}

#[derive(Debug, StructOpt)]
pub enum ServiceSubCommands {
    /// List services for project in environment
    List {},
    // Create {
    //     /// Name of the service for which to create the infra. Example: api
    //     name: Option<String>,
    // },
}

// #[derive(Debug, StructOpt)]
// pub enum ServiceSubCommand {
//     #[structopt(name = "add", about = "Add new service in cluster")]
//     Add {
//         /// The name of the service to add
//         service_name: String,
//         /// The name of the cluster in which to launch the service
//         cluster_name: String,
//         /// Port on which this service will listen to traffic
//         #[structopt(short, long)]
//         port: Option<i64>,
//     },
// }
