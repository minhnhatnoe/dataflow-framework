//! Configs and the two build passes. These are the only places with runtime
//! lookups (`TypeId` maps); the run itself has none.

use std::any::{type_name, Any, TypeId};
use std::collections::HashMap;
use std::fmt;

use crate::component::Component;
use crate::plan::Described;

/// A config supplied for a component, with enough to check it is the right one.
pub(crate) struct ConfigEntry {
    config: Box<dyn Any>,
    type_id: TypeId,
    name: &'static str,
}

impl ConfigEntry {
    pub(crate) fn name(&self) -> &'static str {
        self.name
    }
}

/// Owned configs, keyed by component type (not config type), so two
/// components or two instantiations of one generic component never share a
/// config by accident. Internally the key is the component's compile-time id
/// (`Described::ID`), with the `TypeId` kept to catch two components whose ids
/// collide.
#[derive(Default)]
pub struct Configs {
    pub(crate) map: HashMap<u64, ConfigEntry>,
}

impl Configs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with<C: Component>(mut self, config: C::Config) -> Self {
        let id = <C as Described>::ID;
        let entry = ConfigEntry { config: Box::new(config), type_id: TypeId::of::<C>(), name: type_name::<C>() };
        if let Some(old) = self.map.insert(id, entry) {
            if old.type_id != TypeId::of::<C>() {
                collision(old.name, type_name::<C>());
            }
        }
        self
    }
}

fn collision(a: &str, b: &str) -> ! {
    panic!("component identity collision between {a} and {b}; rename one")
}

/// Construction state: hands each constructor its config (supplied, or the
/// default) and records the construction order. Also checks that no two
/// components of the graph share a compile-time id, which is the only thing the
/// ids cannot guarantee by themselves (they are hashes of type names).
#[doc(hidden)]
pub struct ConstructPass {
    pub(crate) configs: HashMap<u64, ConfigEntry>,
    /// Compile-time id -> TypeId of the component constructed with it.
    pub(crate) ids: HashMap<u64, (TypeId, &'static str)>,
    pub(crate) order: Vec<&'static str>,
}

impl ConstructPass {
    pub(crate) fn take_config<C: Component>(&mut self) -> C::Config {
        let id = <C as Described>::ID;
        let type_id = TypeId::of::<C>();
        if let Some((other_id, other)) = self.ids.insert(id, (type_id, type_name::<C>())) {
            if other_id != type_id {
                collision(other, type_name::<C>());
            }
        }
        match self.configs.remove(&id) {
            None => C::Config::default(),
            Some(entry) if entry.type_id == type_id => {
                *entry.config.downcast::<C::Config>().expect("keyed by component type")
            }
            Some(entry) => collision(entry.name, type_name::<C>()),
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
