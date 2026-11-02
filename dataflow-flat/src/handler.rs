//! Update handlers: `UpdateHandler<E>` for every entry of a component's `#[subscribe(..)]`.
//!
//! ```ignore
//! // No handlers at all: one blanket impl with `ACTIVE = false`.
//! #[derive(ComponentSubscription, UpdateHandler)]
//! #[subscribe(limits: Limits, feed: Feed)]
//! pub struct Quoter { .. }
//!
//! // Hand-written: every entry, named included, needs an impl.
//! #[derive(ComponentSubscription)]
//! #[subscribe(limits: Limits, Quoter<Btc>)]
//! pub struct Oms { .. }
//! impl UpdateHandler<Limits> for Oms {}                    // `ACTIVE`, does nothing
//! impl UpdateHandler<Quoter<Btc>> for Oms {
//!     fn on_update(&mut self, input: Self::Input<'_>, px: &i64) { .. }
//! }
//! ```
//!
//! The key `E` is the node type of a plain entry, named or not, and for a selected entry a marker
//! the derive generates, `<struct_snake>_edges::<Name>` (the binding's name in camel case, or the
//! innermost selector's for `_` and unit selectors), with `Arg<'a> = &'a <selected part>`.
//! `for S in Catalog { Quoter<S> }` expands to one unnamed entry per element of a catalog (see
//! the `catalog` module); the mapped entries are `Sub<Node, Hd<Node, ()>, pos, By<Owner, Node>>`.
//!
//! `on_update(&mut self, input, v)` is called when the edge fired, before `update`, in the
//! execution order of the parents; `input` is what `update` is about to get. `v` is
//! [`HandledEdge::Arg`]: `&Output` of a by-value parent, a handle parent's `View<'a>`, or the
//! selected part. Unnamed entries are not fields of `Input` (the derive wraps their chain in
//! [`Hd`]), named ones are, whether their impl is active or not.
//!
//! # In the plan and the push program
//!
//! The fourth parameter of every `Sub` is `By<Owner, E>`; the plan reads
//! `<Owner as UpdateHandler<E>>::ACTIVE` through [`Hook`] when it walks the entry. An edge with an
//! active impl is *handled*; a component with at least one handled edge is a handler. A handler's
//! run is one program node (`PG_HRUN`) with the list of the handled edges that may have fired
//! *under the decisions known in that place of the program* (`Plan::mk_hrun`): inside a `Switch`
//! leaf that is exactly the edges whose cones are inside the leaf, with no test at run time.
//! Elsewhere an edge keeps a test on its parent's flag and/or its selectors. A component with no
//! handled edge is planned exactly as an ordinary component. The unnamed (`Hd`) wrapper is a
//! separate matter: it only says that `update` does not read the edge (rule 5, no storage).
//!
//! A handler of a component with a handle parent runs its items and `update` through the split of
//! the states (`HVia` for `ByView`); the others read the stored outputs directly (`HVia` for
//! `ByVal`).

use std::marker::PhantomData;

use crate::component::ComponentSubscription;
use crate::drive::{CondAt, Ch, Drive, EntryOf, ExecK, FiredOf, FiredOps, HRunK, Pn, PnC, PushCtx, PushPh, Rg, RgCl, SlotWriteBy, SrcSlots};
use crate::flat::{EntryPos, Inner, Leaf, NthT, PickEdge, PosSelect};
use crate::gather::{ChainRead, ChainReadCx, ChainSeed, CtxOf, GatherCx, GatherIn, Layout, Ready, Source};
use crate::component::Component;
use crate::node::{Comp, Edge, Vertex};
use crate::split::SplitAt;
use crate::plan::{Planned, NO_FLAG};
use crate::select::{ChainKey, ChainWalk, SelectChain};
use crate::store::AtPos;

