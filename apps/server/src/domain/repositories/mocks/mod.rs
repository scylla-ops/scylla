//! Mock implementations for repository traits
//!
//! This module provides reusable mock implementations for all repository traits
//! using the mockall crate. These mocks are used in use case unit tests.
//!
//! # Organization
//!
//! Each repository has its own file containing:
//! - A simplified mock trait (to work around mockall lifetime limitations)
//! - A mockall-generated mock implementation
//! - An adapter that bridges the mock to the real repository trait
//!
//! # Usage in Tests
//!
//! ```rust
//! #[cfg(test)]
//! mod tests {
//!     use super::*;
//!     use crate::domain::repositories::mocks::{
//!         MockUserRepository,
//!         MockUserRepositoryAdapter,
//!     };
//!     use mockall::predicate::*;
//!
//!     #[tokio::test]
//!     async fn test_my_use_case() {
//!         // Create the mock
//!         let mut mock_user_repo = MockUserRepository::new();
//!
//!         // Set expectations
//!         mock_user_repo
//!             .expect_create()
//!             .returning(|user| Ok(user.clone()));
//!
//!         // Wrap in adapter for use with the real trait
//!         let use_case = MyUseCase::new(
//!             Arc::new(MockUserRepositoryAdapter {
//!                 inner: mock_user_repo
//!             })
//!         );
//!
//!         // Execute and assert...
//!     }
//! }
//! ```

mod organization_repository;
mod project_repository;
mod user_organization_repository;
mod user_project_repository;
mod user_repository;

// Re-export all mocks and adapters
pub use organization_repository::{
    MockOrganizationRepository, MockOrganizationRepositoryAdapter, OrganizationRepositoryMock,
};
pub use project_repository::{
    MockProjectRepository, MockProjectRepositoryAdapter, ProjectRepositoryMock,
};
pub use user_organization_repository::{
    MockUserOrganizationRepository, MockUserOrganizationRepositoryAdapter,
    UserOrganizationRepositoryMock,
};
pub use user_project_repository::{
    MockUserProjectRepository, MockUserProjectRepositoryAdapter, UserProjectRepositoryMock,
};
pub use user_repository::{MockUserRepository, MockUserRepositoryAdapter, UserRepositoryMock};
