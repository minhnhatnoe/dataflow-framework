//! Reading the nodes a component subscribes to: from the stored outputs and
//! the pushed root value while a step runs ([`GatherIn`]), and from the
//! outputs built so far while the pipeline is constructed ([`GatherInit`]).
//!
//! Both are one impl per subscription-tree shape (generated with
//! `leaf_shapes!` / `inner_shape!`). An entry is read through [`Source`], whose
//! blanket impl dispatches on the kind of the node: the root is `&R` (or `()`
//! at construction), a component is a lookup by position in the flat tuples.
//! The lookups are plain `#[inline]`: they are constant descents, and after
//! inlining one field offset.

use std::marker::PhantomData;

use crate::component::{DepList, Sub};
use crate::drive::{FiredOf, FiredOps};
use crate::identity::At;
use crate::node::{CompKind, Edge, HandleKind, Is, NodeKind, RootKind, Vertex};
use crate::plan::{Planned, NO_FLAG};
use crate::select::{ChainKey, SelInit, Select, SelectChain, ViewC, ViewChain};
use crate::split::{SplitAt, StateAt};
use crate::store::AtPos;

/// The storage types of one pipeline (implemented for its `Anchor`).
#[doc(hidden)]
pub trait Layout: Planned {
    /// The root type `R` of `Pipeline<R, _>`.
    type Root: 'static;
    /// The component states, in execution order.
    type States;
    /// The stored outputs, in execution order.
    type Outs;
    /// `States` with every state still to be constructed.
    type StatesOpt: crate::store::Empty + crate::store::Finish<Done = Self::States>;
    /// `Outs` with every output still to be produced.
    type OutsOpt: crate::store::Empty + crate::store::Finish<Done = Self::Outs>;
    /// The `fired` flags of a push.
    type Fired: crate::drive::FiredOps;
}

/// Component position of node `N` in the plan of `A`.
#[doc(hidden)]
pub struct PosOf<A, N>(PhantomData<(A, N)>);
impl<A: Planned, N: Vertex> PosOf<A, N> {
    /// Execution position 0 is the root, so component position = post - 1 (saturating: an invalid plan, e.g. an orphan first, is reported by the plan's check, not by an underflow here).
    pub const P: usize = {
        A::CHECK;
        A::PLAN.post_of_idx(At::<A, N>::INDEX).saturating_sub(1)
    };
}

/// How a node of kind `Self` is read.
#[doc(hidden)]
pub trait SourceKind<N: Vertex, A: Layout>: NodeKind {
    /// The stored value (the output of a by-value node, the handle of a handle component).
    fn fetch<'a>(root: &'a A::Root, outs: &'a A::Outs) -> &'a N::Stored;
    /// What `new` is given while the pipeline is built.
    fn seed<'a>(states: &'a A::StatesOpt, outs: &'a A::OutsOpt) -> <Self as NodeKind>::Init<'a, N>;
    /// Did the node fire in this step?
    fn fires(fired: &FiredOf<A>) -> bool;
}

impl<N: Vertex<Kind = RootKind>, A: Layout> SourceKind<N, A> for RootKind
where
    A::Root: Is<N::Stored>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn fetch<'a>(root: &'a A::Root, _: &'a A::Outs) -> &'a N::Stored {
        root.conv()
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn seed<'a>(_: &'a A::StatesOpt, _: &'a A::OutsOpt) {}
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn fires(_: &FiredOf<A>) -> bool {
        true
    }
}

