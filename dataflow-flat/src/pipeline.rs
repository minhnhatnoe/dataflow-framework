//! The user-facing pipeline: build once, run many times.

use std::collections::HashMap;
use std::marker::PhantomData;

use crate::build::{BuildError, ConfigPass, ConstructPass, Configs};
use crate::walk::Root;

/// The root component `A` and everything it depends on, constructed and ready
/// to run.
///
/// # Execution order
///
/// `run` updates components in a depth-first post-order that follows the
/// subscription listing order, exactly as documented on
/// [`Component`](crate::Component): for each component, its dependencies'
/// blocks in listing order, then the component. The order is guaranteed and
/// checked by `tests/order.rs`.
///
/// # Lifetime of outputs
///
/// Nothing subscribes to the root, so its output is never stored: `run`
/// returns it. The outputs of all other components live only during `run` and
/// are dropped before it returns. Components keep their own state between runs.
pub struct Pipeline<A: Root> {
    states: A::States,
    order: Vec<&'static str>,
    _root: PhantomData<A>,
}

impl<A: Root> Pipeline<A> {
    /// Construct the root and everything upstream of it, each component once,
    /// dependencies first. Components without a config entry get
    /// `Config::default()`; a config supplied for a component outside the
    /// graph is a [`BuildError`].
    pub fn build(configs: Configs) -> Result<Self, BuildError> {
        let mut pass1 = ConfigPass { supplied: configs.map, resolved: HashMap::new(), ids: HashMap::new() };
        pass1.visit::<A>();

        let mut unused: Vec<_> = pass1.supplied.values().map(|(_, name)| *name).collect();
        unused.sort_unstable();
        if !unused.is_empty() {
            return Err(BuildError { unused });
        }

        let mut pass2 = ConstructPass { configs: pass1.resolved, order: Vec::new() };
        let states = A::make(&mut pass2);
        Ok(Pipeline { states, order: pass2.order, _root: PhantomData })
    }

    /// Update every component once, in the documented order, and return the
    /// root's output.
    #[inline]
    pub fn run(&mut self) -> A::Output {
        A::run(&mut self.states)
    }

    /// Component names in construction order (which is also the order of
    /// each run's first updates: dependencies before dependents).
    pub fn order(&self) -> &[&'static str] {
        &self.order
    }

    /// Number of components, the root included.
    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
}