/// What an `UpdateHandler` impl is keyed by for one entry of `#[subscribe(..)]`: the node type for
/// a plain entry (named or not), a marker the derive generates (`<struct_snake>_edges::<Name>`)
/// for a selected one. `Arg` is what `on_update` gets: for a node, what its subscribers read
/// (`&Output` of a by-value component, the `View<'a>` of a handle component); for a selected
/// entry, the selected part, by reference.
pub trait HandledEdge: 'static {
    type Arg<'a>;
}
impl<N: Vertex> HandledEdge for N {
    type Arg<'a> = <N as Edge>::Read<'a>;
}

/// `on_update` of the entry `E`, and whether the plan calls it. Every entry of a component's
/// `#[subscribe(..)]`, named or not, needs an impl: `#[derive(UpdateHandler)]` gives the component
/// one for all of them with `ACTIVE = false`; by hand, `impl UpdateHandler<E> for C {}` keeps the
/// default `ACTIVE = true` and an empty `on_update`. A missing impl is a compile error naming
/// the entry.
#[diagnostic::on_unimplemented(
    message = "`{Self}` subscribes to `{E}` without an `UpdateHandler<{E}>` impl",
    label = "`{Self}` has `{E}` in `#[subscribe(..)]`, but no `UpdateHandler<{E}>` impl",
    note = "every entry needs one: put `#[derive(UpdateHandler)]` on `{Self}` (no handlers at all), or write `impl UpdateHandler<{E}> for {Self} {{}}` (add `fn on_update(&mut self, input: Self::Input<'_>, v: ..)` to be called when the entry fires)"
)]
pub trait UpdateHandler<E: HandledEdge>: ComponentSubscription {
    /// Does the plan call `on_update` for this entry? `#[derive(UpdateHandler)]` sets `false`.
    /// A component whose entries are all inactive is an ordinary component in the push program.
    const ACTIVE: bool = true;
    /// Called when the entry `E` fired in this push, before `update`, with the same `input`
    /// `update` is about to get.
    #[allow(unused_variables)]
    fn on_update(&mut self, input: Self::Input<'_>, v: E::Arg<'_>) {}
}

/// The handler of the entry `E` of the component `H`, as a type in the subscription tree (the
/// fourth parameter of `Sub`): the plan reads `ACTIVE` from it.
#[doc(hidden)]
pub struct By<H, E>(PhantomData<fn() -> (H, E)>);

/// What the plan reads of the fourth parameter of an entry: `()` (no handler: the anchor's
/// entries) or [`By`].
#[doc(hidden)]
pub trait Hook: 'static {
    const ACTIVE: bool;
}
impl Hook for () {
    const ACTIVE: bool = false;
}
impl<H: UpdateHandler<E>, E: HandledEdge> Hook for By<H, E> {
    const ACTIVE: bool = <H as UpdateHandler<E>>::ACTIVE;
}
/// The entry marker of a [`By`].
#[doc(hidden)]
pub trait HookEm: Hook {
    type Em: HandledEdge;
}
impl<H: UpdateHandler<E>, E: HandledEdge> HookEm for By<H, E> {
    type Em = E;
}

/// The selector chain of a handled edge: `C` with the marker `Em`. Reads as
/// `()` for gathering (the value is passed to `on_update` instead, only when
/// the edge fired).
#[doc(hidden)]
pub struct Hd<Em, C>(PhantomData<fn() -> (Em, C)>);

impl<In: 'static, Em: 'static, C: SelectChain<In>> SelectChain<In> for Hd<Em, C> {
    type Out = ();
    type Init<'a, N: Vertex<Stored = In>> = C::Init<'a, N>;
    type Read<'a, N: Vertex> = &'a ();
    const PLAIN: bool = C::PLAIN;
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn matches(x: &In) -> bool {
        C::matches(x)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn get(_: &In) -> &() {
        &()
    }
}

impl<In: 'static, Em: 'static, C: ChainKey<In, J>, const J: usize> ChainKey<In, J> for Hd<Em, C> {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key(x: &In) -> u32 {
        C::key(x)
    }
}

impl<N: Vertex, A: Layout, Em: 'static, C: ChainSeed<N, A, P, I>, const P: usize, const I: usize> ChainSeed<N, A, P, I> for Hd<Em, C> {
    #[inline]
    fn seed<'a>(states: &'a A::StatesOpt, outs: &'a A::OutsOpt) -> <Self as SelectChain<N::Stored>>::Init<'a, N> {
        C::seed(states, outs)
    }
}

