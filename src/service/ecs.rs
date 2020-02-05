use core::fmt;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;

use rusoto_core::Region;
use rusoto_ecs::{
    ContainerDefinition, KeyValuePair, LogConfiguration, PortMapping, RegisterTaskDefinitionRequest,
};
use rusoto_iam::{GetRoleRequest, Iam, IamClient};

use crate::project::Project;
use crate::service::{get_repo_name_for_service, get_repository_details};
use std::path::Path;

/// Errors thrown by add ecs service command
#[derive(Debug, PartialEq)]
pub enum ServiceECSError {
    GetExecutorRoleError(String),
    ECRRepoNotFound(String),
    GetECRRepoError(String),
    CreateTaskDefinitionError(String),
    DeploymentFilesExist(String),
    DeploymentSetupError(String),
}
impl fmt::Display for ServiceECSError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}
impl Error for ServiceECSError {
    fn description(&self) -> &str {
        match *self {
            ServiceECSError::GetExecutorRoleError(ref cause) => cause,
            ServiceECSError::GetECRRepoError(ref cause) => cause,
            ServiceECSError::ECRRepoNotFound(ref cause) => cause,
            ServiceECSError::CreateTaskDefinitionError(ref cause) => cause,
            ServiceECSError::DeploymentFilesExist(ref cause) => cause,
            ServiceECSError::DeploymentSetupError(ref cause) => cause,
        }
    }
}

/// Parameters of add ecs service command
pub struct AddECSServiceParams {
    pub project: Project,
    pub cluster: String,
    pub port: Option<i64>,
    pub service_name: String,
}

fn get_ecs_executor_role(cluster: &str, env_name: &str) -> Result<String, ServiceECSError> {
    let client = IamClient::new(Region::default());
    let executor_role_name = format!("{}_{}_ecs_task_executor", env_name, cluster);

    info!("Looking for ECS task executor role: {}", executor_role_name);

    let request = GetRoleRequest {
        role_name: executor_role_name,
    };
    match client.get_role(request).sync() {
        Ok(role_response) => Ok(role_response.role.arn),
        Err(err) => {
            error!("Error getting role arn: {}", err);
            Err(ServiceECSError::GetExecutorRoleError(err.to_string()))
        }
    }
}

/// Get image name for service
fn get_ecr_repo(
    env_name: &str,
    project_name: &str,
    service_name: &str,
) -> Result<String, ServiceECSError> {
    let repo_name = get_repo_name_for_service(env_name, project_name, service_name);
    match get_repository_details(&repo_name) {
        Ok(repo) => match repo {
            Some(repository_details) => match repository_details.repository_uri {
                Some(uri) => Ok(uri),
                None => Err(ServiceECSError::GetECRRepoError(
                    "Repository had no uri!".to_string(),
                )),
            },
            None => Err(ServiceECSError::ECRRepoNotFound(
                "Repository not found".to_string(),
            )),
        },
        Err(e) => Err(ServiceECSError::GetECRRepoError(e.to_string())),
    }
}

const BASE_CONTAINER_MEMORY_MB: i64 = 128;
const MEMORY_RESERVATION_MB: &str = "128m";
const CPU_SHARES: &str = "100";
const LAUNCH_TYPE: &str = "EC2";

