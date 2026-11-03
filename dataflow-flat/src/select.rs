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
//!
//! A handle component has no stored output to select from: its subscribers read a *view*.
//! A selector on it implements [`SelectView<In>`] instead, where `In` is the view (with its
//! lifetime) and the selected part is a view too (`#[derive(Demux)]` on an enum with a lifetime
//! parameter writes them; `option::Some` and the other library selectors have both impls). The
//! derive wraps the chain by the kind of the source (`ChainWrap`): for a handle source it becomes
//! a [`ViewC`], a [`SelectChain`] whose operations take the parent's state (see `gather.rs`).
//! Nothing selected from a view is stored.

use std::any::TypeId;
use std::marker::PhantomData;

use crate::node::{Edge, Vertex};
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
    /// Is this selector *requested* of a listener? True for
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
    type Init<'a, N: Vertex<Stored = In>>;
    type Read<'a, N: Vertex>;
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
    type Init<'a, N: Vertex<Stored = In>> = crate::node::InitOf<'a, N>;
    type Read<'a, N: Vertex> = crate::node::ReadOf<'a, N>;
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
    type Init<'a, N: Vertex<Stored = In>> = SelInit<'a, T::Out>;
    type Read<'a, N: Vertex> = &'a T::Out;
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
/// dereferences to the selected part.
///
/// The value is the slot's initial value if the input is *stored* (it can be read
/// in a step where its selectors do not match); otherwise, if the selector chain
/// has an [`Initial`] value (`Some`/`Ok`/`Err` and `#[derive(Demux)]` selectors with a
/// `Default` payload, hand-written selectors with an `Initial` impl),
/// `#[derive(ComponentSubscription)]` fills it in at the component, where the
/// payload type is known; otherwise there is none, and dereferencing panics when
/// the pipeline is built, naming the component, the input and the payload
/// ([`SelInit::get`] gives `None` instead).
pub struct SelInit<'a, T>(SelInner<'a, T>);
enum SelInner<'a, T> {
    Slot(&'a T),
    Owned(T),
    Missing { component: &'static str, input: &'static str, payload: &'static str },
    /// Not yet filled by the derive (library internal).
    Empty,
}
impl<'a, T> SelInit<'a, T> {
    pub(crate) fn slot(x: &'a T) -> Self {
        SelInit(SelInner::Slot(x))
    }
    pub(crate) fn empty() -> Self {
        SelInit(SelInner::Empty)
    }
    /// The initial value, if this input has one.
    pub fn get(&self) -> Option<&T> {
        match &self.0 {
            SelInner::Slot(x) => Some(x),
            SelInner::Owned(x) => Some(x),
            _ => None,
        }
    }
}
impl<T: Clone> Clone for SelInit<'_, T> {
    fn clone(&self) -> Self {
        SelInit(match &self.0 {
            SelInner::Slot(x) => SelInner::Slot(x),
            SelInner::Owned(x) => SelInner::Owned(x.clone()),
            SelInner::Missing { component, input, payload } => SelInner::Missing { component, input, payload },
            SelInner::Empty => SelInner::Empty,
        })
    }
}
impl<T: core::fmt::Debug> core::fmt::Debug for SelInit<'_, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.get().fmt(f)
    }
}
impl<T> core::ops::Deref for SelInit<'_, T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        match &self.0 {
            SelInner::Slot(x) => x,
            SelInner::Owned(x) => x,
            SelInner::Missing { component, input, payload } => no_initial_value(component, input, payload),
            SelInner::Empty => no_initial_value("?", "?", core::any::type_name::<T>()),
        }
    }
}
#[cold]
#[inline(never)]
fn no_initial_value(component: &str, input: &str, payload: &str) -> ! {
    panic!(
        "`{component}::new` read the input `{input}` (selected payload `{payload}`), which has no initial value: \
         implement `Default` for `{payload}` or `Initial` for its selector"
    )
}

