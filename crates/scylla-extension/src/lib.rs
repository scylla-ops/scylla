//! Depends on no workspace crate so an out-of-tree edition builds against a pinned tag.

pub mod quota;

pub use quota::{QuotaDecision, QuotaError, QuotaPolicy, QuotaUsage, Resource};

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub struct Extensions {
    entries: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Extensions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert<T: ?Sized + Send + Sync + 'static>(&mut self, implementation: Arc<T>) {
        self.entries
            .insert(TypeId::of::<Arc<T>>(), Arc::new(implementation));
    }

    #[must_use]
    pub fn with<T: ?Sized + Send + Sync + 'static>(mut self, implementation: Arc<T>) -> Self {
        self.insert(implementation);
        self
    }

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