impl<N: Vertex<Kind = CompKind>, A: Layout> SourceKind<N, A> for CompKind
where
    A::Outs: AtPos<N::Stored, { PosOf::<A, N>::P }>,
    A::OutsOpt: AtPos<Option<N::Stored>, { PosOf::<A, N>::P }>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn fetch<'a>(_: &'a A::Root, outs: &'a A::Outs) -> &'a N::Stored {
        <A::Outs as AtPos<N::Stored, { PosOf::<A, N>::P }>>::at(outs)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn seed<'a>(_: &'a A::StatesOpt, outs: &'a A::OutsOpt) -> &'a N::Stored {
        <A::OutsOpt as AtPos<Option<N::Stored>, { PosOf::<A, N>::P }>>::at(outs)
            .as_ref()
            .expect("a subscription is constructed before its subscriber")
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn fires(fired: &FiredOf<A>) -> bool {
        // A component nobody's condition reads has no flag, and nobody asks
        // (the plan gives a flag to every component a conditional component
        // reads).
        if FlagOf::<A, N>::ALWAYS || FlagOf::<A, N>::FLAG == NO_FLAG {
            true
        } else {
            fired.get(FlagOf::<A, N>::FLAG)
        }
    }
}

impl<N: crate::node::HandleView, A: Layout> SourceKind<N, A> for HandleKind
where
    A::Outs: AtPos<N::Stored, { PosOf::<A, N>::P }>,
    A::OutsOpt: AtPos<Option<N::Stored>, { PosOf::<A, N>::P }>,
    A::StatesOpt: AtPos<Option<N>, { PosOf::<A, N>::P }>,
{
    /// The stored handle (the view needs the state too: see [`SourceCx`]).
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn fetch<'a>(_: &'a A::Root, outs: &'a A::Outs) -> &'a N::Stored {
        <A::Outs as AtPos<N::Stored, { PosOf::<A, N>::P }>>::at(outs)
    }
    /// The view of the constructed parent and its first handle.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn seed<'a>(states: &'a A::StatesOpt, outs: &'a A::OutsOpt) -> <N as Edge>::Read<'a> {
        let state = <A::StatesOpt as AtPos<Option<N>, { PosOf::<A, N>::P }>>::at(states)
            .as_ref()
            .expect("a subscription is constructed before its subscriber");
        let handle = <A::OutsOpt as AtPos<Option<N::Stored>, { PosOf::<A, N>::P }>>::at(outs)
            .as_ref()
            .expect("a subscription is constructed before its subscriber");
        N::__view(state, handle)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn fires(fired: &FiredOf<A>) -> bool {
        if FlagOf::<A, N>::ALWAYS || FlagOf::<A, N>::FLAG == NO_FLAG {
            true
        } else {
            fired.get(FlagOf::<A, N>::FLAG)
        }
    }
}

/// A node that can be read in the pipeline `A`.
#[doc(hidden)]
pub trait Source<A: Layout>: Vertex {
    fn fetch<'a>(root: &'a A::Root, outs: &'a A::Outs) -> &'a Self::Stored;
    fn seed<'a>(states: &'a A::StatesOpt, outs: &'a A::OutsOpt) -> crate::node::InitOf<'a, Self>;
    fn fires(fired: &FiredOf<A>) -> bool;
}
impl<A: Layout, N: Vertex> Source<A> for N
where
    N::Kind: SourceKind<N, A>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn fetch<'a>(root: &'a A::Root, outs: &'a A::Outs) -> &'a N::Stored {
        <N::Kind as SourceKind<N, A>>::fetch(root, outs)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn seed<'a>(states: &'a A::StatesOpt, outs: &'a A::OutsOpt) -> crate::node::InitOf<'a, N> {
        <N::Kind as SourceKind<N, A>>::seed(states, outs)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn fires(fired: &FiredOf<A>) -> bool {
        <N::Kind as SourceKind<N, A>>::fires(fired)
    }
}

/// The inputs of one component's `update`, read from the pushed root and the
/// stored outputs.
#[doc(hidden)]
pub trait GatherIn<A: Layout, const P: usize>: DepList {
    fn gather<'a>(root: &'a A::Root, outs: &'a A::Outs) -> Self::Inputs<'a>;
}
macro_rules! gather_in_leaf {
    ([$($E:ident $C:ident $I:ident $X:ident)*]) => {
        impl<A: Layout, const P: usize, $($E: Source<A>, $C: ChainRead<$E, A, P, $I>, const $I: usize, $X: 'static,)*> GatherIn<A, P> for ($(Sub<$E, $C, $I, $X>,)*) {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            #[allow(unused_variables)]
            fn gather<'a>(root: &'a A::Root, outs: &'a A::Outs) -> Self::Inputs<'a> {
                ($(<$C as ChainRead<$E, A, P, $I>>::read(root, outs),)*)
            }
        }
    };
}
dataflow_macros::leaf_shapes!(gather_in_leaf);
macro_rules! gather_in_inner {
    ([$($K:ident)*]) => {
        impl<A: Layout, const P: usize, $($K: GatherIn<A, P>,)*> GatherIn<A, P> for ($($K,)*) {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            fn gather<'a>(root: &'a A::Root, outs: &'a A::Outs) -> Self::Inputs<'a> {
                ($(<$K as GatherIn<A, P>>::gather(root, outs),)*)
            }
        }
    };
}
dataflow_macros::inner_shape!(gather_in_inner);

/// What one component's `new` is given, read from the outputs built so far.
#[doc(hidden)]
pub trait GatherInit<A: Layout, const P: usize>: DepList {
    fn gather<'a>(states: &'a A::StatesOpt, outs: &'a A::OutsOpt) -> Self::Inits<'a>;
}
macro_rules! gather_init_leaf {
    ([$($E:ident $C:ident $I:ident $X:ident)*]) => {
        impl<A: Layout, const P: usize, $($E: Source<A>, $C: ChainSeed<$E, A, P, $I>, const $I: usize, $X: 'static,)*> GatherInit<A, P> for ($(Sub<$E, $C, $I, $X>,)*) {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            #[allow(unused_variables)]
            fn gather<'a>(states: &'a A::StatesOpt, outs: &'a A::OutsOpt) -> Self::Inits<'a> {
                ($(<$C as ChainSeed<$E, A, P, $I>>::seed(states, outs),)*)
            }
        }
    };
}
dataflow_macros::leaf_shapes!(gather_init_leaf);
macro_rules! gather_init_inner {
    ([$($K:ident)*]) => {
        impl<A: Layout, const P: usize, $($K: GatherInit<A, P>,)*> GatherInit<A, P> for ($($K,)*) {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            fn gather<'a>(states: &'a A::StatesOpt, outs: &'a A::OutsOpt) -> Self::Inits<'a> {
                ($(<$K as GatherInit<A, P>>::gather(states, outs),)*)
            }
        }
    };
}
dataflow_macros::inner_shape!(gather_init_inner);

/// What `new` is given for the entry `I` of the component at position `P`, with
/// this selector chain from node `N`: the stored output for a plain edge; for a
/// selected one the initial value of its slot, if it has a slot (it is read in
/// steps where the selectors do not match), else nothing ([`SelInit`]).
#[doc(hidden)]
pub trait ChainSeed<N: Vertex, A: Layout, const P: usize, const I: usize>: SelectChain<N::Stored> {
    fn seed<'a>(states: &'a A::StatesOpt, outs: &'a A::OutsOpt) -> <Self as SelectChain<N::Stored>>::Init<'a, N>;
}
impl<N: Source<A>, A: Layout, const P: usize, const I: usize> ChainSeed<N, A, P, I> for () {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn seed<'a>(states: &'a A::StatesOpt, outs: &'a A::OutsOpt) -> crate::node::InitOf<'a, N> {
        <N as Source<A>>::seed(states, outs)
    }
}
impl<N: Vertex, A: Layout, S: Select<N::Stored>, T: SelectChain<S::Out>, const P: usize, const I: usize> ChainSeed<N, A, P, I> for (S, T)
where
    SlotAt<A, P, I>: ModeTy,
    (S, T): SeedVia<N, A, <SlotAt<A, P, I> as ModeTy>::Mode, { SlotAt::<A, P, I>::POS }>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn seed<'a>(_: &'a A::StatesOpt, outs: &'a A::OutsOpt) -> <Self as SelectChain<N::Stored>>::Init<'a, N> {
        <(S, T) as SeedVia<N, A, <SlotAt<A, P, I> as ModeTy>::Mode, { SlotAt::<A, P, I>::POS }>>::seed_via(outs)
    }
}

/// `new` of the subscriber of a handle component through selectors gets the selected part of the
/// view of the constructed parent (as of its first handle) if the chain matches it, else `None`.
impl<N: crate::node::HandleView + Source<A>, A: Layout, C: ViewChain<N>, const P: usize, const I: usize> ChainSeed<N, A, P, I> for ViewC<N, C> {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn seed<'a>(states: &'a A::StatesOpt, outs: &'a A::OutsOpt) -> <Self as SelectChain<N::Stored>>::Init<'a, N> {
        C::try_get(<N as Source<A>>::seed(states, outs))
    }
}

/// How an entry with a selector chain reaches its value: directly through its
/// source, or through a slot (see `SlotNode`). The plan decides, per entry.
#[doc(hidden)]
pub struct SlotAt<A, const P: usize, const I: usize>(PhantomData<A>);
impl<A: Planned, const P: usize, const I: usize> SlotAt<A, P, I> {
    /// The storage position of the slot the entry reads (`usize::MAX`: none).
    pub const POS: usize = A::PLAN.edge_slot_pos(P, I);
    pub const MODE: u8 = if Self::POS == usize::MAX { 0 } else { 1 };
}
#[doc(hidden)]
pub trait ModeTy {
    type Mode;
}
impl<A: Planned, const P: usize, const I: usize> ModeTy for SlotAt<A, P, I>
where
    (): ModeOf<{ SlotAt::<A, P, I>::MODE }>,
{
    type Mode = <() as ModeOf<{ SlotAt::<A, P, I>::MODE }>>::Ty;
}
#[doc(hidden)]
pub struct Direct;
#[doc(hidden)]
pub struct Stored;
#[doc(hidden)]
pub trait ModeOf<const M: u8> {
    type Ty;
}
impl ModeOf<0> for () {
    type Ty = Direct;
}
impl ModeOf<1> for () {
    type Ty = Stored;
}

/// `new`'s value for a selected entry, by mode.
#[doc(hidden)]
pub trait SeedVia<N: Vertex, A: Layout, Mode, const POS: usize>: SelectChain<N::Stored> {
    fn seed_via<'a>(outs: &'a A::OutsOpt) -> <Self as SelectChain<N::Stored>>::Init<'a, N>;
}
impl<N: Vertex, A: Layout, S: Select<N::Stored>, T: SelectChain<S::Out>, const POS: usize> SeedVia<N, A, Direct, POS> for (S, T) {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn seed_via<'a>(_: &'a A::OutsOpt) -> SelInit<'a, T::Out> {
        SelInit::empty()
    }
}
impl<N: Vertex, A: Layout, S: Select<N::Stored>, T: SelectChain<S::Out>, const POS: usize> SeedVia<N, A, Stored, POS> for (S, T)
where
    A::OutsOpt: AtPos<Option<T::Out>, POS>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn seed_via<'a>(outs: &'a A::OutsOpt) -> SelInit<'a, T::Out> {
        SelInit::slot(
            <A::OutsOpt as AtPos<Option<T::Out>, POS>>::at(outs)
                .as_ref()
                .expect("a stored selected value is initialised before any subscriber is constructed"),
        )
    }
}

