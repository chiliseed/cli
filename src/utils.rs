use std::thread::sleep;
use std::time::Duration;

use crate::client::APIClient;

pub fn await_exec_result(api_client: &APIClient, run_slug: &str) {
    let timeout_minutes = 30;
    let mut waited = 0;
    loop {
        if waited >= timeout_minutes * 60 {
            eprintln!("TIMING OUT after 30 minutes. Please contact support for help");
            return;
        }

        sleep(Duration::from_secs(30));
        waited += 30;

        println!("Checking create status");

        match api_client.get_exec_log(&run_slug) {
            Ok(exec_log) => {
                debug!("{:?}", exec_log);

                if let Some(success) = exec_log.is_success {
                    if success {
                        println!("Infra is ready after {}s", waited);
                    } else {
                        println!("ERROR creating infra after {}s", waited);
                    }
                    return;
                }

                println!("Still creating [{}s]", waited);
                continue;
            }
            Err(_err) => {
                eprintln!("Error checking status");
                return;
            }
        }
    }
}
