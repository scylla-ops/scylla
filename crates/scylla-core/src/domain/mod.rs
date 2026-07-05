// Domain state machines are modelled as enums whose variants carry only the data
// valid in that state. Forbid `_` catch-all arms across the domain so that adding
// a variant fails the build everywhere it is not yet handled, instead of silently
// falling through.
#![deny(clippy::wildcard_enum_match_arm)]

pub mod clock;
pub mod entities;
pub mod errors;
pub mod value_objects;
