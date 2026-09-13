//! The extension contract between the Scylla core and an edition binary.
//!
//! An edition (the Community binary in this repository, or a private
//! Enterprise build) implements these traits and registers the implementations
//! in an [`Extensions`] registry at startup. The core looks its extension
//! points up by trait and never knows which edition it is running in.
//!
//! This crate deliberately depends on no other workspace crate: only the traits
//! and the minimal types that appear in their signatures live here, so an
//! implementation compiles outside this repository against a pinned tag
//! without pulling the domain model, the database or the gRPC stack.
//!
//! Adding an extension point is adding a trait here. Nothing else in this crate
//! changes: the registry is keyed by trait, so no field, method or parameter
//! has to be added anywhere for a new point to be registered or looked up.

pub mod quota;

pub use quota::{QuotaDecision, QuotaError, QuotaPolicy, QuotaUsage, Resource};

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// The implementations an edition provides, keyed by extension trait.
///
/// One entry per extension point: registering a second implementation of the
/// same trait replaces the first. Cheap to clone (every entry is an `Arc`),
/// so it can be handed to every service that needs it. A point that was not
/// registered is simply absent; the core supplies its default.
#[derive(Clone, Default)]
pub struct Extensions {
    entries: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Extensions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register `implementation` as the edition's implementation of `T`,
    /// replacing any previous one.
    ///
    /// `T` is the extension trait, so the call names it:
    /// `extensions.insert::<dyn QuotaPolicy>(Arc::new(MyQuota))`.
    pub fn insert<T: ?Sized + Send + Sync + 'static>(&mut self, implementation: Arc<T>) {
        self.entries
            .insert(TypeId::of::<Arc<T>>(), Arc::new(implementation));
    }

    /// Builder-style [`Extensions::insert`].
    #[must_use]
    pub fn with<T: ?Sized + Send + Sync + 'static>(mut self, implementation: Arc<T>) -> Self {
        self.insert(implementation);
        self
    }

    /// The registered implementation of `T`, if the edition provided one.
    #[must_use]
    pub fn get<T: ?Sized + Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        self.entries
            .get(&TypeId::of::<Arc<T>>())
            .and_then(|entry| entry.downcast_ref::<Arc<T>>())
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait Greeter: Send + Sync {
        fn greet(&self) -> &'static str;
    }
    struct Hello;
    impl Greeter for Hello {
        fn greet(&self) -> &'static str {
            "hello"
        }
    }
    struct Hi;
    impl Greeter for Hi {
        fn greet(&self) -> &'static str {
            "hi"
        }
    }

    #[test]
    fn an_unregistered_point_is_absent() {
        assert!(Extensions::new().get::<dyn Greeter>().is_none());
    }

    #[test]
    fn a_registered_point_is_found_by_its_trait() {
        let extensions = Extensions::new().with::<dyn Greeter>(Arc::new(Hello));
        assert_eq!(extensions.get::<dyn Greeter>().unwrap().greet(), "hello");
    }

    #[test]
    fn registering_the_same_point_twice_replaces_it() {
        let extensions = Extensions::new()
            .with::<dyn Greeter>(Arc::new(Hello))
            .with::<dyn Greeter>(Arc::new(Hi));
        assert_eq!(extensions.get::<dyn Greeter>().unwrap().greet(), "hi");
    }

    #[test]
    fn a_clone_shares_the_implementations() {
        let extensions = Extensions::new().with::<dyn Greeter>(Arc::new(Hello));
        let copy = extensions.clone();
        assert!(Arc::ptr_eq(
            &extensions.get::<dyn Greeter>().unwrap(),
            &copy.get::<dyn Greeter>().unwrap()
        ));
    }
}
