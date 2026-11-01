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
    /// Is this a keyed selector ([`On`])? The plan collects the keys of keyed
    /// decisions as the requests of a [`Listener`](crate::Listener), and two
    /// keyed selectors with the same `KEY` at one place are an error.
    const KEYED: bool = false;
    /// The name used in messages about a keyed selector (the symbol's name).
    const LABEL: &'static str = "";
    /// `Some(part)` if the pattern matches.
    fn select(x: &In) -> Option<&Self::Out>;
}

/// A chain of selectors as a nested tuple `(S1, (S2, ... ()))`, applied to a
/// value of type `In`. `()` is the empty chain. The derive builds it.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "the selector chain `{Self}` cannot be applied to a value of type `{In}`",
    label = "each selector must implement `Select<_>` for the output of the one before it (a `Some`/`None` needs an `Option`, an `On<_>` needs an output that implements `Keyed`)"
)]
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
        let d = plan.decision(
            parent,
            dn,
            TypeId::of::<S>(),
            <S as Select<In>>::KEY,
            <S as Select<In>>::KEYED,
            if <S as Select<In>>::LABEL.is_empty() { core::any::type_name::<S>() } else { <S as Select<In>>::LABEL },
        );
        T::walk(plan, parent, d)
    }
}

/// A symbol: a type-level name with an integer key, such as a market
/// (`struct Btc; impl Symbol for Btc { const KEY: u32 = 0; .. }`). Keys are
/// dense per source in the intended use (0..n).
pub trait Symbol: 'static {
    /// Identifies the symbol among those selected from one parent.
    const KEY: u32;
    /// Shown in messages.
    const NAME: &'static str;
}

/// A value that carries a key and an item, such as a message of a multiplexed
/// feed: the thing [`On`] selects from.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not `Keyed`, so `On<_>` can't select from it",
    label = "`On<S>` needs the output of the subscribed node to implement `dataflow_flat::Keyed`"
)]
pub trait Keyed {
    /// What `On` hands to the subscriber.
    type Item: 'static;
    /// The key of this value, if it has one.
    fn key(&self) -> Option<u32>;
    /// The payload.
    fn item(&self) -> &Self::Item;
}

/// The keyed selector: matches iff the parent's value has key `S::KEY`; the
/// part is the item. Two `On`s with different symbols at one place are
/// mutually exclusive, and the keys of the `On`s that read a
/// [`Listener`](crate::Listener) are its requests.
///
/// ```ignore
/// #[subscribe(u: Ws => Some, On<Btc>)]
/// ```
pub struct On<S>(core::marker::PhantomData<fn() -> S>);

impl<In: Keyed, S: Symbol> Select<In> for On<S> {
    type Out = In::Item;
    const KEY: u32 = S::KEY;
    const KEYED: bool = true;
    const LABEL: &'static str = S::NAME;
    #[inline(always)]
    fn select(x: &In) -> Option<&In::Item> {
        if x.key() == Option::Some(S::KEY) {
            Option::Some(x.item())
        } else {
            Option::None
        }
    }
}

/// A component that wants to know which keys its subscribers read: declared
/// with `#[listener]` on the `ComponentSubscription` derive. At
/// [`Pipeline::build`](crate::Pipeline::build), after every component is
/// constructed and before it returns, `subscribe` is called once with the
/// sorted, deduplicated keys of every [`On`] that reads this component's
/// output (directly or after other selectors).
pub trait Listener {
    fn subscribe(&mut self, keys: &[u32]);
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
