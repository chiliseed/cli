use crate::api_client::ApiClient;
use crate::env_vars;
use crate::schemas::{Resource, Service};

pub fn add_database(api_client: &ApiClient, service: Service, db: Resource) {
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
        if !env_vars::create(api_client, service.clone(), key, &val) {
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

    println!("Redeploy {} services to see these variables.", service.name);
}
