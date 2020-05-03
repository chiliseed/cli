use text_io::read;

use crate::api_client::{ApiClient, CreateServiceRequest};
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

    println!("Does your service need web interface? [y/n]: ");
    let is_web: String = read!();
    let mut has_web_interface = true;
    if !vec!["y", "Y", "yes", "Yes", "YES"].contains(&is_web.as_str()) {
        has_web_interface = false;
    }

    let mut subdomain = "".to_string();
    let mut container_port = "".to_string();
    let mut alb_port_http = "".to_string();
    let mut alb_port_https = "".to_string();
    let mut health_check_endpoint = "".to_string();

    if has_web_interface {
        println!("Your service subdomain (for example, for api.example.com, subdomain is `api`): ");
        subdomain = read!();

        println!("On what port will your container listen (example: 8000): ");
        container_port = read!();

        println!("On what port do you want the load balancer to listen for HTTP traffic for this service (example: 80): ");
        alb_port_http = read!();

        println!("On what port do you want the load balancer to listen for HTTPS traffic for this service (example: 443): ");
        alb_port_https = read!();

        println!("What is your health check endpoint (example: /api/health/check/): ");
        health_check_endpoint = read!();
    }

    let mut default_dockerfile_path = "Dockerfile".to_string();
    println!(
        "Path to service's dockerfile, relative to project root [defaults to '{}']: ",
        default_dockerfile_path
    );
    let dockerfile: String = read!();
    if !dockerfile.is_empty() {
        default_dockerfile_path = dockerfile;
    }

    let mut default_dockerfile_target = None;
    println!("Optional specific dockerfile target to build: ");
    let dockerfile_target: String = read!();
    if !dockerfile_target.is_empty() {
        default_dockerfile_target = Some(dockerfile_target);
    }

    let service = CreateServiceRequest {
        name,
        has_web_interface,
        default_dockerfile_path,
        default_dockerfile_target,
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
    await_exec_result(api_client, &run_slug, None);
}
