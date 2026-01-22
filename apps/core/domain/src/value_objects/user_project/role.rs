use crate::errors::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};
use std::fmt;

/// UserProjectRole value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserProjectRole {
    inner: UserProjectRoleInner,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum UserProjectRoleInner {
    Owner,
    Admin,
    Member,
}

impl UserProjectRole {
    /// Create a new UserProjectRole from string with validation
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

    /// Create an owner role
    pub fn owner() -> Self {
        Self {
            inner: UserProjectRoleInner::Owner,
        }
    }

    /// Create an admin role
    pub fn admin() -> Self {
        Self {
            inner: UserProjectRoleInner::Admin,
        }
    }

    /// Create a user role
    pub fn user() -> Self {
        Self {
            inner: UserProjectRoleInner::Member,
        }
    }

    /// Get the role as a string slice
    pub fn as_str(&self) -> &'static str {
        match self.inner {
            UserProjectRoleInner::Owner => "owner",
            UserProjectRoleInner::Admin => "admin",
            UserProjectRoleInner::Member => "member",
        }
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    /// Check if the role is owner
    pub fn is_owner(&self) -> bool {
        matches!(self.inner, UserProjectRoleInner::Owner)
    }

    /// Check if the role is admin
    pub fn is_admin(&self) -> bool {
        matches!(self.inner, UserProjectRoleInner::Admin)
    }

    /// Check if the role is user
    pub fn is_user(&self) -> bool {
        matches!(self.inner, UserProjectRoleInner::Member)
    }

    /// Check if the role is a member
    pub fn is_member(&self) -> bool {
        matches!(self.inner, UserProjectRoleInner::Member)
    }

    /// Check if the role has owner privileges
    pub fn has_owner_privileges(&self) -> bool {
        self.is_owner()
    }

    /// Check if the role has admin privileges
    pub fn has_admin_privileges(&self) -> bool {
        self.is_admin()
    }

    /// Check if the role can manage users
    pub fn can_manage_users(&self) -> bool {
        self.is_admin() || self.is_owner()
    }

    /// Check if the role can view content
    pub fn can_view(&self) -> bool {
        true // all roles can view
    }

    /// Check if the role can edit content
    pub fn can_edit(&self) -> bool {
        self.is_admin() || self.is_owner() || self.is_member()
    }
}

impl fmt::Display for UserProjectRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
