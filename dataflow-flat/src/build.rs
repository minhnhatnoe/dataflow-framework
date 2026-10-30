//! Configs and the two build passes. These are the only places with runtime
//! lookups (`TypeId` maps); the run itself has none.

use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt;

use crate::component::{Component, DepList};
use crate::plan::Described;

/// Owned configs, keyed by component type (not config type), so two
/// components or two instantiations of one generic component never share a
/// config by accident.
#[derive(Default)]
pub struct Configs {
    pub(crate) map: HashMap<TypeId, (Box<dyn Any>, &'static str)>,
}

impl Configs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<C: Component>(mut self, config: C::Config) -> Self {
        self.map.insert(TypeId::of::<C>(), (Box::new(config), type_name::<C>()));
        self
    }
}

/// Build pass 1: walk the graph, detect identity collisions, resolve each
/// component's config (supplied or default).
#[doc(hidden)]
pub struct ConfigPass {
    pub(crate) supplied: HashMap<TypeId, (Box<dyn Any>, &'static str)>,
    pub(crate) resolved: HashMap<TypeId, Box<dyn Any>>,
    /// Compile-time id -> TypeId, to catch hash or type-name collisions.
    pub(crate) ids: HashMap<u64, (TypeId, &'static str)>,
}

impl ConfigPass {
    pub(crate) fn visit<C: Component>(&mut self) {
        let id = <C as Described>::ID;
        match self.ids.get(&id) {
            Some((tid, _)) if *tid == TypeId::of::<C>() => return, // seen
            Some((_, other)) => panic!(
                "component identity collision between {other} and {}; rename one",
                type_name::<C>()
            ),
            None => {
                self.ids.insert(id, (TypeId::of::<C>(), type_name::<C>()));
            }
        }
        <C::Deps as DepList>::collect_configs(self);

        let config = match self.supplied.remove(&TypeId::of::<C>()) {
            Some((cfg, _)) => cfg,
            None => Box::new(C::Config::default()) as Box<dyn Any>,
        };
        self.resolved.insert(TypeId::of::<C>(), config);
    }
}

/// Build pass 2: hands each constructor its config and records the order.
#[doc(hidden)]
pub struct ConstructPass {
    pub(crate) configs: HashMap<TypeId, Box<dyn Any>>,
    pub(crate) order: Vec<&'static str>,
}

impl ConstructPass {
    pub(crate) fn take_config<C: Component>(&mut self) -> C::Config {
        *self
            .configs
            .remove(&TypeId::of::<C>())
            .expect("config pass resolved every component")
            .downcast::<C::Config>()
            .expect("keyed by component type")
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
