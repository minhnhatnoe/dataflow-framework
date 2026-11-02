//! Selectors: demuxing on the subscription edge.
//!
//! A subscription may carry a chain of *selectors* applied to the parent's
//! output, left to right:
//!
//! ```ignore
//! #[subscribe(Some(payload::Depth(upd)): BinanceWs)]
//! ```
//!
//! A selector is a zero-sized type implementing [`Select<In>`]: it tests a
//! pattern on a value of type `In` and, when it matches, gives a reference to
//! the matched part. Selectors are not nodes: they have no state and appear only
//! in subscriptions. (A subscriber that reads a selected input in steps where
//! it does not match gets the last matched value, kept by the plan in a slot,
//! or the selector's [`Initial`] value.) Library selectors:
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
    /// Is this selector *requested* of a [`Listener`](crate::Listener)? True for
    /// the selectors of `#[derive(Demux)]`: reading one downstream of a
    /// listener calls `Request<Self>::add` on its requests, and that impl must
    /// exist. `Some`, `Ok`, guards and hand-written selectors are not requests.
    const REQUEST: bool = false;
    /// `Some(part)` if the pattern matches.
    fn select(x: &In) -> Option<&Self::Out>;
    /// The key of the alternative `x` is: for a selector with a `KEY` other
    /// than [`GUARD`], the `KEY` of the selector of the same family that
    /// matches `x` (the variant index of an enum), or [`GUARD`] if none does.
    /// The push evaluates this once for all the keyed selectors applied at
    /// the same place and branches on the result, instead of testing each
    /// selector in turn. A guard returns [`GUARD`]. Every keyed selector
    /// applied at one place must belong to one family, i.e. give the same
    /// `key_of`.
    fn key_of(x: &In) -> u32;
}

/// A chain of selectors as a nested tuple `(S1, (S2, ... ()))`, applied to a
/// value of type `In`. `()` is the empty chain. The derive builds it.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "the selector chain `{Self}` cannot be applied to a value of type `{In}`",
    label = "each selector must implement `Select<_>` for the output of the one before it (a `Some`/`None` needs an `Option`)"
)]
pub trait SelectChain<In: 'static>: 'static {
    /// The value at the end of the chain.
    type Out: 'static;
    /// What `new` gets for an edge with this chain from node `N`: the stored
    /// output for a plain edge, a [`SelInit`] (the initial value of the slot)
    /// for a selected one.
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
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn matches(_: &In) -> bool {
        true
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn get(x: &In) -> &In {
        x
    }
}
impl<In: 'static, S: Select<In>, T: SelectChain<S::Out>> SelectChain<In> for (S, T) {
    type Out = T::Out;
    type Init<'a, N: Node<Output = In>> = SelInit<'a, T::Out>;
    const PLAIN: bool = false;
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn matches(x: &In) -> bool {
        match S::select(x) {
            Some(y) => T::matches(y),
            None => false,
        }
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn get(x: &In) -> &T::Out {
        match S::select(x) {
            Some(y) => T::get(y),
            None => selector_mismatch(),
        }
    }
}

/// What `new` is given for a selected input: the value the input has before its
/// selectors have matched (the selector's initial value, see [`Initial`]). It
/// dereferences to the selected part. The value exists for an input that is
/// *stored*, i.e. one that can be read in a step where its selectors do not
/// match; an input that is always covered has no slot and no initial value, and
/// dereferencing it panics when the pipeline is built ([`SelInit::get`] gives
/// `None` instead).
pub struct SelInit<'a, T>(pub(crate) Option<&'a T>);
impl<'a, T> SelInit<'a, T> {
    /// The initial value, if this input has one.
    pub fn get(&self) -> Option<&'a T> {
        self.0
    }
}
impl<T> Clone for SelInit<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for SelInit<'_, T> {}
impl<T: core::fmt::Debug> core::fmt::Debug for SelInit<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}
impl<T> core::ops::Deref for SelInit<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        match self.0 {
            Some(x) => x,
            None => no_initial_value(),
        }
    }
}
#[cold]
#[inline(never)]
fn no_initial_value() -> ! {
    panic!(
        "`new` read a selected input that has no initial value: only an input that can be read in a step where \
         its selectors do not match is stored and starts from the selector's initial value; this one is always \
         matched when its subscriber runs (`SelInit::get` tests for it)"
    )
}

/// The value a selector's part has before the selector has ever matched.
/// `#[derive(Demux)]` selectors implement it with `Default::default()` of the
/// payload (a variant whose payload is not `Default` has no usable impl, which
/// only matters if it is read in steps where it does not match). `Some` and `Ok`,
/// `Err` do likewise; a hand-written selector implements it however
/// it likes.
#[diagnostic::on_unimplemented(
    message = "the selector `{Self}` has no initial value for `{In}`",
    label = "a selected input that is read in steps where its selectors do not match starts from the selector's initial value",
    note = "the payload must implement `Default` (for `#[derive(Demux)]`, `Some`, `Ok` and `Err`), or write `impl Initial<{In}> for {Self}`"
)]
pub trait Initial<In: 'static>: Select<In> {
    fn initial() -> Self::Out;
}

