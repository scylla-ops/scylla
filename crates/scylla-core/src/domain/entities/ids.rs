use std::fmt;
use std::str::FromStr;

/// Macro to generate type-safe ID wrappers for domain entities
macro_rules! define_id {
    ($name:ident, $table:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

        impl FromStr for $name {
            type Err = IdParseError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                if s.trim().is_empty() {
                    Err(IdParseError {
                        value: s.to_string(),
                    })
                } else {
                    Ok(Self::from(s.to_string()))
                }
            }
        }

        impl EntityId for $name {}
        impl EntityId for &$name {}

        #[cfg(feature = "surrealdb")]
        impl surrealdb_types::SurrealValue for $name {
            fn kind_of() -> surrealdb_types::Kind {
                surrealdb_types::Kind::Record(vec![surrealdb_types::Table::from($table)])
            }

            fn into_value(self) -> surrealdb_types::Value {
                surrealdb_types::Value::RecordId(surrealdb_types::RecordId::new($table, self.0))
            }

            fn from_value(value: surrealdb_types::Value) -> Result<Self, surrealdb_types::Error> {
                match value {
                    surrealdb_types::Value::RecordId(record_id) => match record_id.key {
                        surrealdb_types::RecordIdKey::String(s) => Ok(Self(s)),
                        other => Ok(Self(format!("{:?}", other))),
                    },
                    other => Err(surrealdb_types::ConversionError::from_value(
                        Self::kind_of(),
                        &other,
                    )
                    .into()),
                }
            }
        }
    };
}

#[derive(Debug)]
pub struct IdParseError {
    value: String,
}

impl fmt::Display for IdParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid id format: {}", self.value)
    }
}

impl std::error::Error for IdParseError {}

pub trait EntityId: fmt::Display + AsRef<str> + Send + Sync {}

// Define ID types for all domain entities
define_id!(UserId, "users");
define_id!(OrganizationId, "organizations");
define_id!(ProjectId, "projects");
define_id!(PipelineId, "pipelines");
define_id!(JobId, "jobs");
define_id!(JobLogId, "job_logs");
define_id!(UserOrganizationId, "user_organization");
define_id!(UserProjectId, "user_project");
define_id!(SessionId, "sessions");