/// What the derive calls to give a selected input its initial value in `new`
/// (autoref specialization at the component, where the types are concrete:
/// the chain's [`ChainInitial`] if it exists, else nothing).
#[doc(hidden)]
pub mod probe {
    use super::*;
    use core::marker::PhantomData;
    pub struct Probe<C, In>(PhantomData<(C, In)>);
    impl<C, In> Probe<C, In> {
        #[inline(always)]
        pub fn new() -> Self {
            Probe(PhantomData)
        }
    }
    pub trait ViaInitial<C: SelectChain<In>, In: 'static> {
        fn fill<'a, N: Vertex<Stored = In>>(
            &self,
            raw: C::Init<'a, N>,
            component: &'static str,
            input: &'static str,
        ) -> C::Init<'a, N>;
    }
    impl<C: SelectedChain<In> + ChainInitial<In> + ByValueChain, In: 'static> ViaInitial<C, In> for Probe<C, In> {
        #[inline]
        fn fill<'a, N: Vertex<Stored = In>>(&self, raw: C::Init<'a, N>, _: &'static str, _: &'static str) -> C::Init<'a, N> {
            let s = C::view::<N>(&raw);
            C::wrap::<N>(match s.0 {
                SelInner::Empty => SelInit(SelInner::Owned(C::initial())),
                SelInner::Slot(x) => SelInit(SelInner::Slot(x)),
                _ => SelInit::empty(),
            })
        }
    }
    pub trait ViaNothing<C: SelectChain<In>, In: 'static> {
        fn fill<'a, N: Vertex<Stored = In>>(
            &self,
            raw: C::Init<'a, N>,
            component: &'static str,
            input: &'static str,
        ) -> C::Init<'a, N>;
    }
    impl<C: SelectedChain<In>, In: 'static> ViaNothing<C, In> for &Probe<C, In> {
        #[inline]
        fn fill<'a, N: Vertex<Stored = In>>(&self, raw: C::Init<'a, N>, component: &'static str, input: &'static str) -> C::Init<'a, N> {
            let s = C::view::<N>(&raw);
            C::wrap::<N>(match s.0 {
                SelInner::Empty => SelInit(SelInner::Missing {
                    component,
                    input,
                    payload: core::any::type_name::<C::Out>(),
                }),
                SelInner::Slot(x) => SelInit(SelInner::Slot(x)),
                _ => SelInit::empty(),
            })
        }
    }
    /// A selected entry of a handle component: `new` gets `Option` of the selected view, nothing to
    /// fill in (there is no initial value and no stored slot). The raw value is kept as it is.
    impl<E: Vertex, C: ViewChain<E>> ViaInitial<ViewC<E, C>, E::Stored> for Probe<ViewC<E, C>, E::Stored> {
        #[inline(always)]
        fn fill<'a, N: Vertex<Stored = E::Stored>>(
            &self,
            raw: <ViewC<E, C> as SelectChain<E::Stored>>::Init<'a, N>,
            _: &'static str,
            _: &'static str,
        ) -> <ViewC<E, C> as SelectChain<E::Stored>>::Init<'a, N> {
            raw
        }
    }
    /// A by-value selector chain (a sealed marker without parameters, so that the impl above and
    /// the one for a handle source's chain are provably disjoint).
    pub trait ByValueChain {}
    impl<S, T> ByValueChain for (S, T) {}

    /// A non-empty chain, whose `Init` is a [`SelInit`] (the compiler cannot
    /// always see that through the associated type in generic code).
    pub trait SelectedChain<In: 'static>: SelectChain<In> {
        fn view<'a, N: Vertex<Stored = In>>(raw: &Self::Init<'a, N>) -> SelInit<'a, Self::Out>;
        fn wrap<'a, N: Vertex<Stored = In>>(s: SelInit<'a, Self::Out>) -> Self::Init<'a, N>;
    }
    impl<In: 'static, S: Select<In>, T: SelectChain<S::Out>> SelectedChain<In> for (S, T) {
        #[inline(always)]
        fn view<'a, N: Vertex<Stored = In>>(raw: &<Self as SelectChain<In>>::Init<'a, N>) -> SelInit<'a, T::Out> {
            match raw.0 {
                SelInner::Slot(x) => SelInit(SelInner::Slot(x)),
                _ => SelInit::empty(),
            }
        }
        #[inline(always)]
        fn wrap<'a, N: Vertex<Stored = In>>(s: SelInit<'a, T::Out>) -> <Self as SelectChain<In>>::Init<'a, N> {
            s
        }
    }
}

/// The value a selector's part has before the selector has ever matched.
/// `#[derive(Demux)]` selectors implement it with `Default::default()` of the
/// payload (a variant whose payload is not `Default` has no usable impl, which
/// only matters if it is read in `new` or in steps where it does not match). `Some`
/// and `Ok`, `Err` do likewise; a hand-written selector implements it however
/// it likes. A chain takes the last selector's. A selected input's slot requires
/// this at compile time; for an input without a slot it is optional: `new` gets it
/// through [`SelInit`] if the impl exists, else a panic when `new` reads it.
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

// ---------------------------------------------------------------------------
// Selectors on views (the source is a handle component)
// ---------------------------------------------------------------------------

