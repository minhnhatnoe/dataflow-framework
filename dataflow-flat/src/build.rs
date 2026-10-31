//! Configs and the construction pass. These are the only places with runtime
//! lookups (a `TypeId` map); the run itself has none.

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
}

/// Construction state: hands each constructor its config (supplied, or the
/// default) and records the construction order.
#[doc(hidden)]
pub struct ConstructPass {
    pub(crate) configs: HashMap<TypeId, ConfigEntry>,
    pub(crate) order: Vec<&'static str>,
}

impl ConstructPass {
    pub(crate) fn take_config<C: Component>(&mut self) -> C::Config {
        match self.configs.remove(&TypeId::of::<C>()) {
            None => C::Config::default(),
            Some(entry) => *entry.config.downcast::<C::Config>().expect("keyed by component type"),
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
        if !self.unused.is_empty() {
            writeln!(f, "config supplied for components not in the graph: {}", self.unused.join(", "))?;
        }
        Ok(())
    }
}

impl std::error::Error for BuildError {}
