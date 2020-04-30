use std::fmt;
use text_io::read;

use crate::api_client::{ApiClient, CreateDbRequest};
use crate::env_vars::create_env_var_in_project;
use crate::environments::get_env;
use crate::projects::get_project;
use crate::utils::await_exec_result;

const DEV: &str = "dev";
const PROD: &str = "prod";

enum Preset {
    Dev,
    Prod,
}

impl fmt::Display for Preset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Preset::Dev => write!(f, "{}", DEV),
            Preset::Prod => write!(f, "{}", PROD),
        }
    }
}

pub fn create_db(api_client: &ApiClient, env_name: &str, project_name: &str) {
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

    println!("Database name: ");
    let db_name: String = read!("{}\n");

    println!("Database username: ");
    let username: String = read!("{}\n");

    println!("Select preset ({}/{}, defaults to {}): ", DEV, PROD, DEV);
    let selected_preset: String = read!("{}\n");
    let mut preset = Preset::Dev;
    if !selected_preset.is_empty() {
        preset = match selected_preset.to_lowercase().as_str() {
            DEV => Preset::Dev,
            PROD => Preset::Prod,
            _ => {
                eprintln!(
                    "Bad preset: {}. Only {} or {} are supported",
                    selected_preset, DEV, PROD,
                );
                return;
            }
        };
    }

    let engine = "postgres".to_string();

    debug!("Requesting to create new db");
    let (run_slug, db_slug) = match api_client.create_db(
        &env.slug,
        &CreateDbRequest {
            name: db_name.clone(),
            username,
            engine,
            preset: format!("{}", preset),
            project: project.slug.clone(),
        },
    ) {
        Ok(resp) => (resp.log, resp.resource),
        Err(_) => {
            eprintln!("Server error. Please try again later or contact Chiliseed support");
            return;
        }
    };

    println!("Launching new db: {}", db_name);
    let is_launched = await_exec_result(api_client, &run_slug, Some(40));
    if is_launched {
        // todo add verify that these env vars are not taken
        println!("Adding database environment variables");
        let db = match api_client.get_resource_details(&db_slug) {
            Ok(db) => db,
            Err(_) => {
                eprintln!("Server error. Please try again later.");
                return;
            }
        };
        debug!("new db: {:?}", db);

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
        println!(
            "Database parameters will be injected into your containers under following keys: "
        );
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
}
