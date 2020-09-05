use std::process::exit;

use crate::api_client::{ApiClient, ResourceKind, ResourceListFilter};
use crate::env_vars::create_env_var_in_project;
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

pub fn set_env_vars(db: &Resource) {
    let db_host_key = "DB_HOST";
    let db_port_key = "DB_PORT";
    let db_username_key = "DB_USERNAME";
    let db_password_key = "DB_PASSWORD";
    let db_name_key = "DB_NAME";

    let keys = vec![
        (db_host_key, db.configuration.address.clone()),
        (db_port_key, format!("{}", db.configuration.port)),
        (db_username_key, db.configuration.username.clone()),
        (db_password_key, db.configuration.password.clone()),
        (db_name_key, db.name.clone()),
    ];
    for (key, val) in keys {
        if !create_env_var_in_project(api_client, &project.slug.clone(), key, &val) {
            return;
        }
    }
    println!("Database parameters will be injected into your containers under following keys: ");
    for key in vec![
        db_name_key,
        db_host_key,
        db_port_key,
        db_username_key,
        db_password_key,
    ] {
        println!("{}", key);
    }

    println!("Redeploy {} services to see these variables.", project_name);
}
