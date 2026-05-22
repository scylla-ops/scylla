#![allow(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

pub mod services {
    pub mod common {
        tonic::include_proto!("common");
    }
    pub mod config {
        tonic::include_proto!("config");
    }
    pub mod auth {
        tonic::include_proto!("auth");
    }
    pub mod registration {
        tonic::include_proto!("registration");
    }
    pub mod invitation {
        tonic::include_proto!("invitation");
    }
    pub mod oauth {
        tonic::include_proto!("oauth");
    }
    pub mod user {
        tonic::include_proto!("user");
    }
    pub mod organization {
        tonic::include_proto!("organization");
    }
    pub mod project {
        tonic::include_proto!("project");
    }
    pub mod pipeline {
        tonic::include_proto!("pipeline");
    }
    pub mod job {
        tonic::include_proto!("job");
    }
    pub mod permission {
        tonic::include_proto!("permission");
    }
    pub mod app {
        tonic::include_proto!("app");
    }
    pub mod agent {
        tonic::include_proto!("agent");
    }
    pub mod agent_admin {
        tonic::include_proto!("agent_admin");
    }

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("services_descriptor");
}
