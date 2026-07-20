#![allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

//! Generated gRPC types and service stubs for every Scylla contract.
//!
//! The module tree mirrors the proto packages: `scylla.job.v1` is reachable at
//! [`job::v1`]. prost resolves cross-package references relative to the shared
//! `scylla` prefix, so the packages sit at the crate root with the prefix
//! stripped — `scylla_protocol::job::v1::Job`, not `scylla_protocol::scylla::…`.
//!
//! These are wire DTOs, not domain types: message fields arrive as `Option`,
//! enums as `i32`, ids as wrapper messages. Convert at the boundary (the
//! `scylla-api` mappers) and keep them out of business logic.

/// Leaf value objects shared by every package: ids, `Email`, pagination.
pub mod common {
    pub mod v1 {
        tonic::include_proto!("scylla.common.v1");
    }
}

/// The execution contract shared by the side that defines work and the side
/// that runs it.
pub mod exec {
    pub mod v1 {
        tonic::include_proto!("scylla.exec.v1");
    }
}

pub mod auth {
    pub mod v1 {
        tonic::include_proto!("scylla.auth.v1");
    }
}

pub mod registration {
    pub mod v1 {
        tonic::include_proto!("scylla.registration.v1");
    }
}

pub mod invitation {
    pub mod v1 {
        tonic::include_proto!("scylla.invitation.v1");
    }
}

pub mod oauth {
    pub mod v1 {
        tonic::include_proto!("scylla.oauth.v1");
    }
}

pub mod user {
    pub mod v1 {
        tonic::include_proto!("scylla.user.v1");
    }
}

pub mod organization {
    pub mod v1 {
        tonic::include_proto!("scylla.organization.v1");
    }
}

pub mod project {
    pub mod v1 {
        tonic::include_proto!("scylla.project.v1");
    }
}

/// Authorization: the permission vocabulary plus the policy, grant and role
/// services. One bounded context, four files, one package.
pub mod authz {
    pub mod v1 {
        tonic::include_proto!("scylla.authz.v1");
    }
}

pub mod pipeline {
    pub mod v1 {
        tonic::include_proto!("scylla.pipeline.v1");
    }
}

pub mod secret {
    pub mod v1 {
        tonic::include_proto!("scylla.secret.v1");
    }
}

pub mod job {
    pub mod v1 {
        tonic::include_proto!("scylla.job.v1");
    }
}

pub mod app {
    pub mod v1 {
        tonic::include_proto!("scylla.app.v1");
    }
}

/// Agents: the streaming job channel and the unary admin surface.
pub mod agent {
    pub mod v1 {
        tonic::include_proto!("scylla.agent.v1");
    }
}

pub mod trigger {
    pub mod v1 {
        tonic::include_proto!("scylla.trigger.v1");
    }
}

/// Serialized descriptors for every compiled proto, for gRPC reflection.
pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("scylla_descriptor");