/// A pattern test on a *view*: a value of type `In` (a handle component's
/// [`View<'a>`](crate::Component::View), or the part a selector before this one in the
/// chain matched), taken by value. `In` includes its lifetime, so the selected part can borrow
/// from the same place: `impl<'a> SelectView<FrameView<'a>> for frame_view::Book { type Out = &'a Book; .. }`.
/// `#[derive(Demux)]` on an enum with a lifetime parameter (`enum FrameView<'a> { Book(&'a Book), .. }`)
/// writes these impls; `option::Some` and the other library selectors have one for the views that
/// are `Option`, `Result`.
///
/// A view selector is the view-side twin of [`Select`]: the same decision (`KEY`, `key_of`), but the
/// input is the view, not a reference to a stored output, and the selected part is a view too (a
/// reference, a copy, a struct that borrows), never stored: a subscriber reads it as it is
/// computed, each time it runs.
#[diagnostic::on_unimplemented(
    message = "the selector `{Self}` cannot be applied to the view `{In}`",
    label = "`{Self}` must implement `SelectView<{In}>` to select from this view",
    note = "a selector on a handle component selects from its `View<'a>`: put `#[derive(Demux)]` on the view's enum (`enum V<'a> {{ A(&'a X), .. }}`), or implement `SelectView<V<'a>>` by hand; a by-value selector (`Select<T>`) cannot be applied to a view"
)]
pub trait SelectView<In>: 'static {
    /// The matched part (may borrow: it has the lifetime of `In`).
    type Out;
    /// As [`Select::KEY`].
    const KEY: u32;
    /// As [`Select::REQUEST`]: is this selector *requested* of a listener? `#[derive(Demux)]` sets it.
    const REQUEST: bool = false;
    /// `Some(part)` if the pattern matches.
    fn select(x: In) -> Option<Self::Out>;
    /// As [`Select::key_of`].
    fn key_of(x: &In) -> u32;
}