/// How the entry `I` of the component at position `P` is read in `update`: the
/// selector chain applied to the source's output, or, if the entry is stored,
/// the slot.
#[doc(hidden)]
pub trait ChainRead<E: Source<A>, A: Layout, const P: usize, const I: usize>: SelectChain<E::Stored> {
    fn read<'a>(root: &'a A::Root, outs: &'a A::Outs) -> <Self as SelectChain<E::Stored>>::Read<'a, E>;
}
impl<E: Source<A> + crate::node::Node, A: Layout, const P: usize, const I: usize> ChainRead<E, A, P, I> for () {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read<'a>(root: &'a A::Root, outs: &'a A::Outs) -> crate::node::ReadOf<'a, E> {
        <E as Source<A>>::fetch(root, outs)
    }
}
impl<E: Source<A>, A: Layout, S: Select<E::Stored>, T: SelectChain<S::Out>, const P: usize, const I: usize> ChainRead<E, A, P, I> for (S, T)
where
    SlotAt<A, P, I>: ModeTy,
    (S, T): ReadVia<E, A, <SlotAt<A, P, I> as ModeTy>::Mode, { SlotAt::<A, P, I>::POS }>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read<'a>(root: &'a A::Root, outs: &'a A::Outs) -> <(S, T) as SelectChain<E::Stored>>::Read<'a, E> {
        <(S, T) as ReadVia<E, A, <SlotAt<A, P, I> as ModeTy>::Mode, { SlotAt::<A, P, I>::POS }>>::read_via(root, outs)
    }
}
#[doc(hidden)]
pub trait ReadVia<E: Source<A>, A: Layout, Mode, const POS: usize>: SelectChain<E::Stored> {
    fn read_via<'a>(root: &'a A::Root, outs: &'a A::Outs) -> <Self as SelectChain<E::Stored>>::Read<'a, E>;
}
impl<E: Source<A>, A: Layout, S: Select<E::Stored>, T: SelectChain<S::Out>, const POS: usize> ReadVia<E, A, Direct, POS> for (S, T) {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_via<'a>(root: &'a A::Root, outs: &'a A::Outs) -> &'a T::Out {
        <(S, T)>::get(<E as Source<A>>::fetch(root, outs))
    }
}
impl<E: Source<A>, A: Layout, S: Select<E::Stored>, T: SelectChain<S::Out>, const POS: usize> ReadVia<E, A, Stored, POS> for (S, T)
where
    A::Outs: AtPos<T::Out, POS>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_via<'a>(_: &'a A::Root, outs: &'a A::Outs) -> &'a T::Out {
        <A::Outs as AtPos<T::Out, POS>>::at(outs)
    }
}

