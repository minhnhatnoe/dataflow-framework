//! Driving a *phase* over the component positions in execution order: the
//! construction pass and a push. A phase is a type with a context; the work at
//! one position is `StepAt<Phase, P>`, a blanket impl with all the bounds, and
//! the driver recurses over the balanced ranges of positions (the same shape as
//! the storage tuples) and calls `step` for each position in order. After
//! inlining a push is straight-line code, as for a hand-written struct of the
//! components.
//!
//! # The push program
//!
//! A push is a program built by the plan (`Plan::build_program`), not a run of
//! per-component guards: `Seq` (children in execution order), `Run` (a component,
//! unconditionally), `RunIf` (guarded by the firing of its edges; only for
//! conditions the trie cannot express, such as guards) and `Switch` (the key of
//! the alternative a value is, from `Select::key_of`, evaluated once; then a
//! decision tree over the sorted constant keys of the sibling decisions, which
//! LLVM lowers to a jump table). Leaves run their cones straight-line; a
//! component joined across exclusive terms is repeated in each leaf. Order is
//! kept: every list is in execution order and a switch covers only a run of
//! consecutive components.

use std::any::type_name;
use std::marker::PhantomData;

use crate::build::ConstructPass;
use crate::component::{Component, ComponentSubscription};
use crate::flat::{bound, class, ClassOf, EntryPos, Inner, Leaf, Loc, NthT, PickEdge, PosSelect, Whole};
use crate::gather::{GatherIn, GatherInit, Layout, Ready, Source};
use crate::select::ChainKey;
use crate::node::{Comp, Node};
use crate::plan::{Planned, NO_FLAG};
use crate::store::AtPos;

// ---------------------------------------------------------------------------
// Ranges of component positions as types
// ---------------------------------------------------------------------------

#[doc(hidden)]
pub trait Rg: 'static {
    const LO: usize;
    const HI: usize;
}
impl<R: Planned + 'static> Rg for Whole<R> {
    const LO: usize = 0;
    const HI: usize = Whole::<R>::N;
}
/// Child `K` of the node covering `Q`.
#[doc(hidden)]
pub struct Ch<Q, const K: usize>(PhantomData<Q>);
impl<Q: Rg, const K: usize> Rg for Ch<Q, K> {
    const LO: usize = bound(Q::LO, Q::HI, K);
    const HI: usize = bound(Q::LO, Q::HI, K + 1);
}
#[doc(hidden)]
pub struct ClsOf<Q>(PhantomData<Q>);
impl<Q: Rg> ClsOf<Q> {
    pub const C: usize = class(Q::LO, Q::HI);
}
#[doc(hidden)]
pub trait RgCl {
    type Cl;
}
impl<Q: Rg> RgCl for Q
where
    (): ClassOf<{ ClsOf::<Q>::C }>,
{
    type Cl = <() as ClassOf<{ ClsOf::<Q>::C }>>::Ty;
}
#[doc(hidden)]
pub struct PosAt<Q, const K: usize>(PhantomData<Q>);
impl<Q: Rg, const K: usize> PosAt<Q, K> {
    pub const P: usize = Q::LO + K;
}

// ---------------------------------------------------------------------------
// Phases and the driver
// ---------------------------------------------------------------------------

/// A pass over the positions, with the context it carries.
#[doc(hidden)]
pub trait Phase {
    type Ctx<'c>;
}

