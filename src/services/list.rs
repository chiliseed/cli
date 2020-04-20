use prettytable::{format, Cell, Row, Table};

use super::types::ServiceError;
use super::utils::get_services;
use crate::api_client::ApiClient;

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
                let mut table = Table::new();
                let format = format::FormatBuilder::new().column_separator('\t').build();
                table.set_format(format);

                println!();
                println!("{}", service.name);
                println!("{}", std::iter::repeat("=").take(60).collect::<String>());
                table.add_row(Row::new(vec![
                    Cell::new("Subomain"),
                    Cell::new(service.subdomain.as_str()),
                ]));
                table.add_row(Row::new(vec![
                    Cell::new("Container"),
                    Cell::new(format!("{}", service.container_port).as_str()),
                ]));
                table.add_row(Row::new(vec![
                    Cell::new("ALB HTTP Port"),
                    Cell::new(format!("{}", service.alb_port_http).as_str()),
                ]));
                table.add_row(Row::new(vec![
                    Cell::new("ALB HTTPS Port"),
                    Cell::new(format!("{}", service.alb_port_https).as_str()),
                ]));
                table.add_row(Row::new(vec![
                    Cell::new("Healthcheck"),
                    Cell::new(service.health_check_endpoint.as_str()),
                ]));
                table.add_row(Row::new(vec![
                    Cell::new("ECR Repo"),
                    Cell::new(ecr_repo_name),
                ]));
                table.add_row(Row::new(vec![Cell::new("AWS Region"), Cell::new(region)]));
                table.add_row(Row::new(vec![
                    Cell::new("AWS Account"),
                    Cell::new(account_id),
                ]));

                table.printstd();
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
