use domain::entities::{JobId, OrganizationId, PipelineId, ProjectId, UserId};
use domain::errors::{DomainError, DomainResult};
use domain::value_objects::permission::{Act, Resource, Scope, Target};
use protocol::services::permission::{
    Act as ProtoAct, Resource as ProtoResource, ResourceType, Scope as ProtoScope, ScopeType,
};

// ── Act ───────────────────────────────────────────────────────────────────────

pub fn proto_act_to_domain(proto: ProtoAct) -> DomainResult<Act> {
    match proto {
        ProtoAct::Create => Ok(Act::Create),
        ProtoAct::Read => Ok(Act::Read),
        ProtoAct::Write => Ok(Act::Write),
        ProtoAct::Delete => Ok(Act::Delete),
        ProtoAct::All => Ok(Act::All),
    }
}

pub fn domain_act_to_proto(act: &Act) -> ProtoAct {
    match act {
        Act::Create => ProtoAct::Create,
        Act::Read => ProtoAct::Read,
        Act::Write => ProtoAct::Write,
        Act::Delete => ProtoAct::Delete,
        Act::All => ProtoAct::All,
    }
}

// ── Scope ─────────────────────────────────────────────────────────────────────

pub fn proto_scope_to_domain(proto: ProtoScope) -> DomainResult<Scope> {
    let scope_type = ScopeType::try_from(proto.r#type).map_err(|_| {
        DomainError::Validation(format!("Invalid ScopeType value: {}", proto.r#type))
    })?;

    match scope_type {
        ScopeType::ScopeSystem => Ok(Scope::System),
        ScopeType::ScopeAll => Ok(Scope::All),
        ScopeType::ScopeOrg => {
            let id = proto.id.ok_or_else(|| {
                DomainError::Validation("ScopeOrg requires an organization id".to_string())
            })?;
            Ok(Scope::Org(OrganizationId::new(id)))
        }
        ScopeType::ScopeProject => {
            let id = proto.id.ok_or_else(|| {
                DomainError::Validation("ScopeProject requires a project id".to_string())
            })?;
            Ok(Scope::Project(ProjectId::new(id)))
        }
        ScopeType::ScopeUser => {
            let id = proto.id.ok_or_else(|| {
                DomainError::Validation("ScopeUser requires a user id".to_string())
            })?;
            Ok(Scope::User(UserId::new(id)))
        }
    }
}

pub fn domain_scope_to_proto(scope: &Scope) -> ProtoScope {
    match scope {
        Scope::System => ProtoScope {
            r#type: ScopeType::ScopeSystem.into(),
            id: None,
        },
        Scope::All => ProtoScope {
            r#type: ScopeType::ScopeAll.into(),
            id: None,
        },
        Scope::Org(id) => ProtoScope {
            r#type: ScopeType::ScopeOrg.into(),
            id: Some(id.to_string()),
        },
        Scope::Project(id) => ProtoScope {
            r#type: ScopeType::ScopeProject.into(),
            id: Some(id.to_string()),
        },
        Scope::User(id) => ProtoScope {
            r#type: ScopeType::ScopeUser.into(),
            id: Some(id.to_string()),
        },
    }
}

// ── Resource ─────────────────────────────────────────────────────────────────

pub fn proto_resource_to_domain(proto: ProtoResource) -> DomainResult<Resource> {
    let resource_type = ResourceType::try_from(proto.r#type).map_err(|_| {
        DomainError::Validation(format!("Invalid ResourceType value: {}", proto.r#type))
    })?;

    match (resource_type, proto.id) {
        (ResourceType::ResourceUser, None) => Ok(Resource::User(Target::All)),
        (ResourceType::ResourceUser, Some(id)) => {
            Ok(Resource::User(Target::Single(UserId::new(id))))
        }
        (ResourceType::ResourceProject, None) => Ok(Resource::Project(Target::All)),
        (ResourceType::ResourceProject, Some(id)) => {
            Ok(Resource::Project(Target::Single(ProjectId::new(id))))
        }
        (ResourceType::ResourceOrganization, None) => Ok(Resource::Organization(Target::All)),
        (ResourceType::ResourceOrganization, Some(id)) => Ok(Resource::Organization(
            Target::Single(OrganizationId::new(id)),
        )),
        (ResourceType::ResourcePipeline, None) => Ok(Resource::Pipeline(Target::All)),
        (ResourceType::ResourcePipeline, Some(id)) => {
            Ok(Resource::Pipeline(Target::Single(PipelineId::new(id))))
        }
        (ResourceType::ResourceAll, _) => Ok(Resource::All),
        (ResourceType::ResourceJob, None) => Ok(Resource::Job(Target::All)),
        (ResourceType::ResourceJob, Some(id)) => {
            Ok(Resource::Job(Target::Single(JobId::new(id))))
        }
    }
}

pub fn domain_resource_to_proto(resource: &Resource) -> ProtoResource {
    match resource {
        Resource::All => ProtoResource {
            r#type: ResourceType::ResourceAll.into(),
            id: None,
        },
        Resource::User(Target::All) => ProtoResource {
            r#type: ResourceType::ResourceUser.into(),
            id: None,
        },
        Resource::User(Target::Single(id)) => ProtoResource {
            r#type: ResourceType::ResourceUser.into(),
            id: Some(id.to_string()),
        },
        Resource::Project(Target::All) => ProtoResource {
            r#type: ResourceType::ResourceProject.into(),
            id: None,
        },
        Resource::Project(Target::Single(id)) => ProtoResource {
            r#type: ResourceType::ResourceProject.into(),
            id: Some(id.to_string()),
        },
        Resource::Organization(Target::All) => ProtoResource {
            r#type: ResourceType::ResourceOrganization.into(),
            id: None,
        },
        Resource::Organization(Target::Single(id)) => ProtoResource {
            r#type: ResourceType::ResourceOrganization.into(),
            id: Some(id.to_string()),
        },
        Resource::Pipeline(Target::All) => ProtoResource {
            r#type: ResourceType::ResourcePipeline.into(),
            id: None,
        },
        Resource::Pipeline(Target::Single(id)) => ProtoResource {
            r#type: ResourceType::ResourcePipeline.into(),
            id: Some(id.to_string()),
        },
        Resource::Job(Target::All) => ProtoResource {
            r#type: ResourceType::ResourceJob.into(),
            id: None,
        },
        Resource::Job(Target::Single(id)) => ProtoResource {
            r#type: ResourceType::ResourceJob.into(),
            id: Some(id.to_string()),
        },
    }
}