// ---------------------------------------------------------------------------
// Reading in a component that has a handle parent
// ---------------------------------------------------------------------------
//
// Such a component is updated through the split of the states (`split::SplitAt`): `&mut` to its
// own state and, in the context, shared references to everything before it. A handle parent's
// field is its view, computed from its state (picked from the context) and its stored handle. The
// other entries are read as in `GatherIn`.

/// The context of the split of the states at position `P`.
#[doc(hidden)]
pub type CtxOf<'x, A, const P: usize> = <<A as Layout>::States as SplitAt<P>>::Ctx<'x>;

/// How a node is read from the context of the split at position `P`.
#[doc(hidden)]
pub trait SourceKindCx<N: Vertex, A: Layout, const P: usize>: NodeKind {
    fn read_cx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> <N as Edge>::Read<'c>
    where
        A::States: SplitAt<P>;
}
impl<N: crate::node::Node<Kind = RootKind>, A: Layout, const P: usize> SourceKindCx<N, A, P> for RootKind
where
    A::Root: Is<N::Output>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_cx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, _: &'c CtxOf<'x, A, P>) -> <N as Edge>::Read<'c>
    where
        A::States: SplitAt<P>,
    {
        <RootKind as SourceKind<N, A>>::fetch(root, outs)
    }
}
impl<N: crate::node::Node<Kind = CompKind>, A: Layout, const P: usize> SourceKindCx<N, A, P> for CompKind
where
    A::Outs: AtPos<N::Output, { PosOf::<A, N>::P }>,
    A::OutsOpt: AtPos<Option<N::Output>, { PosOf::<A, N>::P }>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_cx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, _: &'c CtxOf<'x, A, P>) -> <N as Edge>::Read<'c>
    where
        A::States: SplitAt<P>,
    {
        <CompKind as SourceKind<N, A>>::fetch(root, outs)
    }
}
impl<N: crate::node::HandleView, A: Layout, const P: usize> SourceKindCx<N, A, P> for HandleKind
where
    A::Outs: AtPos<N::Stored, { PosOf::<A, N>::P }>,
    A::OutsOpt: AtPos<Option<N::Stored>, { PosOf::<A, N>::P }>,
    A::States: StateAt<P, { PosOf::<A, N>::P }, N>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_cx<'c, 'x>(_: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> <N as Edge>::Read<'c>
    where
        A::States: SplitAt<P>,
    {
        let state = <A::States as StateAt<P, { PosOf::<A, N>::P }, N>>::state(cx);
        let handle = <A::Outs as AtPos<N::Stored, { PosOf::<A, N>::P }>>::at(outs);
        N::__view(state, handle)
    }
}

