use text_io::read;

use crate::client::{APIClient, EnvListFilters, ProjectListFilters, ProjectRequest};
use crate::schemas::Env;
use crate::utils::await_exec_result;

pub fn get_environment(env_name: Option<String>) -> String {
    let environment_name = env_name.unwrap_or_else(|| {
        println!("Environment name: ");
        read!()
    });
    environment_name
}

fn get_env_and_then<F>(api_client: &APIClient, env_name: &str, callback: F)
where
    F: Fn(&Env),
{
    match api_client.list_envs(Some(&EnvListFilters {
        name: Some(env_name.to_string()),
    })) {
        Ok(envs) => {
            if envs.is_empty() {
                println!("Environment not found. Please check the name and try again.");
                return;
            }
            callback(&envs[0])
        }

        Err(_err) => eprintln!("Error getting envs"),
    }
}

pub fn list_projects(api_client: &APIClient, env_name: String) {
    get_env_and_then(api_client, &env_name, |env| {
        debug!("Getting projects for environment: \n{:?}", env);
        match api_client.list_projects(&env.slug, None) {
            Ok(projects) => {
                if projects.is_empty() {
                    println!("Environment {} has no projects yet.", env_name);
                    return;
                }
                println!("Environment {} has following projects: ", env_name);
                for project in projects {
                    println!("{:?}", project);
                }
            }

            Err(_err) => eprintln!("Error getting projects"),
        }
    })
}

pub fn create_project(api_client: &APIClient, env_name: String, project_name: Option<String>) {
    let p_name = project_name.unwrap_or_else(|| {
        println!("Project name: ");
        read!()
    });

    get_env_and_then(api_client, &env_name, move |env| {
        let is_project_exist = match api_client.list_projects(
            &env.slug,
            Some(&ProjectListFilters {
                name: p_name.clone(),
            }),
        ) {
            Ok(projects) => !projects.is_empty(),
            Err(_) => return,
        };

        if is_project_exist {
            println!("Project with this name already exists. Please choose another name.");
            return;
        }

        let run_slug = match api_client.create_project(
            &ProjectRequest {
                name: p_name.clone(),
            },
            &env.slug,
        ) {
            Ok(resp) => resp.log,
            Err(_) => return,
        };

        println!("Launching project infra: {}", p_name);
        await_exec_result(api_client, &run_slug)
    })
}