/// A handled edge reads nothing in `update` (the value goes to `on_update`).
impl<E: Source<A>, A: Layout, Em: 'static, C: SelectChain<E::Stored>, const P: usize, const I: usize> ChainRead<E, A, P, I> for Hd<Em, C> {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read<'a>(_: &'a A::Root, _: &'a A::Outs) -> &'a () {
        &()
    }
}

impl<E: Source<A>, A: Layout, Em: 'static, C: SelectChain<E::Stored>, const P: usize, const I: usize> ChainReadCx<E, A, P, I> for Hd<Em, C> {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_cx<'c, 'x>(_: &'c A::Root, _: &'c A::Outs, _: &'c CtxOf<'x, A, P>) -> &'c ()
    where
        A::States: SplitAt<P>,
    {
        &()
    }
}

const impl<In, Em, C: [const] ChainWalk<In>> ChainWalk<In> for Hd<Em, C> {
    fn walk(plan: &mut crate::plan::Plan, parent: u32, dn: u32) -> u32 {
        plan.mark_handled();
        C::walk(plan, parent, dn)
    }
}

// ---------------------------------------------------------------------------
// Reading the value of an entry for `on_update`
// ---------------------------------------------------------------------------

/// The value `on_update` gets for entry `I` of the component at `P` (a component with by-value
/// parents), and whether the entry's selectors match the source's current value. A named
/// entry's chain is read as `update` reads it; an unnamed one (`Hd`) is read through its
/// inner chain.
#[doc(hidden)]
pub trait EdgeRd<E: Source<A>, A: Layout, const P: usize, const I: usize>: SelectChain<E::Stored> {
    type V<'a>;
    fn read_v<'a>(root: &'a A::Root, outs: &'a A::Outs) -> Self::V<'a>;
}
impl<E: Source<A>, A: Layout, const P: usize, const I: usize> EdgeRd<E, A, P, I> for ()
where
    (): ChainRead<E, A, P, I>,
{
    type V<'a> = <() as SelectChain<E::Stored>>::Read<'a, E>;
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_v<'a>(root: &'a A::Root, outs: &'a A::Outs) -> Self::V<'a> {
        <() as ChainRead<E, A, P, I>>::read(root, outs)
    }
}
impl<E: Source<A>, A: Layout, S, T, const P: usize, const I: usize> EdgeRd<E, A, P, I> for (S, T)
where
    (S, T): ChainRead<E, A, P, I>,
{
    type V<'a> = <(S, T) as SelectChain<E::Stored>>::Read<'a, E>;
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_v<'a>(root: &'a A::Root, outs: &'a A::Outs) -> Self::V<'a> {
        <(S, T) as ChainRead<E, A, P, I>>::read(root, outs)
    }
}
impl<E: Source<A>, A: Layout, Em: 'static, C, const P: usize, const I: usize> EdgeRd<E, A, P, I> for Hd<Em, C>
where
    C: ChainRead<E, A, P, I>,
{
    type V<'a> = <C as SelectChain<E::Stored>>::Read<'a, E>;
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_v<'a>(root: &'a A::Root, outs: &'a A::Outs) -> Self::V<'a> {
        <C as ChainRead<E, A, P, I>>::read(root, outs)
    }
}

