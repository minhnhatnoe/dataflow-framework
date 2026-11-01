//! Selectors: demuxing on the subscription edge.
//!
//! A subscription may carry a chain of *selectors* applied to the parent's
//! output, left to right:
//!
//! ```ignore
//! #[subscribe(upd: BinanceWs => Some, payload::Depth)]
//! ```
//!
//! A selector is a zero-sized type implementing [`Select<In>`]: it tests a
//! pattern on a value of type `In` and, when it matches, gives a reference to
//! the matched part. Selectors are not nodes: they have no state, nothing is
//! stored, and they appear only in subscriptions. Library selectors:
//! `option::Some` and `option::None` (the derive maps a bare `Some` / `None`
//! in a chain to them); `#[derive(Demux)]` on an enum generates one selector
//! per variant in a module named after the enum in snake_case; a guard is a
//! hand-written `Select` impl with `KEY = GUARD`.
//!
//! The plan sees a selector as a *decision* `(parent, position in the chain,
//! key)`: two decisions at the same place with different keys are mutually
//! exclusive (the variants of one enum), unless one is a guard.

use std::any::TypeId;

use crate::node::Node;
use crate::plan::Plan;

/// The `KEY` of a guard: it is exclusive with nothing.
pub const GUARD: u32 = u32::MAX;

/// A pattern test on a value of type `In`.
pub trait Select<In>: 'static {
    /// The matched part.
    type Out: 'static;
    /// Identifies the alternative among the selectors that can be applied at
    /// the same place (the variant index of an enum). Two selectors with
    /// different keys at the same place are mutually exclusive; [`GUARD`]
    /// means "a condition that may hold together with anything".
    const KEY: u32;
    /// `Some(part)` if the pattern matches.
    fn select(x: &In) -> Option<&Self::Out>;
}

/// A chain of selectors as a nested tuple `(S1, (S2, ... ()))`, applied to a
/// value of type `In`. `()` is the empty chain. The derive builds it.
#[doc(hidden)]
pub trait SelectChain<In: 'static>: 'static {
    /// The value at the end of the chain.
    type Out: 'static;
    /// What `new` gets for an edge with this chain from node `N`: the stored
    /// output for a plain edge, `()` for a selected one.
    type Init<'a, N: Node<Output = In>>;
    /// Is the chain empty?
    const PLAIN: bool;
    /// Does the chain match?
    fn matches(x: &In) -> bool;
    /// The value at the end; panics if the chain does not match (the plan has
    /// checked that it does whenever this is called).
    fn get(x: &In) -> &Self::Out;
}
impl<In: 'static> SelectChain<In> for () {
    type Out = In;
    type Init<'a, N: Node<Output = In>> = crate::node::InitOf<'a, N>;
    const PLAIN: bool = true;
    #[inline(always)]
    fn matches(_: &In) -> bool {
        true
    }
    #[inline(always)]
    fn get(x: &In) -> &In {
        x
    }
}
impl<In: 'static, S: Select<In>, T: SelectChain<S::Out>> SelectChain<In> for (S, T) {
    type Out = T::Out;
    type Init<'a, N: Node<Output = In>> = ();
    const PLAIN: bool = false;
    #[inline(always)]
    fn matches(x: &In) -> bool {
        match S::select(x) {
            Some(y) => T::matches(y),
            None => false,
        }
    }
    #[inline(always)]
    fn get(x: &In) -> &T::Out {
        match S::select(x) {
            Some(y) => T::get(y),
            None => selector_mismatch(),
        }
    }
}

#[cold]
#[inline(never)]
fn selector_mismatch() -> ! {
    panic!("a selected subscription was read in a step where its selectors do not match (this is a bug in dataflow-flat)")
}

/// The plan's walk of a chain: record each selector as a decision under the
/// previous one, starting at `dn` under node `parent`; returns the last.
#[doc(hidden)]
pub const trait ChainWalk<In> {
    fn walk(plan: &mut Plan, parent: u32, dn: u32) -> u32;
}
const impl<In> ChainWalk<In> for () {
    fn walk(_: &mut Plan, _: u32, dn: u32) -> u32 {
        dn
    }
}
const impl<In: 'static, S: Select<In>, T: [const] ChainWalk<S::Out>> ChainWalk<In> for (S, T) {
    fn walk(plan: &mut Plan, parent: u32, dn: u32) -> u32 {
        let d = plan.decision(parent, dn, TypeId::of::<S>(), <S as Select<In>>::KEY);
        T::walk(plan, parent, d)
    }
}

/// Built-in selectors for `Option`.
pub mod option {
    use super::Select;

    /// Matches `Some(x)`; the part is `x`.
    pub struct Some;
    /// Matches `None`; the part is `()`.
    pub struct None;

    impl<T: 'static> Select<Option<T>> for Some {
        type Out = T;
        const KEY: u32 = 0;
        #[inline(always)]
        fn select(x: &Option<T>) -> Option<&T> {
            x.as_ref()
        }
    }
    impl<T: 'static> Select<Option<T>> for None {
        type Out = ();
        const KEY: u32 = 1;
        #[inline(always)]
        fn select(x: &Option<T>) -> Option<&()> {
            if x.is_none() {
                Option::Some(&())
            } else {
                Option::None
            }
        }
    }
}
