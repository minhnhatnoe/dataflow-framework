//! The user-facing pipeline: build once, push many times.

use std::marker::PhantomData;

use crate::build::{BuildError, Configs, ConstructPass};
use crate::sinks::Sinks;

/// A pipeline: the root type `R`, the sinks `S`, and everything upstream of
/// the sinks, constructed and ready to take pushes.
///
/// ```
/// use dataflow_flat::{Component, ComponentSubscription, UpdateHandler, Configs, Pipeline, Root};
///
/// #[derive(Root)]
/// struct Tick;
///
/// #[derive(ComponentSubscription, UpdateHandler)]
/// #[subscribe(tick: Tick)]
/// struct Count;
/// impl Component for Count {
///     type Config = ();
///     type Output = u64;
///     fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
///         (Count, 0)
///     }
///     fn update(&mut self, _: Self::Input<'_>) -> u64 {
///         0
///     }
/// }
///
/// fn main() {
///     let mut p = Pipeline::<Tick, (Count,)>::build(Configs::new()).unwrap();
///     p.push(Tick);
/// }
/// ```
///
/// # Execution order
///
/// A push updates components in a depth-first post-order that follows the
/// subscription listing order, exactly as documented on
/// [`Component`](crate::Component): for each component, its dependencies'
/// blocks in listing order, then the component. The order is guaranteed and
/// checked by `tests/order.rs`. A component runs only when one of its edges
/// fired.
///
/// # Outputs
///
/// The output of every component is stored and persists between pushes. The
/// root's value exists only during the push.
pub struct Pipeline<R, S: Sinks<R>> {
    store: S::Store,
    order: Vec<&'static str>,
    _root: PhantomData<fn(R)>,
}

impl<R, S: Sinks<R>> Pipeline<R, S> {
    /// Construct every component upstream of the sinks, each once, in
    /// execution order; each `new` is given the first outputs of its
    /// subscriptions. Components without a config entry get
    /// `Config::default()`; a config supplied for a component outside the
    /// graph is a [`BuildError`].
    /// A `#[listener]` gets, in `new`, the requests its subscribers' selectors make.
    pub fn build(configs: Configs) -> Result<Self, BuildError> {
        // Configs for components outside the graph are reported before any
        // constructor runs: the plan says which `TypeId`s are in it.
        let mut unused: Vec<_> =
            configs.map.iter().filter(|(id, _)| !S::in_plan(**id)).map(|(_, e)| e.name()).collect();
        unused.sort_unstable();
        if !unused.is_empty() {
            return Err(BuildError { unused });
        }

        let mut pass = ConstructPass::new(configs.map);
        S::requests(&mut pass);
        let store = S::make(&mut pass);
        Ok(Pipeline { store, order: pass.order, _root: PhantomData })
    }

    /// One step: the root takes the value `root`, and every component
    /// downstream of it runs once, in the documented order, reading the
    /// stored outputs of the components that did not run.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    pub fn push(&mut self, root: R) {
        S::push(&mut self.store, root)
    }

    /// Component names in construction order (which is also the order of
    /// each step's updates: dependencies before dependents).
    pub fn order(&self) -> &[&'static str] {
        &self.order
    }

    /// Number of components.
    pub fn len(&self) -> usize {
        self.order.len()
    }

    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
}
