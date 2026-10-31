//! Component identity: the `TypeId` of the component type, compared exactly.
//!
//! A component *is* its type, so its identity is `TypeId::of::<C>()`. That is
//! available in constant evaluation without a feature gate and is unique by
//! construction: two types that print the same `type_name` (two versions of
//! one crate) have different `TypeId`s, and so do two instantiations of a
//! generic component. Nothing is hashed into an identity, so two components
//! can never be confused.
//!
//! In constant evaluation a `TypeId` can be compared (`==`, with
//! `const_trait_impl` and `const_cmp`) but not ordered, hashed or turned into
//! an integer (that needs `transmute`, which `#![forbid(unsafe_code)]` rules
//! out). So the plan keeps its components' `TypeId`s in a plain array in
//! first-reached order, and finds one through the [`table`](crate::table),
//! which starts at a hash of the type *name* (a hint only; [`Hashed`]) and
//! confirms every match with `==`. Everything after that works on indices: the
//! first parent of a component is an index, the subtree of a component is a
//! range of indices, and the per-component constant [`At::INDEX`] is the result
//! of the one lookup that finds the component in the root's plan. The rest of
//! the crate never compares identities again.
//!
//! (Finding a component by a linear scan of the `TypeId`s instead needs no
//! hash and no `const_type_name`, but was measured at +1.2 s of debug compile
//! time for 397 components: see the README, "Identity".)

use std::any::{type_name, TypeId};
use std::marker::PhantomData;

use crate::component::Component;
use crate::plan::Planned;

/// Fills the unused part of the plan's `TypeId` arrays (a `[TypeId; N]` needs
/// some value; `()` is not a component, so it never matches one).
pub(crate) const BLANK: TypeId = TypeId::of::<()>();

/// Panics if a list of identities has a repeat.
pub(crate) const fn assert_distinct(ids: &[TypeId]) {
    let mut i = 0;
    while i < ids.len() {
        let mut j = i + 1;
        while j < ids.len() {
            if ids[i] == ids[j] {
                panic!("a component lists the same dependency twice");
            }
            j += 1;
        }
        i += 1;
    }
}

/// FNV-1a of a type name: where the plan's table starts looking for a
/// component (see `table`). It is only a hint: equal names or equal hashes of
/// different types are told apart by `TypeId`.
const fn name_hash(s: &str) -> u64 {
    let bytes = s.as_bytes();
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        i += 1;
    }
    hash
}

/// Where the plan's table starts looking for a component.
#[doc(hidden)]
pub trait Hashed {
    const HASH: u64;
}
impl<C: Component> Hashed for C {
    const HASH: u64 = name_hash(type_name::<C>());
}

/// The position of component `C` in the plan of root `R`, as a constant.
#[doc(hidden)]
pub struct At<R, C>(PhantomData<(R, C)>);
impl<R: Planned, C: Component> At<R, C> {
    pub const INDEX: usize = R::PLAN.index_of(<C as Hashed>::HASH, TypeId::of::<C>());
}
