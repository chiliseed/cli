use crate::api_client::{ApiClient, CreateServiceRequest};
use crate::schemas::{Project, Service};
use crate::services::{ask_for_value, ask_yes_no};
use crate::utils::await_exec_result;

pub fn update(api_client: &ApiClient, project: Project, service: Service) {
    let name = ask_for_value(
        format!("Your service name (current: {}): ", service.name),
        service.name.clone(),
    );

    let has_web_interface = ask_yes_no(
        format!(
            "Does your service need web interface? (current: {}) [y/n]: ",
            service.has_web_interface
        ),
        service.has_web_interface.clone(),
    );

    let mut subdomain = "".to_string();
    let mut container_port = "".to_string();
    let mut alb_port_http = "".to_string();
    let mut alb_port_https = "".to_string();
    let mut health_check_endpoint = "".to_string();

    if has_web_interface {
        subdomain = ask_for_value(
            format!("Your service subdomain (current: {}): ", service.subdomain),
            service.subdomain.clone(),
        );

        container_port = ask_for_value(
            format!(
                "On what port will your container listen (current: {}): ",
                service.container_port
            ),
            format!("{}", service.container_port),
        );

        alb_port_http = ask_for_value(
            format!("On what port do you want the load balancer to listen for HTTP traffic (current: {}): ", service.alb_port_http),
            format!("{}", service.alb_port_http)
        );

        alb_port_https = ask_for_value(
            format!("On what port do you want the load balancer to listen for HTTPS traffic (current: {}): ", service.alb_port_https),
            format!("{}", service.alb_port_https),
        );

        health_check_endpoint = ask_for_value(
            format!(
                "What is your health check endpoint (current: {}): ",
                service.health_check_endpoint
            ),
            service.health_check_endpoint.clone(),
        );
    }

    let default_dockerfile_path = ask_for_value(
        format!(
            "Path to service's dockerfile, relative to project root (current: {}): ",
            service.default_dockerfile_path
        ),
        service.default_dockerfile_path.clone(),
    );

    let dockerfile_target = service.default_dockerfile_target.unwrap_or("".to_string());
    let default_dockerfile_target = ask_for_value(
        format!(
            "Optional specific dockerfile target to build (current: {}): ",
            dockerfile_target.clone()
        ),
        dockerfile_target,
    );

    let service = Service {
        slug: service.slug.clone(),
        name,
        has_web_interface,
        default_dockerfile_path,
        default_dockerfile_target: Some(default_dockerfile_target),
        subdomain,
        container_port: container_port.parse::<u32>().unwrap(),
        alb_port_http: alb_port_http.parse::<u32>().unwrap(),
        alb_port_https: alb_port_https.parse::<u32>().unwrap(),
        health_check_endpoint,
        ecr_repo_url: service.ecr_repo_url.clone(),
    };

    let run_slug = match api_client.update_service(&service, &project.slug) {
        Ok(resp) => resp.log,
        Err(_) => return,
    };

    println!("Updating service");
    await_exec_result(api_client, &run_slug, None);
}
