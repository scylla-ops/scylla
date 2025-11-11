use crate::domain::errors::{DomainError, DomainResult};
use std::fmt;

/// GlobalRole value object with validation
///
/// Represents a user's global role across the entire system.
/// Used for permissions in the global domain ("*").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserGlobalRole {
    inner: UserGlobalRoleInner,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum UserGlobalRoleInner {
    Admin,
    User,
}

impl UserGlobalRole {
    /// Create a new GlobalRole from string with validation
    pub fn new(value: impl Into<String>) -> DomainResult<Self> {
        let value = value.into();
        let trimmed = value.trim().to_lowercase();

        let inner = match trimmed.as_str() {
            "admin" => UserGlobalRoleInner::Admin,
            "user" => UserGlobalRoleInner::User,
            _ => {
                return Err(DomainError::validation(format!(
                    "Invalid global role: {}. Must be 'admin' or 'user'",
                    value
                )));
            }
        };

        Ok(Self { inner })
    }

    /// Create an admin role
    pub fn admin() -> Self {
        Self {
            inner: UserGlobalRoleInner::Admin,
        }
    }

    /// Create a user role (default for new users)
    pub fn user() -> Self {
        Self {
            inner: UserGlobalRoleInner::User,
        }
    }

    /// Get the role as a string slice
    pub fn as_str(&self) -> &'static str {
        match self.inner {
            UserGlobalRoleInner::Admin => "admin",
            UserGlobalRoleInner::User => "user",
        }
    }

    /// Convert to string
    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }

    /// Check if the role is admin
    pub fn is_admin(&self) -> bool {
        matches!(self.inner, UserGlobalRoleInner::Admin)
    }

    /// Check if the role is user
    pub fn is_user(&self) -> bool {
        matches!(self.inner, UserGlobalRoleInner::User)
    }

    /// Check if the role has admin privileges
    pub fn has_admin_privileges(&self) -> bool {
        self.is_admin()
    }
}

impl fmt::Display for UserGlobalRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AsRef<str> for UserGlobalRole {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Default for UserGlobalRole {
    fn default() -> Self {
        Self::user()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_role_creation() {
        assert!(UserGlobalRole::new("admin").is_ok());
        assert!(UserGlobalRole::new("user").is_ok());
        assert!(UserGlobalRole::new("invalid").is_err());
    }

    #[test]
    fn test_global_role_case_insensitive() {
        assert_eq!(UserGlobalRole::new("ADMIN").unwrap().as_str(), "admin");
        assert_eq!(UserGlobalRole::new("USER").unwrap().as_str(), "user");
    }

    #[test]
    fn test_global_role_methods() {
        let admin = UserGlobalRole::admin();
        let user = UserGlobalRole::user();

        assert!(admin.is_admin());
        assert!(!admin.is_user());
        assert!(admin.has_admin_privileges());

        assert!(!user.is_admin());
        assert!(user.is_user());
        assert!(!user.has_admin_privileges());
    }

    #[test]
    fn test_global_role_default() {
        assert_eq!(UserGlobalRole::default().as_str(), "user");
    }
}