/// A node readable from the context of the split at `P`, whatever its kind.
#[doc(hidden)]
pub trait SourceCx<A: Layout, const P: usize>: Vertex {
    fn read_cx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> <Self as Edge>::Read<'c>
    where
        A::States: SplitAt<P>;
}
impl<A: Layout, N: Vertex, const P: usize> SourceCx<A, P> for N
where
    N::Kind: SourceKindCx<N, A, P>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_cx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> <N as Edge>::Read<'c>
    where
        A::States: SplitAt<P>,
    {
        <N::Kind as SourceKindCx<N, A, P>>::read_cx(root, outs, cx)
    }
}

/// Entry `I` of the component at `P`, read from the context: a plain entry through its source, a
/// selected one as in [`ChainRead`] (selectors on a handle component are not supported yet).
#[doc(hidden)]
pub trait ChainReadCx<E: Source<A>, A: Layout, const P: usize, const I: usize>: SelectChain<E::Stored> {
    fn read_cx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> <Self as SelectChain<E::Stored>>::Read<'c, E>
    where
        A::States: SplitAt<P>;
}
impl<E: Source<A> + SourceCx<A, P>, A: Layout, const P: usize, const I: usize> ChainReadCx<E, A, P, I> for () {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_cx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> crate::node::ReadOf<'c, E>
    where
        A::States: SplitAt<P>,
    {
        <E as SourceCx<A, P>>::read_cx(root, outs, cx)
    }
}
impl<E: Source<A>, A: Layout, S: Select<E::Stored>, T: SelectChain<S::Out>, const P: usize, const I: usize> ChainReadCx<E, A, P, I> for (S, T)
where
    (S, T): ChainRead<E, A, P, I>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_cx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, _: &'c CtxOf<'x, A, P>) -> <(S, T) as SelectChain<E::Stored>>::Read<'c, E>
    where
        A::States: SplitAt<P>,
    {
        <(S, T) as ChainRead<E, A, P, I>>::read(root, outs)
    }
}

