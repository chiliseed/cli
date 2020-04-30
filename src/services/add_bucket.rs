use crate::api_client::{ApiClient, CreateStaticsBucketRequest};
use crate::schemas::Service;
use crate::utils::await_exec_result;

pub fn add_statics(api_client: &ApiClient, service: Service) {
    let run_slug = match api_client.create_statics_bucket(
        &service.slug,
        &CreateStaticsBucketRequest {
            name: service.name.clone(),
        },
    ) {
        Ok(resp) => resp.log,
        Err(err) => {
            debug!("Error: {}", err);
            eprintln!("Server error. Please try again later");
            return;
        }
    };
    println!("Adding statics bucket to service: {}", service.name);
    await_exec_result(api_client, &run_slug, None);
}
