use std::error::Error;
use std::fmt;

use text_io::read;

use crate::client::{APIClient, EnvListFilters, ProjectListFilters, ProjectRequest};
use crate::schemas::{Env, Project};
use crate::utils::await_exec_result;

#[derive(Debug)]
pub enum ProjectError {
    ProjectNotFound(String),
    ErrorGettingProject(String),
}

impl Error for ProjectError {}

impl fmt::Display for ProjectError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub type ProjectResult<T> = Result<T, ProjectError>;

pub fn get_env_name(env_name: Option<String>) -> String {
    env_name.unwrap_or_else(|| {
        println!("Environment name: ");
        read!()
    })
}

pub fn get_project_name(maybe_project_name: Option<String>) -> String {
    maybe_project_name.unwrap_or_else(|| {
        println!("Project name: ");
        read!()
    })
}

pub fn get_env_and_then<F>(api_client: &APIClient, env_name: &str, callback: F)
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

pub fn list_projects(api_client: &APIClient, env_name: &str) {
    get_env_and_then(api_client, env_name, |env| {
        debug!("Getting projects for environment: \n{:?}", env);
        match api_client.list_projects(&env.slug, None) {
            Ok(projects) => {
                if projects.is_empty() {
                    println!("Environment {} has no projects yet.", env.name);
                    return;
                }
                println!("Environment {} has following projects: ", env.name);
                for project in projects {
                    println!("{:?}", project);
                }
            }

            Err(_err) => eprintln!("Error getting projects"),
        }
    })
}

pub fn create_project(api_client: &APIClient, env_name: &str, project_name: Option<String>) {
    let p_name = get_project_name(project_name);

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
        await_exec_result(api_client, &run_slug);
    })
}

pub fn get_project(
    api_client: &APIClient,
    env_slug: &str,
    project_name: &str,
) -> ProjectResult<Project> {
    match api_client.list_projects(
        env_slug,
        Some(&ProjectListFilters {
            name: project_name.to_string(),
        }),
    ) {
        Ok(projects) => {
            if projects.is_empty() {
                Err(ProjectError::ProjectNotFound(format!(
                    "Project {} was not found",
                    project_name
                )))
            } else {
                Ok(projects[0].clone())
            }
        }
        Err(_) => Err(ProjectError::ErrorGettingProject(
            "Error getting project".to_string(),
        )),
    }
}
