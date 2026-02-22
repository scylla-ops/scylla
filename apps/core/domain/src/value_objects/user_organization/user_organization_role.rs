use crate::errors::{DomainError, DomainResult};
use std::fmt;
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

/// UserOrganizationRole value object with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub struct UserOrganizationRole {
    inner: UserOrganizationRoleInner,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
enum UserOrganizationRoleInner {
    Owner,
    Admin,
    Member,
}

impl UserOrganizationRole {
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim().to_lowercase();

        let inner = match trimmed.as_str() {
            "owner" => UserOrganizationRoleInner::Owner,
            "admin" => UserOrganizationRoleInner::Admin,
            "member" => UserOrganizationRoleInner::Member,
            _ => {
                return Err(DomainError::validation(format!(
                    "Invalid user organization role: {}",
                    value
                )));
            }
        };

        Ok(Self { inner })
    }

    pub fn owner() -> Self {
        Self {
            inner: UserOrganizationRoleInner::Owner,
        }
    }

    pub fn admin() -> Self {
        Self {
            inner: UserOrganizationRoleInner::Admin,
        }
    }

    pub fn member() -> Self {
        Self {
            inner: UserOrganizationRoleInner::Member,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self.inner {
            UserOrganizationRoleInner::Owner => "owner",
            UserOrganizationRoleInner::Admin => "admin",
            UserOrganizationRoleInner::Member => "member",
        }
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    pub fn is_owner(&self) -> bool {
        matches!(self.inner, UserOrganizationRoleInner::Owner)
    }

    pub fn is_admin(&self) -> bool {
        matches!(self.inner, UserOrganizationRoleInner::Admin)
    }

    pub fn is_user(&self) -> bool {
        matches!(self.inner, UserOrganizationRoleInner::Member)
    }

    pub fn is_member(&self) -> bool {
        matches!(self.inner, UserOrganizationRoleInner::Member)
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

impl fmt::Display for UserOrganizationRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AsRef<str> for UserOrganizationRole {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_organization_role_creation() {
        assert!(UserOrganizationRole::new("owner").is_ok());
        assert!(UserOrganizationRole::new("admin").is_ok());
        assert!(UserOrganizationRole::new("member").is_ok());
        assert!(UserOrganizationRole::new("invalid").is_err());
    }

    #[test]
    fn test_user_organization_role_case_insensitive() {
        assert_eq!(
            UserOrganizationRole::new("ADMIN").unwrap().as_str(),
            "admin"
        );
        assert_eq!(
            UserOrganizationRole::new("member").unwrap().as_str(),
            "member"
        );
    }

    #[test]
    fn test_user_organization_role_methods() {
        let owner = UserOrganizationRole::owner();
        let admin = UserOrganizationRole::admin();
        let user = UserOrganizationRole::member();

        assert!(owner.is_owner());
        assert!(!owner.is_admin());
        assert!(!owner.is_user());
        assert!(!owner.is_member());
        assert!(owner.has_owner_privileges());
        assert!(owner.can_manage_users());
        assert!(owner.can_view());
        assert!(owner.can_edit());

        assert!(admin.is_admin());
        assert!(!admin.is_user());
        assert!(!admin.is_member());
        assert!(admin.has_admin_privileges());
        assert!(admin.can_manage_users());
        assert!(admin.can_view());
        assert!(admin.can_edit());

        assert!(!user.is_admin());
        assert!(user.is_user());
        assert!(user.is_member());
        assert!(!user.has_admin_privileges());
        assert!(!user.can_manage_users());
        assert!(user.can_view());
        assert!(user.can_edit());
    }
}