/// A chain of view selectors `(S1, (S2, ... ()))` applied to a view of type `In`.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "the selector chain `{Self}` cannot be applied to the view `{In}`",
    label = "each selector must implement `SelectView<_>` for the view (or the part) before it",
    note = "a selector on a handle component selects from its `View<'a>`: put `#[derive(Demux)]` on the view's enum (`enum V<'a> {{ A(&'a X), .. }}`), or implement `SelectView<V<'a>>` by hand; `Some`/`Ok`/.. work when the view is an `Option`/`Result`; a by-value selector (`Select<T>` for a stored output) cannot be applied to a view"
)]
pub trait ViewChainOn<In>: 'static {
    /// The part at the end of the chain.
    type Out;
    /// Does the chain match?
    fn matches(x: In) -> bool;
    /// The part at the end; panics if the chain does not match (the plan has checked that it does
    /// whenever this is called).
    fn get(x: In) -> Self::Out;
    /// The part at the end if the chain matches.
    fn try_get(x: In) -> Option<Self::Out>;
    /// The key of the alternative at depth `J` (as [`ChainKey`]).
    fn key(x: In, j: usize) -> u32;
}
impl<In> ViewChainOn<In> for () {
    type Out = In;
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn matches(_: In) -> bool {
        true
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn get(x: In) -> In {
        x
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn try_get(x: In) -> Option<In> {
        Some(x)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key(_: In, _: usize) -> u32 {
        GUARD
    }
}
impl<In, S: SelectView<In>, T: ViewChainOn<S::Out>> ViewChainOn<In> for (S, T) {
    type Out = T::Out;
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn matches(x: In) -> bool {
        match S::select(x) {
            Some(y) => T::matches(y),
            None => false,
        }
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn get(x: In) -> T::Out {
        match S::select(x) {
            Some(y) => T::get(y),
            None => selector_mismatch(),
        }
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn try_get(x: In) -> Option<T::Out> {
        match S::select(x) {
            Some(y) => T::try_get(y),
            None => None,
        }
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key(x: In, j: usize) -> u32 {
        if j == 0 {
            S::key_of(&x)
        } else {
            match S::select(x) {
                Some(y) => T::key(y, j - 1),
                None => GUARD,
            }
        }
    }
}

/// A chain of view selectors applied to the view of the node `E` (`E::Read<'a>`, for every `'a`).
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "the selector chain `{Self}` cannot be applied to the view of `{E}`",
    label = "`{E}` is a handle component: its selectors must implement `SelectView<{E}'s View>`",
    note = "use selectors made for the view: `#[derive(Demux)]` on the view's enum (with a lifetime parameter), `Some`/`Ok`/.. when the view is an `Option`/`Result`, or a hand-written `SelectView`; a `Select<T>` for a stored output (a by-value selector) does not apply to a view"
)]
pub trait ViewChain<E: Vertex>: 'static {
    /// The part at the end of the chain, as a subscriber's field.
    type Out<'a>;
    fn matches<'a>(x: <E as Edge>::Read<'a>) -> bool;
    fn get<'a>(x: <E as Edge>::Read<'a>) -> Self::Out<'a>;
    fn try_get<'a>(x: <E as Edge>::Read<'a>) -> Option<Self::Out<'a>>;
    fn key<'a, const J: usize>(x: <E as Edge>::Read<'a>) -> u32;
}
impl<E: Vertex, C> ViewChain<E> for C
where
    for<'a> C: ViewChainOn<<E as Edge>::Read<'a>>,
{
    type Out<'a> = <C as ViewChainOn<<E as Edge>::Read<'a>>>::Out;
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn matches<'a>(x: <E as Edge>::Read<'a>) -> bool {
        <C as ViewChainOn<<E as Edge>::Read<'a>>>::matches(x)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn get<'a>(x: <E as Edge>::Read<'a>) -> Self::Out<'a> {
        <C as ViewChainOn<<E as Edge>::Read<'a>>>::get(x)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn try_get<'a>(x: <E as Edge>::Read<'a>) -> Option<Self::Out<'a>> {
        <C as ViewChainOn<<E as Edge>::Read<'a>>>::try_get(x)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key<'a, const J: usize>(x: <E as Edge>::Read<'a>) -> u32 {
        <C as ViewChainOn<<E as Edge>::Read<'a>>>::key(x, J)
    }
}

/// The selector chain `C` as the plan and the drivers see it when the source `E` is a handle component
/// component (`NodeKind::Chain`). It is a `SelectChain<E::Stored>` so that everything that
/// carries a chain (the subscription tree, the derive's types) is unchanged, but its `Read` is the
/// selected view and its `Init` is `Option` of it; the operations that need the source's state
/// (matching, the switch key, reading) are the `ChainTest`, `ChainKeyAt`, `ChainReadCx` and
/// `ChainSeed` impls in `gather.rs`. A selected value from a handle is never stored (the plan
/// rejects an entry that would need it).
#[doc(hidden)]
pub struct ViewC<E, C>(PhantomData<fn() -> (E, C)>);
impl<E: Vertex, C: ViewChain<E>> SelectChain<E::Stored> for ViewC<E, C> {
    type Out = ();
    type Init<'a, N: Vertex<Stored = E::Stored>> = Option<<C as ViewChain<E>>::Out<'a>>;
    type Read<'a, N: Vertex> = <C as ViewChain<E>>::Out<'a>;
    const PLAIN: bool = false;
    fn matches(_: &E::Stored) -> bool {
        view_without_state()
    }
    fn get(_: &E::Stored) -> &() {
        view_without_state()
    }
}
#[cold]
#[inline(never)]
fn view_without_state() -> ! {
    panic!("a selector on a handle component was applied without its state (this is a bug in dataflow-flat)")
}

/// The plan's walk of a chain of view selectors on a view of type `In`: as [`ChainWalk`].
#[doc(hidden)]
pub const trait ViewWalk<In> {
    fn walk(plan: &mut Plan, parent: u32, dn: u32) -> u32;
}
const impl<In> ViewWalk<In> for () {
    fn walk(_: &mut Plan, _: u32, dn: u32) -> u32 {
        dn
    }
}
const impl<In, S: SelectView<In>, T: [const] ViewWalk<S::Out>> ViewWalk<In> for (S, T) {
    fn walk(plan: &mut Plan, parent: u32, dn: u32) -> u32 {
        let d = plan.decision(parent, dn, TypeId::of::<S>(), <S as SelectView<In>>::KEY);
        T::walk(plan, parent, d)
    }
}
const impl<E: Vertex, C: [const] ViewWalk<<E as Edge>::Read<'static>>> ChainWalk<E::Stored> for ViewC<E, C> {
    fn walk(plan: &mut Plan, parent: u32, dn: u32) -> u32 {
        C::walk(plan, parent, dn)
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
    impl<T> super::SelectView<Option<T>> for Some {
        type Out = T;
        const KEY: u32 = 0;
        #[cfg_attr(not(debug_assertions), inline(always))]
        #[cfg_attr(debug_assertions, inline)]
        fn select(x: Option<T>) -> Option<T> {
            x
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
    impl<T> super::SelectView<Option<T>> for None {
        type Out = ();
        const KEY: u32 = 1;
        #[cfg_attr(not(debug_assertions), inline(always))]
        #[cfg_attr(debug_assertions, inline)]
        fn select(x: Option<T>) -> Option<()> {
            if x.is_none() {
                Option::Some(())
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
    impl<T, E> super::SelectView<Result<T, E>> for Ok {
        type Out = T;
        const KEY: u32 = 0;
        #[cfg_attr(not(debug_assertions), inline(always))]
        #[cfg_attr(debug_assertions, inline)]
        fn select(x: Result<T, E>) -> Option<T> {
            x.ok()
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
    impl<T, E> super::SelectView<Result<T, E>> for Err {
        type Out = E;
        const KEY: u32 = 1;
        #[cfg_attr(not(debug_assertions), inline(always))]
        #[cfg_attr(debug_assertions, inline)]
        fn select(x: Result<T, E>) -> Option<E> {
            x.err()
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
