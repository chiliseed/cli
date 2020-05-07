use std::process::exit;

use crate::api_client::{ApiClient, ResourceKind, ResourceListFilter};
use crate::schemas::Resource;

pub fn get_db(api_client: &ApiClient, project_slug: &str, db_identifier: String) -> Resource {
    let filters = ResourceListFilter {
        kind: ResourceKind::Database,
        identifier: Some(db_identifier),
    };
    match api_client.list_resources(project_slug, Some(&filters)) {
        Ok(dbs) => {
            if dbs.is_empty() {
                eprintln!("Database not found. Please check the identifier.");
                exit(1)
            } else {
                dbs[0].clone()
            }
        }
        Err(_) => {
            eprintln!("Server error. Please try again later.");
            exit(1)
        }
    }
}