/// The work of phase `Ph` at component position `P`.
///
/// Fails to hold when the graph is not valid: typically a second
/// `#[derive(Root)]` type (which takes a component position), or a component
/// whose subscriptions don't match.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "the dataflow graph is not valid: the node at component position {P} can't be run (a second root in the graph, or a subscription that is not a node of this pipeline?)",
    label = "`{Self}` can't run position {P}"
)]
pub trait StepAt<Ph: Phase, const P: usize>: Planned {
    fn step(ctx: &mut Ph::Ctx<'_>);
}

/// Run phase `Ph` over the range `Self`, class `Cl`, in position order.
#[doc(hidden)]
pub trait Drive<A: Planned, Ph: Phase, Cl>: Rg {
    fn drive(ctx: &mut Ph::Ctx<'_>);
}

macro_rules! drive_leaf {
    ($n:literal; $($k:literal)*) => {
        impl<A: Planned, Ph: Phase, Q: Rg> Drive<A, Ph, Leaf<$n>> for Q
        where
            $(A: StepAt<Ph, { PosAt::<Q, $k>::P }>,)*
        {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            #[allow(unused_variables)]
            fn drive(ctx: &mut Ph::Ctx<'_>) {
                $(<A as StepAt<Ph, { PosAt::<Q, $k>::P }>>::step(ctx);)*
            }
        }
    };
}
dataflow_macros::leaf_idx!(drive_leaf);

macro_rules! drive_inner {
    ($($k:literal)*) => {
        impl<A: Planned, Ph: Phase, Q: Rg> Drive<A, Ph, Inner> for Q
        where
            $(Ch<Q, $k>: RgCl,
              Ch<Q, $k>: Drive<A, Ph, <Ch<Q, $k> as RgCl>::Cl>,)*
        {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            fn drive(ctx: &mut Ph::Ctx<'_>) {
                $(<Ch<Q, $k> as Drive<A, Ph, <Ch<Q, $k> as RgCl>::Cl>>::drive(ctx);)*
            }
        }
    };
}
dataflow_macros::inner_idx!(drive_inner);

// ---------------------------------------------------------------------------
// Construction
// ---------------------------------------------------------------------------

#[doc(hidden)]
pub struct ConstructPh<A>(PhantomData<A>);
#[doc(hidden)]
pub struct ConstructCtx<'c, A: Layout> {
    pub(crate) states: &'c mut A::StatesOpt,
    pub(crate) outs: &'c mut A::OutsOpt,
    pub(crate) pass: &'c mut ConstructPass,
}
impl<A: Layout> Phase for ConstructPh<A> {
    type Ctx<'c> = ConstructCtx<'c, A>;
}

impl<A: Layout, const P: usize> StepAt<ConstructPh<A>, P> for A
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: GatherInit<A>,
    A::StatesOpt: AtPos<Option<<A as NthT<P>>::T>, P>,
    A::OutsOpt: AtPos<Option<<<A as NthT<P>>::T as Node>::Output>, P>,
{
    fn step(ctx: &mut ConstructCtx<'_, A>) {
        type C<A, const P: usize> = <A as NthT<P>>::T;
        let config = ctx.pass.take_config::<C<A, P>>();
        let (state, out) = {
            let raw = <<C<A, P> as ComponentSubscription>::Deps as GatherInit<A>>::gather(&*ctx.outs);
            <C<A, P> as crate::component::Component>::new(config, <C<A, P> as ComponentSubscription>::init(raw))
        };
        *<A::StatesOpt as AtPos<Option<C<A, P>>, P>>::at_mut(&mut *ctx.states) = Some(state);
        *<A::OutsOpt as AtPos<Option<<C<A, P> as Node>::Output>, P>>::at_mut(&mut *ctx.outs) = Some(out);
        ctx.pass.order.push(type_name::<C<A, P>>());
    }
}

// ---------------------------------------------------------------------------
// Push
// ---------------------------------------------------------------------------

/// Which components fired in this step, by flag (see
/// [`Plan`](crate::plan::Plan): only components whose firing a conditional
/// component reads have a flag), indexed by constants. After inlining the
/// array is split into locals and the unused flags vanish: with no conditions
/// there is no `fired` state in the generated code.
#[doc(hidden)]
pub struct Fired<const N: usize> {
    bits: [bool; N],
}
/// The number of `fired` flags pipeline `A` needs.
#[doc(hidden)]
pub struct NFlags<A>(PhantomData<A>);
impl<A: Planned> NFlags<A> {
    pub const N: usize = A::PLAN.n_flag_slots();
}
/// `Fired<N>` for a constant, as a type (the pattern of `flat::RngTy`: the
/// constant lives in the where clause of an impl, not in an associated type).
#[doc(hidden)]
pub trait FiredSized<const N: usize> {
    type T: FiredOps;
}
impl<const N: usize> FiredSized<N> for () {
    type T = Fired<N>;
}
#[doc(hidden)]
pub trait FiredTy {
    type T: FiredOps;
}
impl<A: Planned> FiredTy for NFlags<A>
where
    (): FiredSized<{ NFlags::<A>::N }>,
{
    type T = <() as FiredSized<{ NFlags::<A>::N }>>::T;
}
/// Operations on the `fired` flags (of whichever size the pipeline needs).
#[doc(hidden)]
pub trait FiredOps {
    fn new() -> Self;
    fn set(&mut self, flag: usize);
    fn get(&self, flag: usize) -> bool;
}
impl<const N: usize> FiredOps for Fired<N> {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn new() -> Self {
        Fired { bits: [false; N] }
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn set(&mut self, flag: usize) {
        self.bits[flag] = true;
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn get(&self, flag: usize) -> bool {
        self.bits[flag]
    }
}
/// The `fired` flags of pipeline `A`.
#[doc(hidden)]
pub type FiredOf<A> = <A as Layout>::Fired;

/// The state of one push.
#[doc(hidden)]
pub struct PushCtx<'c, A: Layout> {
    pub(crate) root: &'c A::Root,
    pub(crate) states: &'c mut A::States,
    pub(crate) outs: &'c mut A::Outs,
    pub(crate) fired: FiredOf<A>,
}

/// The phase marker of a push. The traits that select an impl of `A` by its
/// bounds alone (`RunAt`, `RunIfAt`, `EntryOf`) take it as a parameter: a bound
/// `A: RunAt<Ph, X>` on a generic `Ph` then can't match the blanket impl, which
/// is only for `PushPh<A>`, and the solver takes it from the where clause
/// instead of trying to prove it (which recurses without end on an unevaluated
/// constant).
#[doc(hidden)]
pub struct PushPh<A>(PhantomData<A>);

/// The body of running one component (shared by `RunAt::run` and the guarded
/// `RunIfAt::run_if`; a macro, not a fn, so it costs no mono item per component).
macro_rules! run_body {
    ($A:ident, $P:ident, $ctx:ident) => {{
        type C<A, const P: usize> = <A as NthT<P>>::T;
        // The inputs borrow the stored outputs and the root only while
        // `update` runs; the write to the output comes after.
        let out = {
            let inputs = <<C<$A, $P> as ComponentSubscription>::Deps as GatherIn<$A>>::gather($ctx.root, &*$ctx.outs);
            <$A::States as AtPos<C<$A, $P>, $P>>::at_mut(&mut *$ctx.states).update(<C<$A, $P> as ComponentSubscription>::input(inputs))
        };
        *<$A::Outs as AtPos<<C<$A, $P> as Node>::Output, $P>>::at_mut(&mut *$ctx.outs) = out;
        if CondAt::<$A, $P>::FLAG != NO_FLAG {
            $ctx.fired.set(CondAt::<$A, $P>::FLAG);
        }
    }};
}

/// Run the component at component position `P` unconditionally: gather its
/// inputs, update it, store its output, and record that it fired if a guarded
/// component reads that.
#[doc(hidden)]
pub trait RunAt<Ph, const P: usize>: Layout {
    fn run(ctx: &mut PushCtx<'_, Self>);
}
impl<A: Layout, const P: usize> RunAt<PushPh<A>, P> for A
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: GatherIn<A>,
    A::States: AtPos<<A as NthT<P>>::T, P>,
    A::Outs: AtPos<<<A as NthT<P>>::T as Node>::Output, P>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn run(ctx: &mut PushCtx<'_, A>) {
        run_body!(A, P, ctx);
    }
}

/// Run the component at `P` if one of its edges fired (the flag-based guard).
#[doc(hidden)]
pub trait RunIfAt<Ph, const P: usize>: Layout {
    fn run_if(ctx: &mut PushCtx<'_, Self>);
}
impl<A: Layout, const P: usize> RunIfAt<PushPh<A>, P> for A
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: GatherIn<A> + Ready<A>,
    A::States: AtPos<<A as NthT<P>>::T, P>,
    A::Outs: AtPos<<<A as NthT<P>>::T as Node>::Output, P>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn run_if(ctx: &mut PushCtx<'_, A>) {
        if <<<A as NthT<P>>::T as ComponentSubscription>::Deps as Ready<A>>::ready(ctx.root, &*ctx.outs, &ctx.fired) {
            run_body!(A, P, ctx);
        }
    }
}

/// The flag of the component at component position `P`, from the plan.
#[doc(hidden)]
pub struct CondAt<A, const P: usize>(PhantomData<A>);
impl<A: Planned, const P: usize> CondAt<A, P> {
    pub const FLAG: usize = A::PLAN.flag_idx(Loc::<A, P>::IDX);
}

// ---------------------------------------------------------------------------
// The push program: a tree of nodes from the plan
// ---------------------------------------------------------------------------

/// Node `N` of the push program of `A` (see `Plan::build_program`).
#[doc(hidden)]
pub struct Pn<A, const N: usize>(PhantomData<A>);
/// The plan's facts about program node `N`.
#[doc(hidden)]
pub struct PnC<A, const N: usize>(PhantomData<A>);
impl<A: Planned, const N: usize> PnC<A, N> {
    pub const KIND: u8 = A::PLAN.pg_kind_of(N);
    pub const A0: usize = A::PLAN.pg_a_of(N);
    pub const SP: usize = A::PLAN.sw_pos(N);
    pub const SI: usize = A::PLAN.sw_entry(N);
    pub const SJ: usize = A::PLAN.sw_depth(N);
}
/// The root node of the program of `A`.
#[doc(hidden)]
pub struct RootProg<A>(PhantomData<A>);
impl<A: Planned> RootProg<A> {
    pub const N: usize = A::PLAN.root_prog();
}

/// The `K`-th child entry of the range `Q`: the node and the key.
#[doc(hidden)]
pub struct ChOf<A, Q, const K: usize>(PhantomData<(A, Q)>);
impl<A: Planned, Q: Rg, const K: usize> ChOf<A, Q, K> {
    pub const NODE: usize = A::PLAN.ch_node_of(Q::LO + K);
    pub const KEY: u32 = A::PLAN.ch_key_of(Q::LO + K);
}
/// The key at the middle of the range `Q`.
#[doc(hidden)]
pub struct MidKey<A, Q>(PhantomData<(A, Q)>);
impl<A: Planned, Q: Rg> MidKey<A, Q> {
    pub const KEY: u32 = A::PLAN.ch_key_of((Q::LO + Q::HI) / 2);
}
/// The left and right halves of a range.
#[doc(hidden)]
pub struct LeftOf<Q>(PhantomData<Q>);
#[doc(hidden)]
pub struct RightOf<Q>(PhantomData<Q>);
impl<Q: Rg> Rg for LeftOf<Q> {
    const LO: usize = Q::LO;
    const HI: usize = (Q::LO + Q::HI) / 2;
}
impl<Q: Rg> Rg for RightOf<Q> {
    const LO: usize = (Q::LO + Q::HI) / 2;
    const HI: usize = Q::HI;
}

/// Run program node `Self`.
#[doc(hidden)]
pub trait Exec<A: Layout, Ph> {
    fn exec(ctx: &mut PushCtx<'_, A>);
}

#[doc(hidden)]
pub struct SeqK;
#[doc(hidden)]
pub struct RunK;
#[doc(hidden)]
pub struct RunIfK;
#[doc(hidden)]
pub struct SwitchK;
#[doc(hidden)]
pub struct HRunK;
#[doc(hidden)]
pub trait PgKindOf<const K: u8> {
    type Ty;
}
impl PgKindOf<0> for () {
    type Ty = SeqK;
}
impl PgKindOf<1> for () {
    type Ty = RunK;
}
impl PgKindOf<2> for () {
    type Ty = RunIfK;
}
impl PgKindOf<3> for () {
    type Ty = SwitchK;
}
impl PgKindOf<4> for () {
    type Ty = HRunK;
}
#[doc(hidden)]
pub trait ExecK<A: Layout, Ph, K> {
    fn exec_k(ctx: &mut PushCtx<'_, A>);
}
#[doc(hidden)]
pub trait PnKind {
    type Ty;
}
impl<A: Planned, const N: usize> PnKind for Pn<A, N>
where
    (): PgKindOf<{ PnC::<A, N>::KIND }>,
{
    type Ty = <() as PgKindOf<{ PnC::<A, N>::KIND }>>::Ty;
}
impl<A: Layout, Ph, const N: usize> Exec<A, Ph> for Pn<A, N>
where
    Pn<A, N>: PnKind,
    Pn<A, N>: ExecK<A, Ph, <Pn<A, N> as PnKind>::Ty>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec(ctx: &mut PushCtx<'_, A>) {
        <Pn<A, N> as ExecK<A, Ph, <Pn<A, N> as PnKind>::Ty>>::exec_k(ctx)
    }
}

impl<A: Layout, Ph, const N: usize> ExecK<A, Ph, RunK> for Pn<A, N>
where
    A: RunAt<Ph, { PnC::<A, N>::A0 }>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec_k(ctx: &mut PushCtx<'_, A>) {
        <A as RunAt<Ph, { PnC::<A, N>::A0 }>>::run(ctx)
    }
}
impl<A: Layout, Ph, const N: usize> ExecK<A, Ph, RunIfK> for Pn<A, N>
where
    A: RunIfAt<Ph, { PnC::<A, N>::A0 }>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec_k(ctx: &mut PushCtx<'_, A>) {
        <A as RunIfAt<Ph, { PnC::<A, N>::A0 }>>::run_if(ctx)
    }
}

// -- Seq: the children in order, over the balanced ranges of the child array --

/// The range of child entries of node `N` (a Seq or a Switch).
#[doc(hidden)]
pub struct NodeRg<A, const N: usize>(PhantomData<A>);
impl<A: Planned, const N: usize> Rg for NodeRg<A, N> {
    const LO: usize = A::PLAN.pg_a_of(N);
    const HI: usize = A::PLAN.pg_a_of(N) + A::PLAN.pg_b_of(N);
}
#[doc(hidden)]
pub trait SeqDrive<A: Layout, Ph, Cl>: Rg {
    fn seq(ctx: &mut PushCtx<'_, A>);
}
macro_rules! seq_leaf {
    ($n:literal; $($k:literal)*) => {
        impl<A: Layout, Ph, Q: Rg> SeqDrive<A, Ph, Leaf<$n>> for Q
        where
            $(Pn<A, { ChOf::<A, Q, $k>::NODE }>: Exec<A, Ph>,)*
        {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            #[allow(unused_variables)]
            fn seq(ctx: &mut PushCtx<'_, A>) {
                $(<Pn<A, { ChOf::<A, Q, $k>::NODE }> as Exec<A, Ph>>::exec(ctx);)*
            }
        }
    };
}
dataflow_macros::leaf_idx!(seq_leaf);
macro_rules! seq_inner {
    ($($k:literal)*) => {
        impl<A: Layout, Ph, Q: Rg> SeqDrive<A, Ph, Inner> for Q
        where
            $(Ch<Q, $k>: RgCl,
              Ch<Q, $k>: SeqDrive<A, Ph, <Ch<Q, $k> as RgCl>::Cl>,)*
        {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            fn seq(ctx: &mut PushCtx<'_, A>) {
                $(<Ch<Q, $k> as SeqDrive<A, Ph, <Ch<Q, $k> as RgCl>::Cl>>::seq(ctx);)*
            }
        }
    };
}
dataflow_macros::inner_idx!(seq_inner);

impl<A: Layout, Ph, const N: usize> ExecK<A, Ph, SeqK> for Pn<A, N>
where
    NodeRg<A, N>: RgCl,
    NodeRg<A, N>: SeqDrive<A, Ph, <NodeRg<A, N> as RgCl>::Cl>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec_k(ctx: &mut PushCtx<'_, A>) {
        <NodeRg<A, N> as SeqDrive<A, Ph, <NodeRg<A, N> as RgCl>::Cl>>::seq(ctx)
    }
}

// -- Switch: the key of the value once, then a decision tree over the sorted keys --

/// The entry `I` of component position `P`: its node and selector chain.
#[doc(hidden)]
pub trait EntryOf<Ph, const P: usize, const I: usize>: Layout {
    type E: Node;
    type Ch;
}
impl<A: Layout, const P: usize, const I: usize> EntryOf<PushPh<A>, P, I> for A
where
    A: NthT<P>,
    <A as NthT<P>>::T: ComponentSubscription,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps:
        PosSelect<EntryPos<I>> + PickEdge<EntryPos<I>, { <<<A as NthT<P>>::T as ComponentSubscription>::Deps as PosSelect<EntryPos<I>>>::POS }>,
{
    type E = <<<A as NthT<P>>::T as ComponentSubscription>::Deps as PickEdge<
        EntryPos<I>,
        { <<<A as NthT<P>>::T as ComponentSubscription>::Deps as PosSelect<EntryPos<I>>>::POS },
    >>::E;
    type Ch = <<<A as NthT<P>>::T as ComponentSubscription>::Deps as PickEdge<
        EntryPos<I>,
        { <<<A as NthT<P>>::T as ComponentSubscription>::Deps as PosSelect<EntryPos<I>>>::POS },
    >>::Ch;
}

/// The key the value at a switch has.
#[doc(hidden)]
pub trait SwitchKey<A: Layout, Ph> {
    fn key(root: &A::Root, outs: &A::Outs) -> u32;
}
#[doc(hidden)]
pub trait SwInfo<A: Layout, Ph> {
    type E: Node;
    type Ch;
}
impl<A: Layout, Ph, const N: usize> SwInfo<A, Ph> for Pn<A, N>
where
    A: EntryOf<Ph, { PnC::<A, N>::SP }, { PnC::<A, N>::SI }>,
{
    type E = <A as EntryOf<Ph, { PnC::<A, N>::SP }, { PnC::<A, N>::SI }>>::E;
    type Ch = <A as EntryOf<Ph, { PnC::<A, N>::SP }, { PnC::<A, N>::SI }>>::Ch;
}
impl<A: Layout, Ph, const N: usize> SwitchKey<A, Ph> for Pn<A, N>
where
    Pn<A, N>: SwInfo<A, Ph>,
    <Pn<A, N> as SwInfo<A, Ph>>::E: Source<A>,
    <Pn<A, N> as SwInfo<A, Ph>>::Ch: ChainKey<<<Pn<A, N> as SwInfo<A, Ph>>::E as Node>::Output, { PnC::<A, N>::SJ }>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key(root: &A::Root, outs: &A::Outs) -> u32 {
        <<Pn<A, N> as SwInfo<A, Ph>>::Ch as ChainKey<<<Pn<A, N> as SwInfo<A, Ph>>::E as Node>::Output, { PnC::<A, N>::SJ }>>::key(
            <<Pn<A, N> as SwInfo<A, Ph>>::E as Source<A>>::fetch(root, outs),
        )
    }
}

