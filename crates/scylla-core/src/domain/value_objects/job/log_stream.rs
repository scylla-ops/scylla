use serde::{Deserialize, Serialize};
#[cfg(feature = "surrealdb")]
use surrealdb_types::SurrealValue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "surrealdb", derive(SurrealValue))]
pub enum LogStream {
    Stdout,
    Stderr,
}
