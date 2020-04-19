use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

use crate::api_client::ApiClient;

const WAIT_TIME_SECS: u64 = 10;

pub fn await_exec_result(api_client: &ApiClient, run_slug: &str) -> bool {
    let timeout_minutes = 30;
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

// Wrapper for executing commands in user shell. All output is sent to user shell
// pub fn exec_command(cmd: &str, args: Vec<&str>) -> bool {
//     println!("{} {:?}", cmd, args);
//     let mut cli_command = match Command::new(cmd)
//         .args(&args)
//         .stdin(Stdio::inherit())
//         .stdout(Stdio::inherit())
//         .stderr(Stdio::inherit())
//         .spawn()
//     {
//         Err(err) => panic!("Error spawning: {}", err.description()),
//         Ok(process) => process,
//     };
//
//     cli_command.wait().unwrap().success()
// }
