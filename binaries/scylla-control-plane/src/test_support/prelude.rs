//! One-stop import for test code.
//! `use scylla_control_plane::test_support::prelude::*;` brings every builder,
//! shortcut, double and seeder into scope.

pub use scylla_core::test_support::prelude::*;

pub use super::scenarios::*;
pub use super::seed::*;