/// The initial value at the end of a selector chain: the last selector's.
#[doc(hidden)]
pub trait ChainInitial<In: 'static>: SelectChain<In> {
    fn initial() -> Self::Out;
}
impl<In: 'static, S: Initial<In>> ChainInitial<In> for (S, ()) {
    fn initial() -> S::Out {
        S::initial()
    }
}
impl<In: 'static, S: Select<In>, S2: Select<S::Out>, T2: SelectChain<S2::Out>> ChainInitial<In> for (S, (S2, T2))
where
    (S2, T2): ChainInitial<S::Out>,
{
    fn initial() -> <(S2, T2) as SelectChain<S::Out>>::Out {
        <(S2, T2) as ChainInitial<S::Out>>::initial()
    }
}

/// A stored selected value is cloned out of the source's output.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "`{Self}` is read through selectors in steps where they do not match, so it is stored, and must implement `Clone`",
    label = "the last matching value is cloned into the pipeline's storage"
)]
pub trait Keep: Clone + 'static {}
#[diagnostic::do_not_recommend]
impl<T: Clone + 'static> Keep for T {}

/// Identity, for a conditional impl on a concrete type (a `where` that does
/// not hold on a concrete type is an error; on a generic parameter it is not).
#[doc(hidden)]
pub trait Identity {
    type Is;
    fn from_is(x: Self::Is) -> Self;
}
impl<T> Identity for T {
    type Is = T;
    #[inline(always)]
    fn from_is(x: T) -> T {
        x
    }
}

#[cold]
#[inline(never)]
fn selector_mismatch() -> ! {
    panic!("a selected subscription was read in a step where its selectors do not match (this is a bug in dataflow-flat)")
}

/// The key of the alternative a value is at depth `J` of a chain (the chain
/// applied up to depth `J`, then [`Select::key_of`] of the selector at `J`);
/// [`GUARD`] if the chain does not match up to there. The push uses it to
/// dispatch on the keyed selectors applied at one place.
#[doc(hidden)]
pub trait ChainKey<In: 'static, const J: usize>: SelectChain<In> {
    fn key(x: &In) -> u32;
}
impl<In: 'static, const J: usize> ChainKey<In, J> for () {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key(_: &In) -> u32 {
        GUARD
    }
}
pub(crate) const fn dec(j: usize) -> usize {
    if j == 0 {
        0
    } else {
        j - 1
    }
}
impl<In: 'static, S: Select<In>, T: ChainKey<S::Out, { dec(J) }>, const J: usize> ChainKey<In, J> for (S, T) {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key(x: &In) -> u32 {
        if J == 0 {
            S::key_of(x)
        } else {
            match S::select(x) {
                Some(y) => T::key(y),
                None => GUARD,
            }
        }
    }
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
        );
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
        #[cfg_attr(not(debug_assertions), inline(always))]
        #[cfg_attr(debug_assertions, inline)]
        fn select(x: &Option<T>) -> Option<&T> {
            x.as_ref()
        }
        #[cfg_attr(not(debug_assertions), inline(always))]
        #[cfg_attr(debug_assertions, inline)]
        fn key_of(x: &Option<T>) -> u32 {
            if x.is_some() {
                0
            } else {
                1
            }
        }
    }
    impl<T: Default + 'static> super::Initial<Option<T>> for Some {
        fn initial() -> T {
            T::default()
        }
    }
    impl<T: 'static> super::Initial<Option<T>> for None {
        fn initial() {}
    }
    impl<T: 'static> Select<Option<T>> for None {
        type Out = ();
        const KEY: u32 = 1;
        #[cfg_attr(not(debug_assertions), inline(always))]
        #[cfg_attr(debug_assertions, inline)]
        fn select(x: &Option<T>) -> Option<&()> {
            if x.is_none() {
                Option::Some(&())
            } else {
                Option::None
            }
        }
        #[cfg_attr(not(debug_assertions), inline(always))]
        #[cfg_attr(debug_assertions, inline)]
        fn key_of(x: &Option<T>) -> u32 {
            if x.is_some() {
                0
            } else {
                1
            }
        }
    }
}

/// Built-in selectors for `Result`.
pub mod result {
    use super::Select;

    /// Matches `Ok(x)`; the part is `x`.
    pub struct Ok;
    /// Matches `Err(e)`; the part is `e`.
    pub struct Err;

    impl<T: 'static, E: 'static> Select<Result<T, E>> for Ok {
        type Out = T;
        const KEY: u32 = 0;
        #[cfg_attr(not(debug_assertions), inline(always))]
        #[cfg_attr(debug_assertions, inline)]
        fn select(x: &Result<T, E>) -> Option<&T> {
            x.as_ref().ok()
        }
        #[cfg_attr(not(debug_assertions), inline(always))]
        #[cfg_attr(debug_assertions, inline)]
        fn key_of(x: &Result<T, E>) -> u32 {
            if x.is_ok() {
                0
            } else {
                1
            }
        }
    }
    impl<T: Default + 'static, E: 'static> super::Initial<Result<T, E>> for Ok {
        fn initial() -> T {
            T::default()
        }
    }
    impl<T: 'static, E: Default + 'static> super::Initial<Result<T, E>> for Err {
        fn initial() -> E {
            E::default()
        }
    }
    impl<T: 'static, E: 'static> Select<Result<T, E>> for Err {
        type Out = E;
        const KEY: u32 = 1;
        #[cfg_attr(not(debug_assertions), inline(always))]
        #[cfg_attr(debug_assertions, inline)]
        fn select(x: &Result<T, E>) -> Option<&E> {
            x.as_ref().err()
        }
        #[cfg_attr(not(debug_assertions), inline(always))]
        #[cfg_attr(debug_assertions, inline)]
        fn key_of(x: &Result<T, E>) -> u32 {
            if x.is_ok() {
                0
            } else {
                1
            }
        }
    }
}
