use std::fmt;

/// `sqlx::{Type, Encode, Decode}` impls for a String-backed newtype ID.
///
/// Kept private and invoked exclusively by [`define_id!`] so the generated
/// expansion stays small. Re-declared per type because `sqlx::Type` requires
/// concrete impls (no blanket impl via a trait).
#[cfg(feature = "postgres")]
macro_rules! impl_sqlx_for_id {
    ($name:ident) => {
        impl sqlx::Type<sqlx::Postgres> for $name {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                <String as sqlx::Type<sqlx::Postgres>>::type_info()
            }
            fn compatible(ty: &sqlx::postgres::PgTypeInfo) -> bool {
                <String as sqlx::Type<sqlx::Postgres>>::compatible(ty)
            }
        }

        impl<'q> sqlx::Encode<'q, sqlx::Postgres> for $name {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer,
            ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
                <&str as sqlx::Encode<'q, sqlx::Postgres>>::encode_by_ref(&self.0.as_str(), buf)
            }
        }

        impl<'r> sqlx::Decode<'r, sqlx::Postgres> for $name {
            fn decode(
                value: sqlx::postgres::PgValueRef<'r>,
            ) -> Result<Self, sqlx::error::BoxDynError> {
                let s = <String as sqlx::Decode<'r, sqlx::Postgres>>::decode(value)?;
                Ok(Self(s))
            }
        }
    };
}

/// Stub used when the `postgres` feature is off.
#[cfg(not(feature = "postgres"))]
macro_rules! impl_sqlx_for_id {
    ($name:ident) => {};
}

/// Generate a type-safe ID wrapper for a domain entity.
///
/// Produces the newtype, generation API, common trait impls
/// (`Display`, `From<String>`, `From<&str>`, `From<Self> for String`, `AsRef<str>`),
/// the `EntityId` marker, and `sqlx` integration (under the `postgres` feature).
macro_rules! define_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
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

        impl_sqlx_for_id!($name);
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
define_id!(UserOrganizationId);
define_id!(UserProjectId);
define_id!(SessionId);
define_id!(AppId);
define_id!(AppTokenId);
define_id!(AppCredentialId);
define_id!(CedarPolicyId);
define_id!(InvitationId);
