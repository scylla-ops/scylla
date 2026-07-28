//! The domain model, organised by subject.
//!
//! **Where does a new file go?** Into `domain/<subject>/` as soon as the concept
//! it describes only makes sense when talking about that subject. It stays at
//! the root of `domain/` only if it serves every subject without describing any:
//! `errors.rs`, `ids.rs`, `clock.rs`, and nothing else.
//!
//! The previous split was `entities/` versus `value_objects/`, which answered
//! "what kind of type is this?" instead of "what is this about". It scattered
//! the pipeline across three places and left `dag.rs`, `clock.rs` and the job
//! event with no home at all. Grouping by subject means everything a pipeline is
//! made of sits in one directory, and `Pipeline` can own its DAG planner without
//! two directories referring back to each other.

// Domain state machines are modelled as enums whose variants carry only the data
// valid in that state. Forbid `_` catch-all arms across the domain so that adding
// a variant fails the build everywhere it is not yet handled, instead of silently
// falling through.
#![deny(clippy::wildcard_enum_match_arm)]

// Shared by every subject, describing none.
pub mod clock;
pub mod errors;
pub mod ids;

// One module per subject. A subject with a single file has no directory.
pub mod agent;
pub mod app;
pub mod invitation;
pub mod job;
pub mod organization;
pub mod permission;
pub mod pipeline;
pub mod project;
pub mod role;
pub mod secret;
pub mod session;
pub mod trigger;
pub mod user;
