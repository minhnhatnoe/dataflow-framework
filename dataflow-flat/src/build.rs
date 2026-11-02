//! Configs and the construction pass. These are the only places with runtime
//! lookups (a `TypeId` map); a push has none.

use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt;

use crate::component::Component;

/// A config supplied for a component, with its name for error messages.
pub(crate) struct ConfigEntry {
    config: Box<dyn Any>,
    name: &'static str,
}

impl ConfigEntry {
    pub(crate) fn name(&self) -> &'static str {
        self.name
    }
}

/// Owned configs, keyed by the `TypeId` of the component (not of the config),
/// so two components or two instantiations of one generic component never
/// share a config by accident. The key is exact: it is the same identity the
/// compile-time plan uses.
#[derive(Default)]
pub struct Configs {
    pub(crate) map: HashMap<TypeId, ConfigEntry>,
}

impl Configs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<C: Component>(mut self, config: C::Config) -> Self {
        let entry = ConfigEntry { config: Box::new(config), name: type_name::<C>() };
        self.map.insert(TypeId::of::<C>(), entry);
        self
    }

    /// The config of a handle component ([`HandleComponent`](crate::HandleComponent)).
    pub fn with_handle<C: crate::component::HandleComponent>(mut self, config: C::Config) -> Self {
        let entry = ConfigEntry { config: Box::new(config), name: type_name::<C>() };
        self.map.insert(TypeId::of::<C>(), entry);
        self
    }
}

/// Construction state: hands each constructor its config (supplied, or the
/// default) and records the construction order.
#[doc(hidden)]
pub struct ConstructPass {
    pub(crate) configs: HashMap<TypeId, ConfigEntry>,
    pub(crate) order: Vec<&'static str>,
    /// Listener requests collected by the first pass, by listener `TypeId`.
    pub(crate) requests: HashMap<TypeId, Box<dyn Any>>,
    /// (listener, selector) pairs already requested.
    pub(crate) seen: std::collections::HashSet<(TypeId, TypeId)>,
}

impl ConstructPass {
    pub(crate) fn new(configs: HashMap<TypeId, ConfigEntry>) -> Self {
        ConstructPass { configs, order: Vec::new(), requests: HashMap::new(), seen: Default::default() }
    }
    /// The requests collected for listener `C` (empty if nobody reads it).
    pub fn take_requests<C: 'static, R: Default + 'static>(&mut self) -> R {
        match self.requests.remove(&TypeId::of::<C>()) {
            None => R::default(),
            Some(b) => *b.downcast::<R>().expect("keyed by listener"),
        }
    }
    pub(crate) fn take_config<C: crate::node::Comp>(&mut self) -> C::Cfg {
        match self.configs.remove(&TypeId::of::<C>()) {
            None => C::Cfg::default(),
            Some(entry) => *entry.config.downcast::<C::Cfg>().expect("keyed by component type"),
        }
    }
}

/// Why [`Pipeline::build`](crate::Pipeline::build) failed.
#[derive(Debug)]
pub struct BuildError {
    /// Components that were given a config but are not in the graph.
    pub unused: Vec<&'static str>,
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "config supplied for components not in the graph: {}", self.unused.join(", "))
    }
}

impl std::error::Error for BuildError {}
