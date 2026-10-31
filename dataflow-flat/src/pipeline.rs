//! The user-facing pipeline: build once, run many times.

use std::marker::PhantomData;

use crate::build::{BuildError, Configs, ConstructPass};
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
        // Configs for components outside the graph are reported before any
        // constructor runs: the plan of the root says which `TypeId`s are in it.
        let mut unused: Vec<_> =
            configs.map.iter().filter(|(id, _)| !A::in_plan(**id)).map(|(_, e)| e.name()).collect();
        unused.sort_unstable();
        if !unused.is_empty() {
            return Err(BuildError { unused });
        }

        let mut pass = ConstructPass { configs: configs.map, order: Vec::new() };
        let states = A::make(&mut pass);
        Ok(Pipeline { states, order: pass.order, _root: PhantomData })
    }

    /// Update every component once, in the documented order, and return the
    /// root's output.
    #[inline(always)]
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