/// As [`EdgeRd`], for a component with a handle parent (read from the context of the split).
#[doc(hidden)]
pub trait EdgeRdCx<E: Source<A>, A: Layout, const P: usize, const I: usize>: SelectChain<E::Stored> {
    type V<'a>;
    fn read_vx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> Self::V<'c>
    where
        A::States: SplitAt<P>;
}
impl<E: Source<A>, A: Layout, const P: usize, const I: usize> EdgeRdCx<E, A, P, I> for ()
where
    (): ChainReadCx<E, A, P, I>,
{
    type V<'a> = <() as SelectChain<E::Stored>>::Read<'a, E>;
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_vx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> Self::V<'c>
    where
        A::States: SplitAt<P>,
    {
        <() as ChainReadCx<E, A, P, I>>::read_cx(root, outs, cx)
    }
}
impl<E: Source<A>, A: Layout, S, T, const P: usize, const I: usize> EdgeRdCx<E, A, P, I> for (S, T)
where
    (S, T): ChainReadCx<E, A, P, I>,
{
    type V<'a> = <(S, T) as SelectChain<E::Stored>>::Read<'a, E>;
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_vx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> Self::V<'c>
    where
        A::States: SplitAt<P>,
    {
        <(S, T) as ChainReadCx<E, A, P, I>>::read_cx(root, outs, cx)
    }
}
impl<E: Source<A>, A: Layout, Em: 'static, C, const P: usize, const I: usize> EdgeRdCx<E, A, P, I> for Hd<Em, C>
where
    C: ChainReadCx<E, A, P, I>,
{
    type V<'a> = <C as SelectChain<E::Stored>>::Read<'a, E>;
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_vx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> Self::V<'c>
    where
        A::States: SplitAt<P>,
    {
        <C as ChainReadCx<E, A, P, I>>::read_cx(root, outs, cx)
    }
}

/// The hook (fourth parameter) of the entry `I` of the component at `P`.
#[doc(hidden)]
pub trait HookOf<Ph, const P: usize, const I: usize>: Layout {
    type X: 'static;
}
impl<A: Layout, const P: usize, const I: usize> HookOf<PushPh<A>, P, I> for A
where
    A: NthT<P>,
    <A as NthT<P>>::T: ComponentSubscription,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps:
        PosSelect<EntryPos<I>> + PickEdge<EntryPos<I>, { <<<A as NthT<P>>::T as ComponentSubscription>::Deps as PosSelect<EntryPos<I>>>::POS }>,
{
    type X = <<<A as NthT<P>>::T as ComponentSubscription>::Deps as PickEdge<
        EntryPos<I>,
        { <<<A as NthT<P>>::T as ComponentSubscription>::Deps as PosSelect<EntryPos<I>>>::POS },
    >>::X;
}

// ---------------------------------------------------------------------------
// The push: one handled edge, the items of a run, the run
// ---------------------------------------------------------------------------

/// The plan's facts about the `K`-th item of the range `Q`: the entry position
/// of the handled edge, and its mode (bit 0: test the parent's flag, bit 1:
/// match the selectors; as a `usize` so it can be a const generic).
#[doc(hidden)]
pub struct HiOf<A, Q, const K: usize>(PhantomData<(A, Q)>);
impl<A: Planned, Q: Rg, const K: usize> HiOf<A, Q, K> {
    pub const POS: usize = A::PLAN.hi_pos_of(Q::LO + K);
    pub const MODE: usize = A::PLAN.hi_mode_of(Q::LO + K) as usize;
}

/// Item `K`: if the handled edge fired, `on_update` it.
#[doc(hidden)]
pub trait HItemAt<Cx: Copy, H: ComponentSubscription, const P: usize, const I: usize, const M: usize>: Layout {
    fn item(root: &Self::Root, outs: &Self::Outs, fired: &FiredOf<Self>, me: &mut H, cx: Cx);
}
type EOf<A, const P: usize, const I: usize> = <A as EntryOf<PushPh<A>, P, I>>::E;
type COf<A, const P: usize, const I: usize> = <A as EntryOf<PushPh<A>, P, I>>::Ch;
type EmOf<A, const P: usize, const I: usize> = <<A as HookOf<PushPh<A>, P, I>>::X as HookEm>::Em;

