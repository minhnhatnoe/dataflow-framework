//! Component identity: the `TypeId` of the component type, compared exactly.
//!
//! A node *is* its type, so its identity is `TypeId::of::<N>()`. That is
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

use crate::node::Node;
use crate::plan::Planned;

/// Fills the unused part of the plan's `TypeId` arrays (a `[TypeId; N]` needs
/// some value; `()` is not a component, so it never matches one).
pub(crate) const BLANK: TypeId = TypeId::of::<()>();

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
impl<N: Node> Hashed for N {
    const HASH: u64 = name_hash(type_name::<N>());
}

/// The index of node `N` in the plan of anchor `A`, as a constant.
#[doc(hidden)]
pub struct At<A, N>(PhantomData<(A, N)>);
impl<A: Planned, N: Node> At<A, N> {
    pub const INDEX: usize = A::PLAN.index_of(<N as Hashed>::HASH, TypeId::of::<N>());
}
