use crate::client::{APIClient, APIResult};

pub fn add(name: Option<&str>, domain: Option<&str>) -> APIResult<()> {
    Ok(())
}

pub fn list(api_client: &APIClient) -> APIResult<()> {
    api_client.list_envs().and_then(|envs| {
        if envs.is_empty() {
            println!("You have not created any environments yet.");
            return Ok(());
        }
        println!("Your environments: ");
        for env in envs {
            println!("{}", env.name);
        }
        Ok(())
    })
}