impl<A: Layout, H: ComponentSubscription, const P: usize, const I: usize, const M: usize> HItemAt<(), H, P, I, M> for A
where
    A: NthT<P, T = H>,
    A: EntryOf<PushPh<A>, P, I>,
    A: HookOf<PushPh<A>, P, I>,
    <A as HookOf<PushPh<A>, P, I>>::X: HookEm,
    EOf<A, P, I>: Source<A>,
    COf<A, P, I>: EdgeRd<EOf<A, P, I>, A, P, I>,
    H: UpdateHandler<EmOf<A, P, I>>,
    for<'a> EmOf<A, P, I>: HandledEdge<Arg<'a> = <COf<A, P, I> as EdgeRd<EOf<A, P, I>, A, P, I>>::V<'a>>,
    <H as ComponentSubscription>::Deps: GatherIn<A, P>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn item(root: &A::Root, outs: &A::Outs, fired: &FiredOf<A>, me: &mut H, _: ()) {
        let mode = M;
        if (mode & 1 == 0 || <EOf<A, P, I> as Source<A>>::fires(fired))
            && (mode & 2 == 0
                || <COf<A, P, I> as SelectChain<<EOf<A, P, I> as Vertex>::Stored>>::matches(<EOf<A, P, I> as Source<A>>::fetch(root, outs)))
        {
            let inputs = <<H as ComponentSubscription>::Deps as GatherIn<A, P>>::gather(root, outs);
            <H as UpdateHandler<EmOf<A, P, I>>>::on_update(
                me,
                <H as ComponentSubscription>::input(inputs),
                <COf<A, P, I> as EdgeRd<EOf<A, P, I>, A, P, I>>::read_v(root, outs),
            );
        }
    }
}
impl<'c, 'x, A: Layout, H: ComponentSubscription, const P: usize, const I: usize, const M: usize> HItemAt<&'c CtxOf<'x, A, P>, H, P, I, M> for A
where
    A: NthT<P, T = H>,
    A: EntryOf<PushPh<A>, P, I>,
    A: HookOf<PushPh<A>, P, I>,
    <A as HookOf<PushPh<A>, P, I>>::X: HookEm,
    EOf<A, P, I>: Source<A>,
    COf<A, P, I>: EdgeRdCx<EOf<A, P, I>, A, P, I>,
    H: UpdateHandler<EmOf<A, P, I>>,
    for<'a> EmOf<A, P, I>: HandledEdge<Arg<'a> = <COf<A, P, I> as EdgeRdCx<EOf<A, P, I>, A, P, I>>::V<'a>>,
    <H as ComponentSubscription>::Deps: GatherCx<A, P>,
    A::States: SplitAt<P>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn item(root: &A::Root, outs: &A::Outs, fired: &FiredOf<A>, me: &mut H, cx: &'c CtxOf<'x, A, P>) {
        let mode = M;
        if (mode & 1 == 0 || <EOf<A, P, I> as Source<A>>::fires(fired))
            && (mode & 2 == 0
                || <COf<A, P, I> as SelectChain<<EOf<A, P, I> as Vertex>::Stored>>::matches(<EOf<A, P, I> as Source<A>>::fetch(root, outs)))
        {
            let inputs = <<H as ComponentSubscription>::Deps as GatherCx<A, P>>::gather(root, outs, cx);
            <H as UpdateHandler<EmOf<A, P, I>>>::on_update(
                me,
                <H as ComponentSubscription>::input(inputs),
                <COf<A, P, I> as EdgeRdCx<EOf<A, P, I>, A, P, I>>::read_vx(root, outs, cx),
            );
        }
    }
}

/// The items of handler run `N` as a range.
#[doc(hidden)]
pub struct HItems<A, const N: usize>(PhantomData<A>);
impl<A: Planned, const N: usize> Rg for HItems<A, N> {
    const LO: usize = A::PLAN.hr_start(N);
    const HI: usize = A::PLAN.hr_start(N) + A::PLAN.hr_count(N);
}