/// A selected entry whose source is a handle component: the chain applied to the view, which is
/// read from the context like a plain handle entry. The selected part is never stored.
impl<E: crate::node::HandleView + Source<A> + SourceCx<A, P>, A: Layout, C: ViewChain<E>, const P: usize, const I: usize> ChainReadCx<E, A, P, I> for ViewC<E, C> {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn read_cx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> <Self as SelectChain<E::Stored>>::Read<'c, E>
    where
        A::States: SplitAt<P>,
    {
        C::get(<E as SourceCx<A, P>>::read_cx(root, outs, cx))
    }
}

/// The inputs of a component with a handle parent, read from the pushed root, the stored outputs
/// and the context of the split of the states.
#[doc(hidden)]
pub trait GatherCx<A: Layout, const P: usize>: DepList {
    fn gather<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> Self::Inputs<'c>
    where
        A::States: SplitAt<P>;
}
macro_rules! gather_cx_leaf {
    ([$($E:ident $C:ident $I:ident $X:ident)*]) => {
        impl<A: Layout, const P: usize, $($E: Source<A>, $C: ChainReadCx<$E, A, P, $I>, const $I: usize, $X: 'static,)*> GatherCx<A, P> for ($(Sub<$E, $C, $I, $X>,)*) {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            #[allow(unused_variables)]
            fn gather<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> Self::Inputs<'c>
            where
                A::States: SplitAt<P>,
            {
                ($(<$C as ChainReadCx<$E, A, P, $I>>::read_cx(root, outs, cx),)*)
            }
        }
    };
}
dataflow_macros::leaf_shapes!(gather_cx_leaf);
macro_rules! gather_cx_inner {
    ([$($K:ident)*]) => {
        impl<A: Layout, const P: usize, $($K: GatherCx<A, P>,)*> GatherCx<A, P> for ($($K,)*) {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            fn gather<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> Self::Inputs<'c>
            where
                A::States: SplitAt<P>,
            {
                ($(<$K as GatherCx<A, P>>::gather(root, outs, cx),)*)
            }
        }
    };
}
dataflow_macros::inner_shape!(gather_cx_inner);

// ---------------------------------------------------------------------------
// Firing
// ---------------------------------------------------------------------------

