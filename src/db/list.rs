use crate::api_client::{ApiClient, ResourceKind, ResourceListFilter};
use crate::schemas::Project;
use crate::utils::{add_row_to_output_table, get_output_table};

pub fn list_databases(api_client: &ApiClient, project: &Project) {
    let filter = ResourceListFilter {
        kind: ResourceKind::Database,
        identifier: None,
    };
    match api_client.list_resources(&project.slug, Some(&filter)) {
        Ok(dbs) => {
            debug!("resources: {:?}", dbs);
            for db in dbs {
                println!();
                println!("{}", db.name);
                println!("{}", std::iter::repeat("=").take(60).collect::<String>());

                let mut table = get_output_table();
                add_row_to_output_table(&mut table, vec!["Name", db.name.as_str()]);
                add_row_to_output_table(&mut table, vec!["Identifier", db.identifier.as_str()]);
                add_row_to_output_table(&mut table, vec!["Preset", db.preset.as_str()]);
                add_row_to_output_table(&mut table, vec!["Engine", db.engine.as_str()]);
                add_row_to_output_table(
                    &mut table,
                    vec!["Engine version", db.configuration.engine_version.as_str()],
                );
                add_row_to_output_table(
                    &mut table,
                    vec!["Instance type", db.configuration.instance_type.as_str()],
                );
                add_row_to_output_table(
                    &mut table,
                    vec![
                        "Allocated storage",
                        &format!("{}", db.configuration.allocated_storage),
                    ],
                );
                add_row_to_output_table(
                    &mut table,
                    vec!["Created at", &db.created_at.to_rfc2822()],
                );
                table.printstd();
            }
        }

        Err(err) => {
            debug!("Error: {}", err);
            println!("Server error. Please try again later.")
        }
    }
}