#[doc(hidden)]
pub trait HDrive<A: Layout, Cx: Copy, H: ComponentSubscription, const P: usize, Cl>: Rg {
    fn items(root: &A::Root, outs: &A::Outs, fired: &FiredOf<A>, me: &mut H, cx: Cx);
}
macro_rules! h_leaf {
    ($n:literal; $($k:literal)*) => {
        impl<A: Layout, Cx: Copy, H: ComponentSubscription, const P: usize, Q: Rg> HDrive<A, Cx, H, P, Leaf<$n>> for Q
        where
            $(A: HItemAt<Cx, H, P, { HiOf::<A, Q, $k>::POS }, { HiOf::<A, Q, $k>::MODE }>,)*
        {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            #[allow(unused_variables)]
            fn items(root: &A::Root, outs: &A::Outs, fired: &FiredOf<A>, me: &mut H, cx: Cx) {
                $(<A as HItemAt<Cx, H, P, { HiOf::<A, Q, $k>::POS }, { HiOf::<A, Q, $k>::MODE }>>::item(root, outs, fired, &mut *me, cx);)*
            }
        }
    };
}
dataflow_macros::leaf_idx!(h_leaf);
macro_rules! h_inner {
    ($($k:literal)*) => {
        impl<A: Layout, Cx: Copy, H: ComponentSubscription, const P: usize, Q: Rg> HDrive<A, Cx, H, P, Inner> for Q
        where
            $(Ch<Q, $k>: RgCl,
              Ch<Q, $k>: HDrive<A, Cx, H, P, <Ch<Q, $k> as RgCl>::Cl>,)*
        {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            fn items(root: &A::Root, outs: &A::Outs, fired: &FiredOf<A>, me: &mut H, cx: Cx) {
                $(<Ch<Q, $k> as HDrive<A, Cx, H, P, <Ch<Q, $k> as RgCl>::Cl>>::items(root, outs, fired, &mut *me, cx);)*
            }
        }
    };
}
dataflow_macros::inner_idx!(h_inner);

/// The two ways to run a handler: reading stored outputs (by-value parents), or through the split
/// of the states (a handle parent). The plan says which (`Plan::hr_view`).
#[doc(hidden)]
pub struct HVal;
#[doc(hidden)]
pub struct HVw;
#[doc(hidden)]
pub struct HrView<A, const N: usize>(PhantomData<A>);
impl<A: Planned, const N: usize> HrView<A, N> {
    pub const V: bool = A::PLAN.hr_view(N);
}
#[doc(hidden)]
pub trait HSelC<const V: bool> {
    type M;
}
impl HSelC<false> for () {
    type M = HVal;
}
impl HSelC<true> for () {
    type M = HVw;
}
#[doc(hidden)]
pub trait HrM {
    type M;
}
impl<A: Planned, const N: usize> HrM for HrView<A, N>
where
    (): HSelC<{ HrView::<A, N>::V }>,
{
    type M = <() as HSelC<{ HrView::<A, N>::V }>>::M;
}

/// Run the handler items of program node `N` and `update` of the component, in the way its mode
/// says; returns the output.
#[doc(hidden)]
pub trait HVia<A: Layout, const P: usize, const N: usize> {
    fn via(ctx: &mut PushCtx<'_, A>) -> <<A as NthT<P>>::T as Vertex>::Stored
    where
        A: NthT<P>,
        <A as NthT<P>>::T: Vertex;
}
impl<A: Layout, const P: usize, const N: usize> HVia<A, P, N> for HVal
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: GatherIn<A, P>,
    A::States: AtPos<<A as NthT<P>>::T, P>,
    HItems<A, N>: RgCl,
    HItems<A, N>: HDrive<A, (), <A as NthT<P>>::T, P, <HItems<A, N> as RgCl>::Cl>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn via(ctx: &mut PushCtx<'_, A>) -> <<A as NthT<P>>::T as Vertex>::Stored {
        let me = <A::States as AtPos<<A as NthT<P>>::T, P>>::at_mut(&mut *ctx.states);
        <HItems<A, N> as HDrive<A, (), <A as NthT<P>>::T, P, <HItems<A, N> as RgCl>::Cl>>::items(ctx.root, &*ctx.outs, &ctx.fired, &mut *me, ());
        let inputs = <<<A as NthT<P>>::T as ComponentSubscription>::Deps as GatherIn<A, P>>::gather(ctx.root, &*ctx.outs);
        me.update(<<A as NthT<P>>::T as ComponentSubscription>::input(inputs))
    }
}
impl<A: Layout, const P: usize, const N: usize> HVia<A, P, N> for HVw
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: GatherCx<A, P>,
    A::States: SplitAt<P, Child = <A as NthT<P>>::T>,
    HItems<A, N>: RgCl,
    for<'c, 'x> HItems<A, N>: HDrive<A, &'c CtxOf<'x, A, P>, <A as NthT<P>>::T, P, <HItems<A, N> as RgCl>::Cl>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn via(ctx: &mut PushCtx<'_, A>) -> <<A as NthT<P>>::T as Vertex>::Stored {
        let (cx, me) = <A::States as SplitAt<P>>::split_at(&mut *ctx.states);
        <HItems<A, N> as HDrive<A, &CtxOf<'_, A, P>, <A as NthT<P>>::T, P, <HItems<A, N> as RgCl>::Cl>>::items(ctx.root, &*ctx.outs, &ctx.fired, &mut *me, &cx);
        let inputs = <<<A as NthT<P>>::T as ComponentSubscription>::Deps as GatherCx<A, P>>::gather(ctx.root, &*ctx.outs, &cx);
        me.update(<<A as NthT<P>>::T as ComponentSubscription>::input(inputs))
    }
}

