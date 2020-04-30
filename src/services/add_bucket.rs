use crate::api_client::{ApiClient, CreateStaticsBucketRequest};
use crate::env_vars;
use crate::schemas::{Project, Service};
use crate::utils::await_exec_result;

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
        let bucket_name_key = "AWS_STATICS_STORAGE_BUCKET_NAME";
        env_vars::create(api_client, service, bucket_name_key, &bucket.identifier);
        println!(
            "Bucket name will be injected into your containers under following key: {}",
            bucket_name_key
        );
        println!("Redeploy {} services to see this variable.", project.name);
    }
}
