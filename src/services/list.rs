use super::types::ServiceError;
use super::utils::get_services;
use crate::client::ApiClient;

pub fn list_services(api_client: &ApiClient, env_name: &str, project_name: &str) {
    match get_services(api_client, env_name, project_name, None) {
        Ok(services) => {
            if services.is_empty() {
                println!("Project has no services.");
                return;
            }

            println!("Project {} has following services: ", project_name);
            for service in services {
                let ecr_repo_url = service.ecr_repo_url.unwrap();
                let ecr_parts: Vec<&str> = ecr_repo_url.split(".com").collect();
                let ecr_repo_name = ecr_parts[1];
                let aws_url_parts: Vec<&str> = ecr_repo_url.split(".").collect();
                let region = aws_url_parts[3];
                let account_id = aws_url_parts[0];
                println!();
                println!("{}", service.name);
                println!("{}", std::iter::repeat("=").take(60).collect::<String>());
                println!(
                    "Subomain {} {}",
                    std::iter::repeat(" ").take(9).collect::<String>(),
                    service.subdomain
                );
                println!(
                    "Container Port {} {}",
                    std::iter::repeat(" ").take(3).collect::<String>(),
                    service.container_port
                );
                println!(
                    "ALB HTTP Port {} {}",
                    std::iter::repeat(" ").take(4).collect::<String>(),
                    service.alb_port_http
                );
                println!(
                    "ALB HTTPS Port {} {}",
                    std::iter::repeat(" ").take(3).collect::<String>(),
                    service.alb_port_https
                );
                println!(
                    "Healthcheck {} {}",
                    std::iter::repeat(" ").take(6).collect::<String>(),
                    service.alb_port_https
                );
                println!(
                    "ECR Repo {} {}",
                    std::iter::repeat(" ").take(9).collect::<String>(),
                    ecr_repo_name
                );
                println!(
                    "AWS Region {} {}",
                    std::iter::repeat(" ").take(7).collect::<String>(),
                    region
                );
                println!(
                    "AWS Account {} {}",
                    std::iter::repeat(" ").take(6).collect::<String>(),
                    account_id
                );
            }
        }

        Err(ServiceError::ServicesNotFound(_err)) => {
            println!(
                "Project {} ({}) has no services yet.",
                project_name, env_name
            );
            return;
        }

        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}
