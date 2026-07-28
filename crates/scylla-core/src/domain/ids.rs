use std::fmt;

/// Generate a type-safe ID wrapper for a domain entity.
///
/// Produces the newtype, generation API, common trait impls
/// (`Display`, `From<String>`, `From<&str>`, `From<Self> for String`, `AsRef<str>`)
/// and the `EntityId` marker.
///
/// IDs deliberately carry no `sqlx` integration: every query passes them as
/// `&str` via `as_str()`, so binding them directly is never needed and the
/// domain stays free of any database dependency.
macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, ::serde::Serialize, ::serde::Deserialize)]
        pub struct $name(String);

        impl $name {
            /// Wrap an existing string as this ID. No validation.
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            /// Generate a fresh lowercase ULID.
            pub fn generate() -> Self {
                Self(ulid::Ulid::new().to_string().to_lowercase())
            }

            pub fn as_str(&self) -> &str {
                &self.0
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

        impl EntityId for $name {}
        impl EntityId for &$name {}
    };
}

pub trait EntityId: fmt::Display + AsRef<str> + Send + Sync {}

// Define ID types for all domain entities
define_id!(UserId);
define_id!(OrganizationId);
define_id!(ProjectId);
define_id!(PipelineId);
define_id!(JobId);
define_id!(JobLogId);
define_id!(SessionId);
define_id!(AppId);
define_id!(AppTokenId);
define_id!(AppCredentialId);
define_id!(InvitationId);
define_id!(SecretId);
define_id!(TriggerId);
