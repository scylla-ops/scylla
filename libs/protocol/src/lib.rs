pub mod job;
pub mod pipeline;
pub mod pipeline_loader;
pub mod shell;

pub use serde;
pub use serde::*;
pub use serde_json;
pub use toml;
pub use uuid;

pub use tonic;

pub mod services {
    tonic::include_proto!("user");
    tonic::include_proto!("auth");
    pub mod orchestrator {
        tonic::include_proto!("orchestrator");
    }
    pub mod pipeline {
        tonic::include_proto!("pipeline");
        pub mod snapshot {
            tonic::include_proto!("pipeline.snapshot");
        }
    }
    pub mod job {
        tonic::include_proto!("job");
    }
    pub mod organization {
        tonic::include_proto!("organization");
    }
    pub mod project {
        tonic::include_proto!("project");
    }

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("services_descriptor");
}