/// Configures ECS task definition
fn create_task_definition(
    project: &Project,
    cluster: &str,
    service_name: &str,
    port: Option<i64>,
) -> Result<RegisterTaskDefinitionRequest, ServiceECSError> {
    let executor_arn = get_ecs_executor_role(cluster, &project.env_name)?;
    let image_uri = get_ecr_repo(&project.env_name, &project.name, service_name)?;

    let mut log_options: HashMap<String, String> = HashMap::new();
    log_options.insert(
        "awslogs-group".to_string(),
        format!("{}-{}-{}", project.env_name, project.name, service_name),
    );
    log_options.insert(
        "awslogs-region".to_string(),
        Region::default().name().to_string(),
    );
    log_options.insert(
        "awslogs-stream-prefix".to_string(),
        service_name.to_string(),
    );

    let containers = vec![ContainerDefinition {
        command: None,
        cpu: None,
        depends_on: None,
        disable_networking: None,
        dns_search_domains: None,
        dns_servers: None,
        docker_labels: None,
        docker_security_options: None,
        entry_point: None,
        environment: Some(vec![KeyValuePair {
            name: Some("VERSION".to_string()),
            value: Some(format!("%VERSION%")),
        }]),
        essential: Some(true),
        extra_hosts: None,
        firelens_configuration: None,
        health_check: None,
        hostname: None,
        image: Some(format!("{}:%VERSION%", image_uri)),
        interactive: None,
        links: None,
        linux_parameters: None,
        log_configuration: Some(LogConfiguration {
            log_driver: "awslogs".to_string(),
            options: Some(log_options),
            secret_options: None,
        }),
        memory: None,
        memory_reservation: None,
        mount_points: None,
        name: Some(service_name.to_string()),
        port_mappings: Some(vec![PortMapping {
            container_port: port,
            host_port: None, // dynamic port mapping via ALB
            protocol: Some("tcp".to_string()),
        }]),
        privileged: None,
        pseudo_terminal: None,
        readonly_root_filesystem: None,
        repository_credentials: None,
        resource_requirements: None,
        secrets: None,
        start_timeout: None,
        stop_timeout: None,
        system_controls: None,
        ulimits: None,
        user: None,
        volumes_from: None,
        working_directory: None,
    }];
    let task_definition = RegisterTaskDefinitionRequest {
        container_definitions: containers,
        cpu: Some(CPU_SHARES.to_string()),
        execution_role_arn: Some(executor_arn.clone()),
        family: service_name.to_string(),
        inference_accelerators: None,
        ipc_mode: None,
        memory: Some(MEMORY_RESERVATION_MB.to_string()),
        network_mode: None,
        pid_mode: None,
        placement_constraints: None,
        proxy_configuration: None,
        requires_compatibilities: None,
        tags: None,
        task_role_arn: None,
        volumes: None,
    };

    debug!("Constructed task definition: {:?}", task_definition);
    Ok(task_definition)
}

const DEPLOYMENT_DIR: &str = "deployment";
const TASK_DEFINITION_TEMPLATE_SUFFIX: &str = ".ecs-td.template.json";

/// Creates deployment/<service-name>.ecs-task-definition.template.json
pub fn setup_deployment(params: AddECSServiceParams) -> Result<(), ServiceECSError> {
    let here = env::current_dir().unwrap();
    let deployment_dir = here.join(DEPLOYMENT_DIR);
    if !deployment_dir.exists() {
        info!("Creating deployment directory");
        fs::create_dir_all(Path::new(deployment_dir.to_str().unwrap()))
            .map_err(|_err| {
                ServiceECSError::DeploymentSetupError(
                    "Failed to create deployment path".to_string(),
                )
            })
            .unwrap();
    }

    let task_definition_template =
        format!("{}{}", params.service_name, TASK_DEFINITION_TEMPLATE_SUFFIX);
    let deployment_conf_file =
        Path::new(deployment_dir.to_str().unwrap()).join(task_definition_template);
    if deployment_conf_file.exists() {
        error!(
            "Existing deployment files found. Remove {} and try again.",
            deployment_conf_file.to_str().unwrap()
        );
        return Err(ServiceECSError::DeploymentFilesExist(
            "Cannot override existing deployment configuration".to_string(),
        ));
    }
    let deployment_conf_file = File::create(deployment_conf_file)
        .map_err(|_err| {
            ServiceECSError::DeploymentSetupError(
                "Failed to create deployment configurations path".to_string(),
            )
        })
        .unwrap();

    let task_definition = create_task_definition(
        &params.project,
        &params.cluster,
        &params.service_name,
        params.port,
    )?;

    info!("Put deployment files in place");
    serde_json::to_writer_pretty(&deployment_conf_file, &task_definition)
        .map_err(|_err| {
            ServiceECSError::DeploymentSetupError("Failed to save task definition".to_string())
        })
        .unwrap();

    Ok(())
}
