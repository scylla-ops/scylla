pub mod repository;

pub use repository::{PgAgentRepository, insert};

#[cfg(test)]
mod tests;