/// The plan's facts about node `N` in pipeline `A`, as constants.
#[doc(hidden)]
pub struct FlagOf<A, N>(PhantomData<(A, N)>);
impl<A: Planned, N: Vertex> FlagOf<A, N> {
    /// Does `N` run in every step?
    pub const ALWAYS: bool = A::PLAN.always_idx(At::<A, N>::INDEX);
    /// Its `fired` flag.
    pub const FLAG: usize = A::PLAN.flag_idx(At::<A, N>::INDEX);
}

/// Does the selector chain of an entry match the current value of its source? From the pushed
/// root and the stored outputs (a by-value source), or, for a handle source, from its state and
/// stored handle (`test`); inside the split of the states, the state comes from the context
/// (`ChainTestCx`).
#[doc(hidden)]
pub trait ChainTest<E: Source<A>, A: Layout>: SelectChain<E::Stored> {
    fn test(root: &A::Root, states: &A::States, outs: &A::Outs) -> bool;
}
impl<E: Source<A>, A: Layout> ChainTest<E, A> for () {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn test(_: &A::Root, _: &A::States, _: &A::Outs) -> bool {
        true
    }
}
impl<E: Source<A>, A: Layout, S: Select<E::Stored>, T: SelectChain<S::Out>> ChainTest<E, A> for (S, T) {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn test(root: &A::Root, _: &A::States, outs: &A::Outs) -> bool {
        <(S, T) as SelectChain<E::Stored>>::matches(<E as Source<A>>::fetch(root, outs))
    }
}
impl<E: crate::node::HandleView + Source<A>, A: Layout, C: ViewChain<E>> ChainTest<E, A> for ViewC<E, C>
where
    A::States: AtPos<E, { PosOf::<A, E>::P }>,
    A::Outs: AtPos<E::Stored, { PosOf::<A, E>::P }>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn test(_: &A::Root, states: &A::States, outs: &A::Outs) -> bool {
        let state = <A::States as AtPos<E, { PosOf::<A, E>::P }>>::at(states);
        let handle = <A::Outs as AtPos<E::Stored, { PosOf::<A, E>::P }>>::at(outs);
        C::matches(E::__view(state, handle))
    }
}

/// The key of the alternative the chain is at depth `J` at a *fused* switch: from the source `E`'s
/// state `me` and the value `o` its `update` just returned (before it is stored): `o` itself for
/// a by-value source, the view of `me` and the handle `o` for a handle source.
#[doc(hidden)]
pub trait ChainKeyPre<E: Vertex, const J: usize>: SelectChain<E::Stored> {
    fn key_pre(me: &E, o: &E::Stored) -> u32;
}
impl<E: Vertex, const J: usize> ChainKeyPre<E, J> for () {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key_pre(_: &E, _: &E::Stored) -> u32 {
        crate::select::GUARD
    }
}
impl<E: Vertex, S: Select<E::Stored>, T: SelectChain<S::Out>, const J: usize> ChainKeyPre<E, J> for (S, T)
where
    (S, T): ChainKey<E::Stored, J>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key_pre(_: &E, o: &E::Stored) -> u32 {
        <(S, T) as ChainKey<E::Stored, J>>::key(o)
    }
}
impl<E: crate::node::HandleView, C: ViewChain<E>, const J: usize> ChainKeyPre<E, J> for ViewC<E, C> {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key_pre(me: &E, o: &E::Stored) -> u32 {
        C::key::<J>(me.__view(o))
    }
}