/// The decision tree over the sorted keys of the range `Q`.
#[doc(hidden)]
pub trait BstExec<A: Layout, Ph, Cl>: Rg {
    fn bst(ctx: &mut PushCtx<'_, A>, key: u32) -> bool;
}
macro_rules! bst_leaf {
    ($n:literal; $($k:literal)*) => {
        impl<A: Layout, Ph, Q: Rg> BstExec<A, Ph, Leaf<$n>> for Q
        where
            $(Pn<A, { ChOf::<A, Q, $k>::NODE }>: Exec<A, Ph>,)*
        {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            #[allow(unused_variables)]
            fn bst(ctx: &mut PushCtx<'_, A>, key: u32) -> bool {
                $(if key == ChOf::<A, Q, $k>::KEY {
                    <Pn<A, { ChOf::<A, Q, $k>::NODE }> as Exec<A, Ph>>::exec(ctx);
                    return true;
                })*
                false
            }
        }
    };
}
dataflow_macros::leaf_idx!(bst_leaf);
impl<A: Layout, Ph, Q: Rg> BstExec<A, Ph, Inner> for Q
where
    LeftOf<Q>: RgCl,
    RightOf<Q>: RgCl,
    LeftOf<Q>: BstExec<A, Ph, <LeftOf<Q> as RgCl>::Cl>,
    RightOf<Q>: BstExec<A, Ph, <RightOf<Q> as RgCl>::Cl>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn bst(ctx: &mut PushCtx<'_, A>, key: u32) -> bool {
        <LeftOf<Q> as BstExec<A, Ph, <LeftOf<Q> as RgCl>::Cl>>::bst(ctx, key)
            || <RightOf<Q> as BstExec<A, Ph, <RightOf<Q> as RgCl>::Cl>>::bst(ctx, key)
    }
}

impl<A: Layout, Ph, const N: usize> ExecK<A, Ph, SwitchK> for Pn<A, N>
where
    Pn<A, N>: SwitchKey<A, Ph>,
    NodeRg<A, N>: RgCl,
    NodeRg<A, N>: BstExec<A, Ph, <NodeRg<A, N> as RgCl>::Cl>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec_k(ctx: &mut PushCtx<'_, A>) {
        let key = <Pn<A, N> as SwitchKey<A, Ph>>::key(ctx.root, &*ctx.outs);
        <NodeRg<A, N> as BstExec<A, Ph, <NodeRg<A, N> as RgCl>::Cl>>::bst(ctx, key);
    }
}
