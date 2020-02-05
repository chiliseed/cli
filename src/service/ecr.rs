use std::fmt;

use rusoto_core::{Region, RusotoError};
use rusoto_ecr::{
    CreateRepositoryError, CreateRepositoryRequest, DescribeRepositoriesRequest, Ecr, EcrClient,
    PutLifecyclePolicyRequest, Repository, Tag,
};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::project::Project;

/// Create ECR repo.
pub fn create_ecr_repo(
    project: &Project,
    repository_name: &str,
) -> Result<Option<Repository>, &'static str> {
    let client = EcrClient::new(Region::default());
    let repo_request = CreateRepositoryRequest {
        repository_name: repository_name.to_string(),
        image_tag_mutability: Some("IMMUTABLE".to_string()),
        tags: Some(vec![
            Tag {
                key: Some("Environment".to_string()),
                value: Some(project.env_name.to_string()),
            },
            Tag {
                key: Some("Project".to_string()),
                value: Some(project.name.to_string()),
            },
        ]),
    };

    match client.create_repository(repo_request).sync() {
        Ok(output) => {
            info!("Successfully created repo");
            Ok(output.repository)
        }
        Err(e) => match e {
            RusotoError::Service(CreateRepositoryError::RepositoryAlreadyExists(_err)) => {
                info!("Repository exists. Retrieving details");

                match get_repository_details(&repository_name) {
                    Ok(repo) => Ok(repo),
                    Err(e) => Err(e),
                }
            }
            _ => {
                error!("Create repository error: {}", e);
                Err("Failed to create new ECR repository")
            }
        },
    }
}

/// Get repository details by repository name
pub(crate) fn get_repository_details(
    repository_name: &str,
) -> Result<Option<Repository>, &'static str> {
    info!("Get repository details: {}", repository_name);
    let req = DescribeRepositoriesRequest {
        max_results: None,
        next_token: None,
        registry_id: None,
        repository_names: Some(vec![repository_name.to_string()]),
    };

    let client = EcrClient::new(Region::default());

    let response = client.describe_repositories(req).sync().unwrap();

    if let Some(repos) = response.repositories {
        for repo in repos {
            if repo.clone().repository_name.unwrap() == repository_name {
                return Ok(Some(repo));
            }
        }
    }

    Ok(None)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum ImageTagStatus {
    #[allow(non_camel_case_types)]
    tagged,
    #[allow(non_camel_case_types)]
    untagged,
    #[allow(non_camel_case_types)]
    any,
}
impl fmt::Display for ImageTagStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Self::tagged => "tagged",
            Self::untagged => "untagged",
            Self::any => "any",
        };

        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[rustfmt::skip]
enum ImageCountType {
    #[allow(non_camel_case_types)]
    imageCountMoreThan,
    #[allow(non_camel_case_types)]
    sinceImagePushed,
}

impl fmt::Display for ImageCountType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Self::imageCountMoreThan => "imageCountMoreThan",
            Self::sinceImagePushed => "sinceImagePushed",
        };

        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PolicyRuleSelection {
    #[serde(rename = "tagStatus")]
    tag_status: ImageTagStatus,

    #[serde(rename = "tagPrefixList", skip_serializing_if = "Option::is_none")]
    tag_prefix_list: Option<Vec<String>>,

    #[serde(rename = "countType")]
    count_type: ImageCountType,

    #[serde(rename = "countUnit", skip_serializing_if = "Option::is_none")]
    count_unit: Option<String>,

    #[serde(rename = "countNumber")]
    count_number: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum PolicyActionType {
    #[allow(non_camel_case_types)]
    expire,
}

impl fmt::Display for PolicyActionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Self::expire => "expire",
        };

        write!(f, "{}", name)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PolicyRuleAction {
    #[serde(rename = "type")]
    action_type: PolicyActionType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PolicyRule {
    #[serde(rename = "rulePriority")]
    rule_priority: u8,
    description: String,
    selection: PolicyRuleSelection,
    action: PolicyRuleAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct PolicyRules {
    rules: Vec<PolicyRule>,
}

const IMAGES_MAX_COUNT: u32 = 100;

/// Create expiration policy for images in a repository.
///
/// Currently supports only expiry by "count more than".
/// First 100 images will be saved, no matter how old, the rest will be deleted.
pub fn create_ecr_repo_policy(
    repository_name: &str,
    registry_id: &str,
) -> Result<(), &'static str> {
    info!("Setting policy for repository: {}", repository_name);

    let policy: PolicyRules = PolicyRules {
        rules: vec![PolicyRule {
            rule_priority: 1,
            description: "Expire images by count > 100".to_string(),
            selection: PolicyRuleSelection {
                tag_status: ImageTagStatus::any,
                tag_prefix_list: None,
                count_type: ImageCountType::imageCountMoreThan,
                count_unit: None,
                count_number: IMAGES_MAX_COUNT,
            },
            action: PolicyRuleAction {
                action_type: PolicyActionType::expire,
            },
        }],
    };

    let life_cycle_policy = PutLifecyclePolicyRequest {
        lifecycle_policy_text: serde_json::to_string(&policy).unwrap(),
        repository_name: repository_name.to_string(),
        registry_id: Some(registry_id.to_string()),
    };

    debug!("Policy details: {:?}", life_cycle_policy);

    let client = EcrClient::new(Region::default());
    match client.put_lifecycle_policy(life_cycle_policy).sync() {
        Ok(_response) => {
            info!(
                "Policy was successfully set for repository: {} policy type: {}",
                repository_name,
                ImageCountType::imageCountMoreThan
            );
            Ok(())
        }
        Err(e) => {
            error!("Failed to set repository lifecycle policy: {}", e);
            Err("Error setting repository lifecycle policy")
        }
    }
}

/// Constructs the name of the ECR repo for the service
pub(crate) fn get_repo_name_for_service(
    env_name: &str,
    project_name: &str,
    service_name: &str,
) -> String {
    format!("{}/{}/{}", env_name, project_name, service_name)
}

/// Main tasks for creating ECR repository.
/// ECR repos will have the name of environment_name/project_name/service_name
/// ECR will have a lifetime policy of last N images saved.
pub fn add_new_repo(project: &Project, service_name: &str) -> Result<Repository, &'static str> {
    let repository_name =
        get_repo_name_for_service(&project.env_name, &project.name, &service_name);
    let repository = create_ecr_repo(&project, &repository_name)
        .unwrap()
        .unwrap();

    let repo = repository.clone();
    debug!("Repo details: {:?}", repo);
    match create_ecr_repo_policy(&repo.repository_name.unwrap(), &repo.registry_id.unwrap()) {
        Ok(()) => Ok(repository),
        Err(e) => Err(e),
    }
}