/// As [`ChainTest`], inside the split of the states at position `P` (a handle source's state is
/// read from the context).
#[doc(hidden)]
pub trait ChainTestCx<E: Source<A>, A: Layout, const P: usize>: SelectChain<E::Stored> {
    fn test_cx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> bool
    where
        A::States: SplitAt<P>;
}
impl<E: Source<A>, A: Layout, const P: usize> ChainTestCx<E, A, P> for () {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn test_cx<'c, 'x>(_: &'c A::Root, _: &'c A::Outs, _: &'c CtxOf<'x, A, P>) -> bool
    where
        A::States: SplitAt<P>,
    {
        true
    }
}
impl<E: Source<A>, A: Layout, S: Select<E::Stored>, T: SelectChain<S::Out>, const P: usize> ChainTestCx<E, A, P> for (S, T) {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn test_cx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, _: &'c CtxOf<'x, A, P>) -> bool
    where
        A::States: SplitAt<P>,
    {
        <(S, T) as SelectChain<E::Stored>>::matches(<E as Source<A>>::fetch(root, outs))
    }
}
impl<E: crate::node::HandleView + Source<A> + SourceCx<A, P>, A: Layout, C: ViewChain<E>, const P: usize> ChainTestCx<E, A, P> for ViewC<E, C> {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn test_cx<'c, 'x>(root: &'c A::Root, outs: &'c A::Outs, cx: &'c CtxOf<'x, A, P>) -> bool
    where
        A::States: SplitAt<P>,
    {
        C::matches(<E as SourceCx<A, P>>::read_cx(root, outs, cx))
    }
}

/// The key of the alternative the chain is at depth `J`, for the switch on a source's value: from
/// the pushed root or the stored output, or, for a handle source, from its view.
#[doc(hidden)]
pub trait ChainKeyAt<E: Source<A>, A: Layout, const J: usize>: SelectChain<E::Stored> {
    fn key_at(root: &A::Root, states: &A::States, outs: &A::Outs) -> u32;
}
impl<E: Source<A>, A: Layout, const J: usize> ChainKeyAt<E, A, J> for () {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key_at(_: &A::Root, _: &A::States, _: &A::Outs) -> u32 {
        crate::select::GUARD
    }
}
impl<E: Source<A>, A: Layout, S: Select<E::Stored>, T: SelectChain<S::Out>, const J: usize> ChainKeyAt<E, A, J> for (S, T)
where
    (S, T): ChainKey<E::Stored, J>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key_at(root: &A::Root, _: &A::States, outs: &A::Outs) -> u32 {
        <(S, T) as ChainKey<E::Stored, J>>::key(<E as Source<A>>::fetch(root, outs))
    }
}
impl<E: crate::node::HandleView + Source<A>, A: Layout, C: ViewChain<E>, const J: usize> ChainKeyAt<E, A, J> for ViewC<E, C>
where
    A::States: AtPos<E, { PosOf::<A, E>::P }>,
    A::Outs: AtPos<E::Stored, { PosOf::<A, E>::P }>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key_at(_: &A::Root, states: &A::States, outs: &A::Outs) -> u32 {
        let state = <A::States as AtPos<E, { PosOf::<A, E>::P }>>::at(states);
        let handle = <A::Outs as AtPos<E::Stored, { PosOf::<A, E>::P }>>::at(outs);
        C::key::<J>(E::__view(state, handle))
    }
}

/// Has any edge of a subscription tree fired? An edge fires if its node fired
/// and its selectors match the node's current value (the pushed root, or the
/// stored output, or a handle component's view of its state).
#[doc(hidden)]
pub trait Ready<A: Layout>: DepList {
    fn ready(root: &A::Root, states: &A::States, outs: &A::Outs, fired: &FiredOf<A>) -> bool;
}
macro_rules! ready_leaf {
    ([$($E:ident $C:ident $I:ident $X:ident)*]) => {
        impl<A: Layout, $($E: Source<A>, $C: ChainTest<$E, A>, const $I: usize, $X: 'static,)*> Ready<A> for ($(Sub<$E, $C, $I, $X>,)*) {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            #[allow(unused_variables)]
            fn ready(root: &A::Root, states: &A::States, outs: &A::Outs, fired: &FiredOf<A>) -> bool {
                false $(|| (<$E as Source<A>>::fires(fired) && <$C as ChainTest<$E, A>>::test(root, states, outs)))*
            }
        }
    };
}
dataflow_macros::leaf_shapes!(ready_leaf);
macro_rules! ready_inner {
    ([$($K:ident)*]) => {
        impl<A: Layout, $($K: Ready<A>,)*> Ready<A> for ($($K,)*) {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            fn ready(root: &A::Root, states: &A::States, outs: &A::Outs, fired: &FiredOf<A>) -> bool {
                false $(|| <$K as Ready<A>>::ready(root, states, outs, fired))*
            }
        }
    };
}
dataflow_macros::inner_shape!(ready_inner);
