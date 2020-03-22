use text_io::read;

use crate::client::{ApiClient, CreateServiceRequest};
use crate::environments::get_env;
use crate::projects::get_project;
use crate::utils::await_exec_result;

pub fn create_service(api_client: &ApiClient, env_name: &str, project_name: &str) {
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

    println!("Your service name (example: api): ");
    let name: String = read!();

    println!("Your service subdomain (for example, for api.example.com, subdomain is `api`): ");
    let subdomain: String = read!();

    println!("On what port will your container listen (example: 8000): ");
    let container_port: String = read!();

    println!("On what port do you want the load balancer to listen for HTTP traffic for this service (example: 80): ");
    let alb_port_http: String = read!();

    println!("On what port do you want the load balancer to listen for HTTPS traffic for this service (example: 443): ");
    let alb_port_https: String = read!();

    println!("What is your health check endpoint (example: /api/health/check/): ");
    let health_check_endpoint: String = read!();

    let service = CreateServiceRequest {
        name,
        subdomain,
        container_port,
        alb_port_http,
        alb_port_https,
        health_check_endpoint,
    };

    let run_slug = match api_client.create_service(&service, &project.slug) {
        Ok(resp) => resp.log,
        Err(_) => return,
    };

    println!("Launching service infra: {}", service.name);
    await_exec_result(api_client, &run_slug);
}