/// Run the handler of program node `N`.
#[doc(hidden)]
pub trait HRunAt<Ph, const P: usize, const N: usize>: Layout {
    fn hrun(ctx: &mut PushCtx<'_, Self>);
}

#[cfg_attr(not(debug_assertions), inline(always))]
#[cfg_attr(debug_assertions, inline)]
fn hrun_at<A: Layout, H: Comp, const P: usize, const N: usize>(ctx: &mut PushCtx<'_, A>)
where
    A: NthT<P, T = H>,
    <H as ComponentSubscription>::Deps: Ready<A>,
    HrView<A, N>: HrM,
    <HrView<A, N> as HrM>::M: HVia<A, P, N>,
    A::Outs: AtPos<<H as Vertex>::Stored, P>,
    SrcSlots<A, P>: RgCl,
    SrcSlots<A, P>: Drive<A, SlotWriteBy<A>, <SrcSlots<A, P> as RgCl>::Cl>,
{
    if A::PLAN.hr_guarded(N)
        && !<<H as ComponentSubscription>::Deps as Ready<A>>::ready(ctx.root, &*ctx.outs, &ctx.fired)
    {
        return;
    }
    let out = <<HrView<A, N> as HrM>::M as HVia<A, P, N>>::via(ctx);
    *<A::Outs as AtPos<<H as Vertex>::Stored, P>>::at_mut(&mut *ctx.outs) = out;
    <SrcSlots<A, P> as Drive<A, SlotWriteBy<A>, <SrcSlots<A, P> as RgCl>::Cl>>::drive(ctx);
    if CondAt::<A, P>::FLAG != NO_FLAG {
        ctx.fired.set(CondAt::<A, P>::FLAG);
    }
}

impl<A: Layout, const P: usize, const N: usize> HRunAt<PushPh<A>, P, N> for A
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: Ready<A>,
    HrView<A, N>: HrM,
    <HrView<A, N> as HrM>::M: HVia<A, P, N>,
    A::Outs: AtPos<<<A as NthT<P>>::T as Vertex>::Stored, P>,
    SrcSlots<A, P>: RgCl,
    SrcSlots<A, P>: Drive<A, SlotWriteBy<A>, <SrcSlots<A, P> as RgCl>::Cl>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn hrun(ctx: &mut PushCtx<'_, A>) {
        hrun_at::<A, <A as NthT<P>>::T, P, N>(ctx)
    }
}

impl<A: Layout, Ph, const N: usize> ExecK<A, Ph, HRunK> for Pn<A, N>
where
    A: HRunAt<Ph, { PnC::<A, N>::A0 }, N>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec_k(ctx: &mut PushCtx<'_, A>) {
        <A as HRunAt<Ph, { PnC::<A, N>::A0 }, N>>::hrun(ctx)
    }
}
