use text_io::read;

use crate::client::{APIClient, EnvListFilters};

pub fn list_projects(api_client: &APIClient, env_name: Option<String>) {
    let environment_name = env_name.unwrap_or_else(|| {
        println!("Environment name: ");
        read!()
    });

    match api_client.list_envs(Some(&EnvListFilters {
        name: Some(environment_name.to_string()),
    })) {
        Ok(envs) => {
            if envs.is_empty() {
                println!("Environment not found. Please check the name and try again.");
                return;
            }

            debug!("Getting projects for environment: \n{:?}", envs[0]);

            match api_client.list_projects(&envs[0].slug) {
                Ok(projects) => {
                    if projects.is_empty() {
                        println!("Environment {} has no projects yet.", environment_name);
                        return;
                    }
                    println!("Environment {} has following projects: ", environment_name);
                    for project in projects {
                        println!("{:?}", project);
                    }
                }

                Err(_err) => eprintln!("Error getting projects"),
            }
        }

        Err(_err) => eprintln!("Error getting envs"),
    }
}
