use crate::errors::{DomainError, DomainResult};
use std::fmt;
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

/// UserProjectRole value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct UserProjectRole {
    inner: UserProjectRoleInner,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
enum UserProjectRoleInner {
    Owner,
    Admin,
    Member,
}

impl UserProjectRole {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim().to_lowercase();

        let inner = match trimmed.as_str() {
            "owner" => UserProjectRoleInner::Owner,
            "admin" => UserProjectRoleInner::Admin,
            "member" => UserProjectRoleInner::Member,
            _ => {
                return Err(DomainError::validation(format!(
                    "Invalid user project role: {}",
                    value
                )));
            }
        };

        Ok(Self { inner })
    }

    pub fn owner() -> Self {
        Self {
            inner: UserProjectRoleInner::Owner,
        }
    }

    pub fn admin() -> Self {
        Self {
            inner: UserProjectRoleInner::Admin,
        }
    }

    pub fn member() -> Self {
        Self {
            inner: UserProjectRoleInner::Member,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self.inner {
            UserProjectRoleInner::Owner => "owner",
            UserProjectRoleInner::Admin => "admin",
            UserProjectRoleInner::Member => "member",
        }
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    pub fn is_owner(&self) -> bool {
        matches!(self.inner, UserProjectRoleInner::Owner)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.inner, UserProjectRoleInner::Admin)
    }

    pub fn is_user(&self) -> bool {
        matches!(self.inner, UserProjectRoleInner::Member)
    }

    pub fn is_member(&self) -> bool {
        matches!(self.inner, UserProjectRoleInner::Member)
    }

    pub fn has_owner_privileges(&self) -> bool {
        self.is_owner()
    }

    pub fn has_admin_privileges(&self) -> bool {
        self.is_admin()
    }

    pub fn can_manage_users(&self) -> bool {
        self.is_admin() || self.is_owner()
    }

    pub fn can_view(&self) -> bool {
        true
    }

    pub fn can_edit(&self) -> bool {
        self.is_admin() || self.is_owner() || self.is_member()
    }
}

impl AsRef<str> for UserProjectRole {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for UserProjectRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
