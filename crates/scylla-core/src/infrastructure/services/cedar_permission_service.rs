use crate::application::PermissionService;
use crate::application::caller::CallerContext;
use crate::application::user_role::UserRoleRepository;
use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::value_objects::permission::policy::Policy;
use crate::domain::value_objects::permission::{Act, Resource, Target};
use async_trait::async_trait;
use cedar_policy::{
    Authorizer, Context, Decision, Entities, Entity, EntityUid, PolicySet, Request,
};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, instrument};

/// Cedar policy text compiled into the binary. Authoring lives in
/// `cedar/policies.cedar` next to this file.
const POLICIES: &str = include_str!("cedar/policies.cedar");

pub struct CedarPermissionService<URR: UserRoleRepository> {
    policies: PolicySet,
    authorizer: Authorizer,
    user_role_repo: Arc<URR>,
}

impl<URR: UserRoleRepository> CedarPermissionService<URR> {
    pub fn new(user_role_repo: Arc<URR>) -> DomainResult<Self> {
        let policies = PolicySet::from_str(POLICIES)
            .map_err(|e| DomainError::Internal(format!("cedar policy parse: {e}")))?;
        Ok(Self {
            policies,
            authorizer: Authorizer::new(),
            user_role_repo,
        })
    }

    fn resource_uid(resource: &Resource) -> DomainResult<EntityUid> {
        let s = match resource {
            Resource::User(Target::All) => r#"Scylla::User::"all""#.to_string(),
            Resource::User(Target::Single(id)) => {
                format!(r#"Scylla::User::"{}""#, id.as_str())
            }
            Resource::Pipeline(Target::All) => r#"Scylla::Pipeline::"all""#.to_string(),
            Resource::Pipeline(Target::Single(id)) => {
                format!(r#"Scylla::Pipeline::"{}""#, id.as_str())
            }
            Resource::Job(Target::All) => r#"Scylla::Job::"all""#.to_string(),
            Resource::Job(Target::Single(id)) => format!(r#"Scylla::Job::"{}""#, id.as_str()),
            Resource::Project(Target::All) => r#"Scylla::Project::"all""#.to_string(),
            Resource::Project(Target::Single(id)) => {
                format!(r#"Scylla::Project::"{}""#, id.as_str())
            }
            Resource::Organization(Target::All) => r#"Scylla::Organization::"all""#.to_string(),
            Resource::Organization(Target::Single(id)) => {
                format!(r#"Scylla::Organization::"{}""#, id.as_str())
            }
            Resource::Agent(Target::All) => r#"Scylla::Agent::"all""#.to_string(),
            Resource::Agent(Target::Single(id)) => {
                format!(r#"Scylla::Agent::"{}""#, id.as_str())
            }
            Resource::All => r#"Scylla::Resource::"all""#.to_string(),
        };
        EntityUid::from_str(&s)
            .map_err(|e| DomainError::Internal(format!("cedar resource uid {s}: {e}")))
    }

    fn action_uid(act: &Act) -> DomainResult<EntityUid> {
        let s = match act {
            Act::Create => r#"Action::"create""#,
            Act::Read => r#"Action::"read""#,
            Act::Write => r#"Action::"write""#,
            Act::Delete => r#"Action::"delete""#,
            Act::Execute => r#"Action::"execute""#,
            Act::All => r#"Action::"any""#,
        };
        EntityUid::from_str(s)
            .map_err(|e| DomainError::Internal(format!("cedar action uid {s}: {e}")))
    }

    async fn build_principal(
        &self,
        caller: &CallerContext,
    ) -> DomainResult<(EntityUid, Entities)> {
        match caller {
            CallerContext::User(id) => {
                let uid_str = format!(r#"Scylla::User::"{}""#, id.as_str());
                let principal_uid = EntityUid::from_str(&uid_str)
                    .map_err(|e| DomainError::Internal(format!("cedar user uid: {e}")))?;

                let roles = self.user_role_repo.list_roles_for_user(id).await?;
                let mut parents: HashSet<EntityUid> = HashSet::new();
                let mut entities: Vec<Entity> = Vec::new();
                for role in &roles {
                    let role_uid_str = format!(r#"Scylla::Role::"{}""#, role.as_str());
                    let role_uid = EntityUid::from_str(&role_uid_str)
                        .map_err(|e| DomainError::Internal(format!("cedar role uid: {e}")))?;
                    parents.insert(role_uid.clone());
                    let role_entity = Entity::new(role_uid, HashMap::new(), HashSet::new())
                        .map_err(|e| {
                            DomainError::Internal(format!("cedar role entity: {e}"))
                        })?;
                    entities.push(role_entity);
                }
                let principal_entity =
                    Entity::new(principal_uid.clone(), HashMap::new(), parents).map_err(|e| {
                        DomainError::Internal(format!("cedar user entity: {e}"))
                    })?;
                entities.push(principal_entity);
                let entity_store = Entities::from_entities(entities, None).map_err(|e| {
                    DomainError::Internal(format!("cedar entities build: {e}"))
                })?;
                Ok((principal_uid, entity_store))
            }
            CallerContext::Service(svc) => {
                let uid_str = format!(r#"Scylla::Service::"{}""#, svc.as_str());
                let principal_uid = EntityUid::from_str(&uid_str)
                    .map_err(|e| DomainError::Internal(format!("cedar service uid: {e}")))?;
                let entity =
                    Entity::new(principal_uid.clone(), HashMap::new(), HashSet::new()).map_err(
                        |e| DomainError::Internal(format!("cedar service entity: {e}")),
                    )?;
                let entity_store = Entities::from_entities(vec![entity], None).map_err(|e| {
                    DomainError::Internal(format!("cedar entities build: {e}"))
                })?;
                Ok((principal_uid, entity_store))
            }
            CallerContext::Anonymous => Err(DomainError::Forbidden(
                "Anonymous caller is not permitted".to_string(),
            )),
        }
    }
}

#[async_trait]
impl<URR: UserRoleRepository + 'static> PermissionService for CedarPermissionService<URR> {
    #[instrument(skip(self, caller, policy), fields(caller = ?caller))]
    async fn check(&self, caller: &CallerContext, policy: Policy) -> DomainResult<bool> {
        let (principal_uid, entities) = self.build_principal(caller).await?;
        let action_uid = Self::action_uid(&policy.act)?;
        let resource_uid = Self::resource_uid(&policy.resource)?;

        let request = Request::new(
            principal_uid,
            action_uid,
            resource_uid,
            Context::empty(),
            None,
        )
        .map_err(|e| DomainError::Internal(format!("cedar request: {e}")))?;

        let response = self
            .authorizer
            .is_authorized(&request, &self.policies, &entities);
        match response.decision() {
            Decision::Allow => Ok(true),
            Decision::Deny => {
                debug!(
                    reasons = ?response.diagnostics().reason().collect::<Vec<_>>(),
                    errors = ?response.diagnostics().errors().collect::<Vec<_>>(),
                    "cedar denied"
                );
                Err(DomainError::Forbidden("Permission denied".to_string()))
            }
        }
    }
}
