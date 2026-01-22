pub use serde;
pub use serde::*;
pub use serde_json;
pub use toml;
pub use tonic;

pub mod services {
    pub mod common {
        tonic::include_proto!("common");
    }

    pub mod user {
        tonic::include_proto!("user");
    }
    pub mod auth {
        tonic::include_proto!("auth");
    }
    pub mod orchestrator {
        tonic::include_proto!("orchestrator");
    }
    pub mod pipeline {
        tonic::include_proto!("pipeline");
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

    pub mod pipeline_def {
        tonic::include_proto!("pipeline_def");
    }

    pub mod job_def {
        tonic::include_proto!("job_def");
    }

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("services_descriptor");
}
