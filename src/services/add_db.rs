use crate::api_client::{AddDbRequest, ApiClient};
use crate::db::set_env_vars;
use crate::env_vars;
use crate::schemas::{Resource, Service};
use crate::utils::await_exec_result;

pub fn add_database(api_client: &ApiClient, service: Service, db: Resource) {
    let run_log = match api_client.add_db_to_service(
        &service.slug,
        &AddDbRequest {
            db_slug: db.slug.clone(),
        },
    ) {
        Ok(resp) => resp.log,
        Err(err) => {
            debug!("Server error: {}", err);
            eprintln!("Server error. Please try again later or contact Chiliseed support");
            return;
        }
    };

    if !await_exec_result(api_client, &run_log, None) {
        eprintln!("Failed to connect db to service");
        return;
    }

    set_env_vars(&db);
}
