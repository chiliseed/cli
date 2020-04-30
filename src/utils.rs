use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use crate::api_client::ApiClient;
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

        sleep(Duration::from_secs(WAIT_TIME_SECS));
        waited += WAIT_TIME_SECS;

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

pub fn get_output_table() -> Table {
    let mut table = Table::new();
    let format = format::FormatBuilder::new().column_separator('\t').build();
    table.set_format(format);
    table
}

pub fn add_row_to_output_table(table: &mut Table, values: Vec<&str>) {
    let mut cells: Vec<Cell> = Vec::new();
    for v in values {
        cells.push(Cell::new(v));
    }
    table.add_row(Row::new(cells));
}
