use super::types::ServiceError;
use super::utils::get_services;
use crate::api_client::ApiClient;
use crate::schemas::Project;
use crate::utils::{add_row_to_output_table, get_output_table};

pub fn list_services(api_client: &ApiClient, project: Project) {
    match get_services(api_client, &project, None) {
        Ok(services) => {
            if services.is_empty() {
                println!("Project has no services.");
                return;
            }

            println!("Project {} has following services: ", project.name);
            for service in services {
                let ecr_repo_url = service.ecr_repo_url.unwrap();
                let ecr_parts: Vec<&str> = ecr_repo_url.split(".com").collect();
                let ecr_repo_name = ecr_parts[1];
                let aws_url_parts: Vec<&str> = ecr_repo_url.split(".").collect();
                let region = aws_url_parts[3];
                let account_id = aws_url_parts[0];

                let mut table = get_output_table();

                println!();
                println!("{}", service.name);
                println!("{}", std::iter::repeat("=").take(60).collect::<String>());
                add_row_to_output_table(&mut table, vec!["Subomain", service.subdomain.as_str()]);
                add_row_to_output_table(
                    &mut table,
                    vec!["Container", format!("{}", service.container_port).as_str()],
                );
                add_row_to_output_table(
                    &mut table,
                    vec![
                        "ALB HTTP Port",
                        format!("{}", service.alb_port_http).as_str(),
                    ],
                );
                add_row_to_output_table(
                    &mut table,
                    vec![
                        "ALB HTTPS Port",
                        format!("{}", service.alb_port_https).as_str(),
                    ],
                );
                add_row_to_output_table(
                    &mut table,
                    vec!["Healthcheck", service.health_check_endpoint.as_str()],
                );
                add_row_to_output_table(
                    &mut table,
                    vec!["Dockerfile", service.default_dockerfile_path.as_str()],
                );
                if service.default_dockerfile_target.is_some() {
                    add_row_to_output_table(
                        &mut table,
                        vec![
                            "Dockerfile stage",
                            service.default_dockerfile_target.unwrap().as_str(),
                        ],
                    );
                }
                add_row_to_output_table(&mut table, vec!["ECR Repo", ecr_repo_name]);
                add_row_to_output_table(&mut table, vec!["AWS Region", region]);
                add_row_to_output_table(&mut table, vec!["AWS Account", account_id]);

                table.printstd();
            }
        }

        Err(ServiceError::ServicesNotFound(_err)) => {
            println!("Project {} has no services yet.", project.name);
            return;
        }

        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}
