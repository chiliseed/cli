use crate::api_client::{ApiClient, CreateStaticsBucketRequest};
use crate::env_vars;
use crate::schemas::{Project, Service};
use crate::utils::await_exec_result;

const STATICS_BUCKET_KEY_NAME: &str = "AWS_STATICS_STORAGE_BUCKET_NAME";

pub fn add_statics(api_client: &ApiClient, service: Service, project: Project) {
    let (run_slug, bucket_slug) = match api_client.create_statics_bucket(
        &service.slug,
        &CreateStaticsBucketRequest {
            name: service.name.clone(),
        },
    ) {
        Ok(resp) => (resp.log, resp.resource),
        Err(err) => {
            debug!("Error: {}", err);
            eprintln!("Server error. Please try again later");
            return;
        }
    };
    println!("Adding statics bucket to service: {}", service.name);
    let is_launched = await_exec_result(api_client, &run_slug, None);
    if is_launched {
        // todo add verify that these env vars are not taken
        println!("Adding bucket environment variable");
        let bucket = match api_client.get_bucket_details(&bucket_slug) {
            Ok(resource) => resource,
            Err(_) => {
                eprintln!("Server error. Please try again later.");
                return;
            }
        };
        debug!("new resource: {:?}", bucket);
        env_vars::create(
            api_client,
            service,
            STATICS_BUCKET_KEY_NAME,
            &bucket.identifier,
        );
        println!(
            "Bucket name will be injected into your containers under following key: {}",
            STATICS_BUCKET_KEY_NAME
        );
        println!("Redeploy {} services to see this variable.", project.name);
    }
}

pub fn remove_statics(api_client: &ApiClient, service: Service) {
    let run_log = match api_client.remove_statics_bucket(&service.slug) {
        Ok(resp) => resp.log,
        Err(err) => {
            debug!("Server error: {}", err);
            println!("Server error. Please try again later.");
            return;
        }
    };

    println!("Removing statics infra");
    let is_success = await_exec_result(api_client, &run_log, None);

    if is_success {
        println!("Removing statics env var");
        if env_vars::delete_env_var(api_client, &service.slug, STATICS_BUCKET_KEY_NAME) {
            println!("Statics bucket env var removed successfully");
        } else {
            eprintln!(
                "There was an error removing env var {}. Did it exist",
                STATICS_BUCKET_KEY_NAME
            )
        }
    }
}
