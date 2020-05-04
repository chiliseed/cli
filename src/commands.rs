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
    /// Chiliseed user email
    #[structopt(short, long, env = "CHILISEED_USERNAME")]
    pub username: String,
    /// Chiliseed user password
    #[structopt(short, long, env = "CHILISEED_PASSWORD", hide_env_values = true)]
    pub password: String,
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

    #[structopt(
        name = "env_vars",
        about = "Management commands for service environment variables"
    )]
    EnvVar {
        /// Name of the environment hosting the project
        #[structopt(short, long = "environment")]
        environment_name: Option<String>,
        /// Name of the project hosting the service
        #[structopt(short, long = "project")]
        project_name: Option<String>,
        /// Name of the service for which to add the environment variable
        #[structopt(short, long = "service")]
        service_name: Option<String>,
        #[structopt(subcommand)]
        cmd: EnvVarSubCommands,
    },

    #[structopt(
        name = "db",
        about = "Management commands for databases in your environments"
    )]
    Db {
        /// Name of the environment hosting database(s)
        #[structopt(short, long = "environment")]
        environment_name: Option<String>,
        /// Name of the project for which to add the database
        #[structopt(short, long = "project")]
        project_name: Option<String>,
        #[structopt(subcommand)]
        cmd: DbSubCommands,
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
    /// Register new service and created it's infrastructure
    Create {},
    /// Deploy new version of the service
    Deploy {
        /// Name of the service to deploy
        service_name: String,
    },
    /// Update service parameters
    Update {
        /// Name of the service to update
        service_name: String,
    },
    /// Add a bucket for your static files (public read permission)
    AddStatics {
        /// Name of the service to which you want to add a bucket for static files
        service_name: String,
    },
}

#[derive(Debug, StructOpt)]
pub enum EnvVarSubCommands {
    /// Create new environment variable for a service
    Create {
        /// Environment variable name. Example: API_KEY
        key_name: String,
        /// Environment variable value. Example: some-api-key
        key_value: String,
    },

    /// List environment variables for the service
    List {},
}

#[derive(Debug, StructOpt)]
pub enum DbSubCommands {
    /// Create new database in environment
    Create {},
}
