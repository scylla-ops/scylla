use serde::{Deserialize, Serialize};
use std::fmt;

/// Macro to generate type-safe ID wrappers for domain entities
macro_rules! define_id {
    ($name:ident, $table:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(String);

        impl $name {
            /// Create a new ID from a string
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            /// Generate a new ID
            pub fn generate() -> Self {
                Self(ulid::Ulid::new().to_string().to_lowercase())
            }

            /// Get the ID as a string slice
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Get the table name for this ID type
            pub fn table_name() -> &'static str {
                $table
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self::new(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self::new(s)
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }
    };
}

// Define ID types for all domain entities
define_id!(UserId, "users");
define_id!(OrganizationId, "organizations");
define_id!(ProjectId, "projects");
define_id!(PipelineId, "pipelines");
define_id!(JobId, "jobs");
define_id!(UserOrganizationId, "user_organization");
define_id!(UserProjectId, "user_project");
define_id!(BlacklistId, "blacklist");
