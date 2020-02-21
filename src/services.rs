use crate::client::APIClient;
use crate::environments::get_env;
use crate::projects::get_project;

pub fn list_services(api_client: &APIClient, env_name: &str, project_name: &str) {
    let env = match get_env(api_client, env_name) {
        Ok(e) => e,
        Err(err) => {
            eprintln!("Error getting environment: {}", err);
            return;
        }
    };
    let project = match get_project(api_client, &env.slug, project_name) {
        Ok(p) => p,
        Err(err) => {
            eprintln!("Error getting project: {}", err);
            return;
        }
    };

    match api_client.list_services(&project.slug) {
        Ok(services) => {
            if services.is_empty() {
                println!(
                    "Project {} ({}) has no services yet.",
                    project.name, env.name
                );
                return;
            }
            println!("Project {} has following services: ", project.name);
            for service in services {
                println!("{:?}", service);
            }
        }
        Err(_) => eprintln!("Error getting services"),
    }
}
