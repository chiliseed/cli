use std::process::{exit, Command};
use std::thread::sleep;
use std::time::Duration;

use crate::api_client::ApiClient;
use crate::db::get_db;
use crate::environments::get_env;
use crate::projects::{get_env_name, get_project, get_project_name};
use crate::schemas::{Env, Project, Resource, Service};
use crate::services::{get_service, get_service_name};
use prettytable::{format, Cell, Row, Table};

const WAIT_TIME_SECS: u64 = 10;
const WAIT_SERVER_TIMEOUT_MINUTES: u64 = 30;

pub fn await_exec_result(
    api_client: &ApiClient,
    run_slug: &str,
    timeout_minutes: Option<u64>,
) -> bool {
    let timeout_minutes = timeout_minutes.unwrap_or(WAIT_SERVER_TIMEOUT_MINUTES);
    let mut waited = 0;
    loop {
        if waited >= timeout_minutes * 60 {
            eprintln!("TIMING OUT after 30 minutes. Please contact support for help");
            return false;
        }

        println!("Checking create status");

        match api_client.get_exec_log(&run_slug) {
            Ok(exec_log) => {
                debug!("{:?}", exec_log);

                if let Some(success) = exec_log.is_success {
                    if success {
                        println!("Infra is ready after {}s", waited);
                        return true;
                    }
                    println!("ERROR creating infra after {}s", waited);
                    return false;
                }

                sleep(Duration::from_secs(WAIT_TIME_SECS));
                waited += WAIT_TIME_SECS;
                println!("Still creating [{}s]", waited);
                continue;
            }
            Err(_err) => {
                eprintln!("Error checking status");
                return false;
            }
        }
    }
}

/// Wrapper for executing any commands in command line and get the output for
/// further processing
pub fn exec_command_with_output(
    cmd: &str,
    args: Vec<&str>,
) -> Result<(bool, Vec<u8>), &'static str> {
    debug!("{} {:?}", cmd, args);
    let output = Command::new(cmd)
        .args(&args)
        .output()
        .expect("Failed to execute command");
    Ok((output.status.success(), output.stdout))
}

/// Construct pretty table object, to be used for outputting data structures
/// in table format.
pub fn get_output_table() -> Table {
    let mut table = Table::new();
    let format = format::FormatBuilder::new().column_separator('\t').build();
    table.set_format(format);
    table
}

/// Utility to add rows to pretty table.
///
/// # Example
/// ```
/// let mut table = get_output_table();
/// add_row_to_output_table(&mut table, vec!["Name", "Foobar"]);
/// table.printstd();
/// ```
pub fn add_row_to_output_table(table: &mut Table, values: Vec<&str>) {
    let mut cells: Vec<Cell> = Vec::new();
    for v in values {
        cells.push(Cell::new(v));
    }
    table.add_row(Row::new(cells));
}

pub fn get_environment_or_exit(api_client: &ApiClient, environment_name: Option<String>) -> Env {
    let env_name = get_env_name(environment_name);
    match get_env(api_client, &env_name) {
        Ok(env) => env,
        Err(err) => {
            eprintln!("Error getting environment: {}", err);
            exit(1);
        }
    }
}

pub fn get_project_or_exit(
    api_client: &ApiClient,
    project_name: Option<String>,
    env_slug: &str,
) -> Project {
    let project_name = get_project_name(project_name);
    match get_project(&api_client, env_slug, &project_name) {
        Ok(project) => project,
        Err(err) => {
            eprintln!("Error getting project: {}", err);
            exit(1);
        }
    }
}

pub fn get_service_or_exit(
    api_client: &ApiClient,
    project: &Project,
    service_name: Option<String>,
) -> Service {
    let service_name = get_service_name(service_name);
    get_service(&api_client, &project, &service_name)
}

pub fn get_resource_or_exit(
    api_client: &ApiClient,
    project: &Project,
    db_identifier: String,
) -> Resource {
    get_db(api_client, &project.slug, db_identifier)
}
