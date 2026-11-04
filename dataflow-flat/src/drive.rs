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
//! unconditionally), `RunIf` (guarded by its condition, asked again at run time from the
//! pushed root and the stored outputs, see `guard`; only for conditions the trie cannot express,
//! such as guards) and `Switch` (the key of
//! the alternative a value is, from `Select::key_of`, evaluated once; then a
//! decision tree over the sorted constant keys of the sibling decisions, which
//! LLVM lowers to a jump table). Leaves run their cones straight-line; a
//! component joined across exclusive terms is repeated in each leaf. Order is
//! kept: every list is in execution order and a switch covers only a run of
//! consecutive components.

use std::any::type_name;
use std::marker::PhantomData;

use crate::build::ConstructPass;
use crate::component::{Component, ComponentSubscription, DepList};
use crate::flat::{bound, class, ClassOf, EntryPos, Inner, Leaf, NthS, NthT, PickEdge, PosSelect, Whole};
use crate::gather::{ChainKeyAt, ChainKeyPre, GatherCx, GatherIn, GatherInit, Layout, Source};
use crate::guard::{Gd, Holds};
use crate::split::SplitAt;
use crate::select::{ChainInitial, Keep, SelectChain};
use crate::node::{Comp, SlotNode, Vertex};
use crate::plan::Planned;
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
    /// Whether the subtrees of the driver are functions of their own (phases that run once, at
    /// construction: LLVM then never sees one function the size of the whole graph).
    const OUTLINE: bool = false;
}

/// A subtree of the driver as a function of its own.
#[doc(hidden)]
#[cfg_attr(not(debug_assertions), inline(never))]
#[cfg_attr(debug_assertions, inline)]
pub fn drive_out<A: Planned, Ph: Phase, Cl, Q: Drive<A, Ph, Cl>>(ctx: &mut Ph::Ctx<'_>) {
    Q::drive(ctx)
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
                $(if Ph::OUTLINE {
                    drive_out::<A, Ph, <Ch<Q, $k> as RgCl>::Cl, Ch<Q, $k>>(ctx);
                } else {
                    <Ch<Q, $k> as Drive<A, Ph, <Ch<Q, $k> as RgCl>::Cl>>::drive(ctx);
                })*
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
    const OUTLINE: bool = true;
}

impl<A: Layout, const P: usize> StepAt<ConstructPh<A>, P> for A
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: GatherInit<A, P>,
    A::StatesOpt: AtPos<Option<<A as NthT<P>>::T>, P>,
    A::OutsOpt: AtPos<Option<<<A as NthT<P>>::T as Vertex>::Stored>, P>,
{
    fn step(ctx: &mut ConstructCtx<'_, A>) {
        type C<A, const P: usize> = <A as NthT<P>>::T;
        let config = ctx.pass.take_config::<C<A, P>>();
        let (state, out) = {
            let raw = <<C<A, P> as ComponentSubscription>::Deps as GatherInit<A, P>>::gather(&*ctx.states, &*ctx.outs);
            let mut init = <C<A, P> as ComponentSubscription>::init(raw);
            <C<A, P> as ComponentSubscription>::__fill_requests(&mut init, &mut *ctx.pass);
            <C<A, P> as Component>::new(config, init)
        };
        // A handle component that did not write `fn view` fails here (post-monomorphization) even
        // if nobody reads it; the branch is a constant, so by-value components keep nothing of it.
        if <C<A, P> as Vertex>::VIEW {
            let _ = <C<A, P> as Component>::view(&state, &out);
        }
        *<A::StatesOpt as AtPos<Option<C<A, P>>, P>>::at_mut(&mut *ctx.states) = Some(state);
        *<A::OutsOpt as AtPos<Option<<C<A, P> as Vertex>::Stored>, P>>::at_mut(&mut *ctx.outs) = Some(out);
        ctx.pass.order.push(type_name::<C<A, P>>());
    }
}

// ---------------------------------------------------------------------------
// Push
// ---------------------------------------------------------------------------

/// The state of one push.
#[doc(hidden)]
pub struct PushCtx<'c, A: Layout> {
    pub(crate) root: &'c A::Root,
    pub(crate) states: &'c mut A::States,
    pub(crate) outs: &'c mut A::Outs,
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
/// `RunIfAt::run_if`; a macro, not a fn, so it costs no mono item per component). `$upd` is
/// `upd_val` (the inputs are stored outputs and the root) or `upd_view` (a parent is a handle
/// component): the plan picks the program node kind, so a by-value component's bounds don't
/// mention the view machinery at all.
macro_rules! run_body {
    ($upd:ident, $A:ident, $P:ident, $ctx:ident) => {{
        run_body_pre!($upd, $A, $P, $ctx, |_o| ());
    }};
}
/// `run_body` that also evaluates `$pre` on the value `update` returned, before it is
/// stored, and gives the result back (the fused run and switch takes the key here).
macro_rules! run_body_pre {
    ($upd:ident, $A:ident, $P:tt, $ctx:ident, |$o:ident| $pre:expr) => {{
        type C<A, const P: usize> = <A as NthT<P>>::T;
        // The inputs borrow the stored outputs and the root only while
        // `update` runs; the write to the output comes after.
        let out = $upd!($A, $P, $ctx);
        let pre = {
            let $o = &out;
            $pre
        };
        *<$A::Outs as AtPos<<C<$A, $P> as Vertex>::Stored, $P>>::at_mut(&mut *$ctx.outs) = out;
        // Stored selected values whose source is this component: written when their
        // selectors match, before any subscriber of this step reads them.
        <SrcSlots<$A, $P> as Drive<$A, SlotWriteBy<$A>, <SrcSlots<$A, $P> as RgCl>::Cl>>::drive($ctx);
        pre
    }};
}
/// Gather the inputs of the component at `P` from stored outputs and the root, and call `update`.
macro_rules! upd_val {
    ($A:ident, $P:tt, $ctx:ident) => {{
        type C<A, const P: usize> = <A as NthT<P>>::T;
        let previous = <C<$A, $P> as Component>::take_output(
            <$A::Outs as AtPos<<C<$A, $P> as Vertex>::Stored, $P>>::at_mut(&mut *$ctx.outs));
        let inputs = <<C<$A, $P> as ComponentSubscription>::Deps as GatherIn<$A, $P>>::gather($ctx.root, &*$ctx.outs);
        <$A::States as AtPos<C<$A, $P>, $P>>::at_mut(&mut *$ctx.states).update_recycled(<C<$A, $P> as ComponentSubscription>::input(inputs), previous)
    }};
}
/// As `upd_val`, for a component with a handle parent: its own state is split off from the states
/// before it (`split.rs`), and the view of the parent is computed from those.
macro_rules! upd_view {
    ($A:ident, $P:tt, $ctx:ident) => {{
        type C<A, const P: usize> = <A as NthT<P>>::T;
        let (cx, me) = <$A::States as SplitAt<$P>>::split_at(&mut *$ctx.states);
        let previous = <C<$A, $P> as Component>::take_output(
            <$A::Outs as AtPos<<C<$A, $P> as Vertex>::Stored, $P>>::at_mut(&mut *$ctx.outs));
        let inputs = <<C<$A, $P> as ComponentSubscription>::Deps as GatherCx<$A, $P>>::gather($ctx.root, &*$ctx.outs, &cx);
        <C<$A, $P> as Component>::update_recycled(me, <C<$A, $P> as ComponentSubscription>::input(inputs), previous)
    }};
}

/// Run the component at component position `P` unconditionally: gather its
/// inputs, update it and store its output.
#[doc(hidden)]
pub trait RunAt<Ph, const P: usize>: Layout {
    fn run(ctx: &mut PushCtx<'_, Self>);
}
impl<A: Layout, const P: usize> RunAt<PushPh<A>, P> for A
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: GatherIn<A, P>,
    A::States: AtPos<<A as NthT<P>>::T, P>,
    A::Outs: AtPos<<<A as NthT<P>>::T as Vertex>::Stored, P>,
    SrcSlots<A, P>: RgCl,
    SrcSlots<A, P>: Drive<A, SlotWriteBy<A>, <SrcSlots<A, P> as RgCl>::Cl>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn run(ctx: &mut PushCtx<'_, A>) {
        if WIDE_RUN::<A, P>() {
            run_wide::<A, P>(ctx);
        } else {
            run_body!(upd_val, A, P, ctx);
        }
    }
}
/// `RunAt` for a component with a handle parent.
#[doc(hidden)]
pub trait RunViewAt<Ph, const P: usize>: Layout {
    fn run_view(ctx: &mut PushCtx<'_, Self>);
}
impl<A: Layout, const P: usize> RunViewAt<PushPh<A>, P> for A
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: GatherCx<A, P>,
    A::States: SplitAt<P, Child = <A as NthT<P>>::T>,
    A::Outs: AtPos<<<A as NthT<P>>::T as Vertex>::Stored, P>,
    SrcSlots<A, P>: RgCl,
    SrcSlots<A, P>: Drive<A, SlotWriteBy<A>, <SrcSlots<A, P> as RgCl>::Cl>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn run_view(ctx: &mut PushCtx<'_, A>) {
        if WIDE_RUN::<A, P>() {
            run_wide_view::<A, P>(ctx);
        } else {
            run_body!(upd_view, A, P, ctx);
        }
    }
}

/// Components with at least this many subscriptions run through a function of their own in
/// release builds: their gather is as long as their inputs, and inlined into every switch arm and
/// tail that runs them the push grows with the square of the graph.
#[doc(hidden)]
pub const WIDE_INPUTS: usize = 24;
#[doc(hidden)]
#[allow(non_snake_case)]
#[cfg_attr(not(debug_assertions), inline(always))]
#[cfg_attr(debug_assertions, inline)]
pub fn WIDE_RUN<A: Layout + NthT<P>, const P: usize>() -> bool
where
    <A as NthT<P>>::T: ComponentSubscription,
{
    !cfg!(debug_assertions) && <<<A as NthT<P>>::T as ComponentSubscription>::Deps as DepList>::LEN >= WIDE_INPUTS
}
#[doc(hidden)]
#[cfg_attr(not(debug_assertions), inline(never))]
#[cfg_attr(debug_assertions, inline)]
pub fn run_wide<A: Layout, const P: usize>(ctx: &mut PushCtx<'_, A>)
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: GatherIn<A, P>,
    A::States: AtPos<<A as NthT<P>>::T, P>,
    A::Outs: AtPos<<<A as NthT<P>>::T as Vertex>::Stored, P>,
    SrcSlots<A, P>: RgCl,
    SrcSlots<A, P>: Drive<A, SlotWriteBy<A>, <SrcSlots<A, P> as RgCl>::Cl>,
{
    run_body!(upd_val, A, P, ctx);
}
#[doc(hidden)]
#[cfg_attr(not(debug_assertions), inline(never))]
#[cfg_attr(debug_assertions, inline)]
pub fn run_wide_view<A: Layout, const P: usize>(ctx: &mut PushCtx<'_, A>)
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: GatherCx<A, P>,
    A::States: SplitAt<P, Child = <A as NthT<P>>::T>,
    A::Outs: AtPos<<<A as NthT<P>>::T as Vertex>::Stored, P>,
    SrcSlots<A, P>: RgCl,
    SrcSlots<A, P>: Drive<A, SlotWriteBy<A>, <SrcSlots<A, P> as RgCl>::Cl>,
{
    run_body!(upd_view, A, P, ctx);
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
    pub const B0: usize = A::PLAN.pg_b_of(N);
    pub const IFG: usize = A::PLAN.if_guard(N);
    pub const SP: usize = A::PLAN.sw_pos(N);
    pub const SI: usize = A::PLAN.sw_entry(N);
    pub const SJ: usize = A::PLAN.sw_depth(N);
    pub const HAS_SW: bool = A::PLAN.prog_has_switch(N);
    pub const VIEW_LEAF: bool = A::PLAN.prog_view_leaf(N);
    pub const TINY_ARM: bool = A::PLAN.prog_tiny_arm(N);
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
    fn exec(ctx: &mut PushCtx<'_, A>) -> bool;
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
pub struct RunSwK;
impl PgKindOf<5> for () {
    type Ty = RunSwK;
}
#[doc(hidden)]
pub struct TailK;
impl PgKindOf<6> for () {
    type Ty = TailK;
}
#[doc(hidden)]
pub struct RunViewK;
impl PgKindOf<7> for () {
    type Ty = RunViewK;
}
#[doc(hidden)]
pub struct RunIfViewK;
impl PgKindOf<8> for () {
    type Ty = RunIfViewK;
}
#[doc(hidden)]
pub struct IfK;
#[doc(hidden)]
pub struct GateK;
impl PgKindOf<10> for () {
    type Ty = GateK;
}
impl PgKindOf<0> for () {
    type Ty = SeqK;
}
impl PgKindOf<9> for () {
    type Ty = IfK;
}
#[doc(hidden)]
/// Run a node of the kind `K`. The result says whether the list the node is in goes on: false only
/// for a gate (`GateK`) whose guard does not hold.
pub trait ExecK<A: Layout, Ph, K> {
    fn exec_k(ctx: &mut PushCtx<'_, A>) -> bool;
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
    fn exec(ctx: &mut PushCtx<'_, A>) -> bool {
        <Pn<A, N> as ExecK<A, Ph, <Pn<A, N> as PnKind>::Ty>>::exec_k(ctx)
    }
}

impl<A: Layout, Ph, const N: usize> ExecK<A, Ph, RunK> for Pn<A, N>
where
    A: RunAt<Ph, { PnC::<A, N>::A0 }>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec_k(ctx: &mut PushCtx<'_, A>) -> bool {
        <A as RunAt<Ph, { PnC::<A, N>::A0 }>>::run(ctx);
        true
    }
}
/// A guarded run: the component runs if its guard holds (its condition, asked again from the pushed
/// root and the stored outputs: see `guard`).
impl<A: Layout, Ph, const N: usize> ExecK<A, Ph, RunIfK> for Pn<A, N>
where
    A: RunAt<Ph, { PnC::<A, N>::A0 }>,
    Gd<A, Ph, { PnC::<A, N>::B0 }>: Holds<A, Ph>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec_k(ctx: &mut PushCtx<'_, A>) -> bool {
        if <Gd<A, Ph, { PnC::<A, N>::B0 }> as Holds<A, Ph>>::holds(ctx.root, &*ctx.states, &*ctx.outs) {
            <A as RunAt<Ph, { PnC::<A, N>::A0 }>>::run(ctx)
        }
        true
    }
}

/// A gate: the rest of the list it is in runs only if its guard decision holds (asked once, here).
impl<A: Layout, Ph, const N: usize> ExecK<A, Ph, GateK> for Pn<A, N>
where
    Gd<A, Ph, { PnC::<A, N>::IFG }>: Holds<A, Ph>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec_k(ctx: &mut PushCtx<'_, A>) -> bool {
        <Gd<A, Ph, { PnC::<A, N>::IFG }> as Holds<A, Ph>>::holds(ctx.root, &*ctx.states, &*ctx.outs)
    }
}

/// A block: its children run if its guard decision holds (asked once, here).
impl<A: Layout, Ph, const N: usize> ExecK<A, Ph, IfK> for Pn<A, N>
where
    NodeRg<A, N>: RgCl,
    NodeRg<A, N>: SeqDrive<A, Ph, <NodeRg<A, N> as RgCl>::Cl>,
    Gd<A, Ph, { PnC::<A, N>::IFG }>: Holds<A, Ph>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec_k(ctx: &mut PushCtx<'_, A>) -> bool {
        if <Gd<A, Ph, { PnC::<A, N>::IFG }> as Holds<A, Ph>>::holds(ctx.root, &*ctx.states, &*ctx.outs) {
            let _ = <NodeRg<A, N> as SeqDrive<A, Ph, <NodeRg<A, N> as RgCl>::Cl>>::seq(ctx);
        }
        true
    }
}

impl<A: Layout, Ph, const N: usize> ExecK<A, Ph, RunViewK> for Pn<A, N>
where
    A: RunViewAt<Ph, { PnC::<A, N>::A0 }>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec_k(ctx: &mut PushCtx<'_, A>) -> bool {
        <A as RunViewAt<Ph, { PnC::<A, N>::A0 }>>::run_view(ctx);
        true
    }
}
impl<A: Layout, Ph, const N: usize> ExecK<A, Ph, RunIfViewK> for Pn<A, N>
where
    A: RunViewAt<Ph, { PnC::<A, N>::A0 }>,
    Gd<A, Ph, { PnC::<A, N>::B0 }>: Holds<A, Ph>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec_k(ctx: &mut PushCtx<'_, A>) -> bool {
        if <Gd<A, Ph, { PnC::<A, N>::B0 }> as Holds<A, Ph>>::holds(ctx.root, &*ctx.states, &*ctx.outs) {
            <A as RunViewAt<Ph, { PnC::<A, N>::A0 }>>::run_view(ctx)
        }
        true
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
    /// Runs the list in order; false if a gate stopped it.
    fn seq(ctx: &mut PushCtx<'_, A>) -> bool;
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
            fn seq(ctx: &mut PushCtx<'_, A>) -> bool {
                true $(&& <Pn<A, { ChOf::<A, Q, $k>::NODE }> as Exec<A, Ph>>::exec(ctx))*
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
            fn seq(ctx: &mut PushCtx<'_, A>) -> bool {
                true $(&& <Ch<Q, $k> as SeqDrive<A, Ph, <Ch<Q, $k> as RgCl>::Cl>>::seq(ctx))*
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
    fn exec_k(ctx: &mut PushCtx<'_, A>) -> bool {
        let _ = <NodeRg<A, N> as SeqDrive<A, Ph, <NodeRg<A, N> as RgCl>::Cl>>::seq(ctx);
        true
    }
}

/// The runs the branches of a switch share (`PG_TAIL`), as a function of their own in release builds.
#[doc(hidden)]
#[cfg_attr(not(debug_assertions), inline(never))]
#[cfg_attr(debug_assertions, inline)]
pub fn exec_tail<A: Layout, Ph, const N: usize>(ctx: &mut PushCtx<'_, A>)
where
    NodeRg<A, N>: RgCl,
    NodeRg<A, N>: SeqDrive<A, Ph, <NodeRg<A, N> as RgCl>::Cl>,
{
    let _ = <NodeRg<A, N> as SeqDrive<A, Ph, <NodeRg<A, N> as RgCl>::Cl>>::seq(ctx);
}
impl<A: Layout, Ph, const N: usize> ExecK<A, Ph, TailK> for Pn<A, N>
where
    NodeRg<A, N>: RgCl,
    NodeRg<A, N>: SeqDrive<A, Ph, <NodeRg<A, N> as RgCl>::Cl>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec_k(ctx: &mut PushCtx<'_, A>) -> bool {
        exec_tail::<A, Ph, N>(ctx);
        true
    }
}

// -- Switch: the key of the value once, then a decision tree over the sorted keys --

/// The entry `I` of component position `P`: its node and selector chain.
#[doc(hidden)]
pub trait EntryOf<Ph, const P: usize, const I: usize>: Layout {
    type E: Vertex;
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
    fn key(root: &A::Root, states: &A::States, outs: &A::Outs) -> u32;
}
#[doc(hidden)]
pub trait SwInfo<A: Layout, Ph> {
    type E: Vertex;
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
    <Pn<A, N> as SwInfo<A, Ph>>::Ch: ChainKeyAt<<Pn<A, N> as SwInfo<A, Ph>>::E, A, { PnC::<A, N>::SJ }>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key(root: &A::Root, states: &A::States, outs: &A::Outs) -> u32 {
        <<Pn<A, N> as SwInfo<A, Ph>>::Ch as ChainKeyAt<<Pn<A, N> as SwInfo<A, Ph>>::E, A, { PnC::<A, N>::SJ }>>::key_at(root, states, outs)
    }
}

/// The decision tree over the sorted keys of the range `Q`.
#[doc(hidden)]
pub trait BstExec<A: Layout, Ph, Cl>: Rg {
    fn bst(ctx: &mut PushCtx<'_, A>, key: u32) -> bool;
}
/// A switch arm that has no switch below it, as a function of its own in release builds: LLVM
/// then optimizes many small functions instead of one the size of the whole push. The cost is
/// one call per packet (the arms above it stay inline).
#[doc(hidden)]
#[cfg_attr(not(debug_assertions), inline(always))]
#[cfg_attr(debug_assertions, inline)]
pub fn exec_arm<A: Layout, Ph, const N: usize>(ctx: &mut PushCtx<'_, A>)
where
    Pn<A, N>: Exec<A, Ph>,
{
    if PnC::<A, N>::HAS_SW || PnC::<A, N>::VIEW_LEAF || PnC::<A, N>::TINY_ARM {
        let _ = <Pn<A, N> as Exec<A, Ph>>::exec(ctx);
    } else {
        exec_out::<A, Ph, N>(ctx)
    }
}
#[doc(hidden)]
#[cfg_attr(not(debug_assertions), inline(never))]
#[cfg_attr(debug_assertions, inline)]
pub fn exec_out<A: Layout, Ph, const N: usize>(ctx: &mut PushCtx<'_, A>)
where
    Pn<A, N>: Exec<A, Ph>,
{
    let _ = <Pn<A, N> as Exec<A, Ph>>::exec(ctx);
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
                    exec_arm::<A, Ph, { ChOf::<A, Q, $k>::NODE }>(ctx);
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
    fn exec_k(ctx: &mut PushCtx<'_, A>) -> bool {
        let key = <Pn<A, N> as SwitchKey<A, Ph>>::key(ctx.root, &*ctx.states, &*ctx.outs);
        <NodeRg<A, N> as BstExec<A, Ph, <NodeRg<A, N> as RgCl>::Cl>>::bst(ctx, key);
        true
    }
}

/// The switch node `M` driven by a key taken from the source `T` (the component run just before
/// it) and the value `O` its `update` just returned: for a by-value source the key comes from
/// `o`, for a handle source from the view of its state `me` and the handle `o`.
#[doc(hidden)]
pub trait SwitchPre<A: Layout, Ph, T: Vertex<Stored = O>, O: 'static> {
    fn key(me: &T, o: &O) -> u32;
    fn bst(ctx: &mut PushCtx<'_, A>, key: u32);
}
impl<A: Layout, Ph, T: Vertex<Stored = O>, O: 'static, const M: usize> SwitchPre<A, Ph, T, O> for Pn<A, M>
where
    Pn<A, M>: SwInfo<A, Ph>,
    <Pn<A, M> as SwInfo<A, Ph>>::Ch: ChainKeyPre<T, { PnC::<A, M>::SJ }>,
    NodeRg<A, M>: RgCl,
    NodeRg<A, M>: BstExec<A, Ph, <NodeRg<A, M> as RgCl>::Cl>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key(me: &T, o: &O) -> u32 {
        <<Pn<A, M> as SwInfo<A, Ph>>::Ch as ChainKeyPre<T, { PnC::<A, M>::SJ }>>::key_pre(me, o)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn bst(ctx: &mut PushCtx<'_, A>, key: u32) {
        <NodeRg<A, M> as BstExec<A, Ph, <NodeRg<A, M> as RgCl>::Cl>>::bst(ctx, key);
    }
}

/// What a fused run does with the value `update` returned: take the switch key from it, and
/// afterwards run the decision tree.
#[doc(hidden)]
pub trait SwFn<A: Layout, T: Vertex<Stored = O>, O: 'static> {
    fn key(me: &T, o: &O) -> u32;
    fn bst(ctx: &mut PushCtx<'_, A>, key: u32);
}
#[doc(hidden)]
pub struct Fused<Ph, S>(PhantomData<(Ph, S)>);
impl<A: Layout, Ph, S: SwitchPre<A, Ph, T, O>, T: Vertex<Stored = O>, O: 'static> SwFn<A, T, O> for Fused<Ph, S> {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn key(me: &T, o: &O) -> u32 {
        S::key(me, o)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn bst(ctx: &mut PushCtx<'_, A>, key: u32) {
        S::bst(ctx, key)
    }
}

/// A run fused with the switch on its output: the key comes from the value `update`
/// returned, not from the stored copy, so the jump does not wait for the store.
#[doc(hidden)]
pub trait RunSwAt<Ph, const P: usize>: Layout {
    /// The output of the component at `P`.
    type Out: 'static;
    /// The component at `P`, whose `Stored` is `Out`.
    type Me: Vertex<Stored = Self::Out>;
    fn run_sw<F: SwFn<Self, Self::Me, Self::Out>>(ctx: &mut PushCtx<'_, Self>);
}
impl<A: Layout, const P: usize> RunSwAt<PushPh<A>, P> for A
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: GatherIn<A, P>,
    A::States: AtPos<<A as NthT<P>>::T, P>,
    A::Outs: AtPos<<<A as NthT<P>>::T as Vertex>::Stored, P>,
    A::Outs: AtPos<<<A as NthT<P>>::T as Component>::Handle, P>,
    SrcSlots<A, P>: RgCl,
    SrcSlots<A, P>: Drive<A, SlotWriteBy<A>, <SrcSlots<A, P> as RgCl>::Cl>,
{
    type Out = <<A as NthT<P>>::T as Component>::Handle;
    type Me = <A as NthT<P>>::T;
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn run_sw<F: SwFn<A, Self::Me, Self::Out>>(ctx: &mut PushCtx<'_, A>) {
        let key = run_body_pre!(upd_val, A, P, ctx, |o| F::key(<A::States as AtPos<<A as NthT<P>>::T, P>>::at(&*ctx.states), o));
        F::bst(ctx, key);
    }
}
/// Switch node `Self` fused after the run of the component at `P`.
#[doc(hidden)]
pub trait FuLink<A: Layout, Ph, const P: usize> {
    fn go(ctx: &mut PushCtx<'_, A>);
}
impl<A: Layout, Ph, const P: usize, const M: usize> FuLink<A, Ph, P> for Pn<A, M>
where
    A: RunSwAt<Ph, P>,
    Pn<A, M>: SwitchPre<A, Ph, <A as RunSwAt<Ph, P>>::Me, <A as RunSwAt<Ph, P>>::Out>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn go(ctx: &mut PushCtx<'_, A>) {
        <A as RunSwAt<Ph, P>>::run_sw::<Fused<Ph, Pn<A, M>>>(ctx)
    }
}
impl<A: Layout, Ph, const N: usize> ExecK<A, Ph, RunSwK> for Pn<A, N>
where
    Pn<A, { PnC::<A, N>::B0 }>: FuLink<A, Ph, { PnC::<A, N>::A0 }>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn exec_k(ctx: &mut PushCtx<'_, A>) -> bool {
        <Pn<A, { PnC::<A, N>::B0 }> as FuLink<A, Ph, { PnC::<A, N>::A0 }>>::go(ctx);
        true
    }
}

// ---------------------------------------------------------------------------
// Stored selected values (slots)
// ---------------------------------------------------------------------------
//
// A slot has a storage position after the components (`flat::SlotTy`). It is
// initialised before any component is constructed (`SlotInitPh`), so that `new`
// can read it, and written in a push right after its source has run (`SlotWriteBy`,
// from the run of the source component, or at the start of the push for the root),
// when the chain matches the source's new output.

/// The slots of the root, as a range of storage positions.
#[doc(hidden)]
pub struct RootSlots<A>(PhantomData<A>);
impl<A: Planned> Rg for RootSlots<A> {
    const LO: usize = A::PLAN.root_slots_lo();
    const HI: usize = A::PLAN.root_slots_hi();
}
/// The slots whose source is the component at position `P`.
#[doc(hidden)]
pub struct SrcSlots<A, const P: usize>(PhantomData<A>);
impl<A: Planned, const P: usize> Rg for SrcSlots<A, P> {
    const LO: usize = A::PLAN.comp_slots_lo(P);
    const HI: usize = A::PLAN.comp_slots_hi(P);
}
/// All the slots.
#[doc(hidden)]
pub struct AllSlots<A>(PhantomData<A>);
impl<A: Planned> Rg for AllSlots<A> {
    const LO: usize = Whole::<A>::N;
    const HI: usize = Whole::<A>::T;
}

/// Phase: write the slots of the range (during a push).
#[doc(hidden)]
pub struct SlotWriteBy<A>(PhantomData<A>);
impl<A: Layout> Phase for SlotWriteBy<A> {
    type Ctx<'c> = PushCtx<'c, A>;
}
/// Phase: give every slot its initial value (while the pipeline is built).
#[doc(hidden)]
pub struct SlotInitPh<A>(PhantomData<A>);
impl<A: Layout> Phase for SlotInitPh<A> {
    type Ctx<'c> = ConstructCtx<'c, A>;
    const OUTLINE: bool = true;
}

/// The work of a slot node at storage position `P`.
#[doc(hidden)]
pub trait SlotStep<A: Layout, const P: usize> {
    fn write(ctx: &mut PushCtx<'_, A>);
    fn init(ctx: &mut ConstructCtx<'_, A>);
}
impl<E, C, A, const P: usize> SlotStep<A, P> for SlotNode<E, C>
where
    A: Layout,
    E: Source<A>,
    C: SelectChain<<E as Vertex>::Stored> + ChainInitial<<E as Vertex>::Stored>,
    <C as SelectChain<<E as Vertex>::Stored>>::Out: Keep,
    A::Outs: AtPos<<C as SelectChain<<E as Vertex>::Stored>>::Out, P>,
    A::StatesOpt: AtPos<Option<SlotNode<E, C>>, P>,
    A::OutsOpt: AtPos<Option<<C as SelectChain<<E as Vertex>::Stored>>::Out>, P>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn write(ctx: &mut PushCtx<'_, A>) {
        let x = <E as Source<A>>::fetch(ctx.root, &*ctx.outs);
        if <C as SelectChain<<E as Vertex>::Stored>>::matches(x) {
            let v = <C as SelectChain<<E as Vertex>::Stored>>::get(x).clone();
            *<A::Outs as AtPos<<C as SelectChain<<E as Vertex>::Stored>>::Out, P>>::at_mut(&mut *ctx.outs) = v;
        }
    }
    fn init(ctx: &mut ConstructCtx<'_, A>) {
        *<A::StatesOpt as AtPos<Option<SlotNode<E, C>>, P>>::at_mut(&mut *ctx.states) = Some(SlotNode(PhantomData));
        *<A::OutsOpt as AtPos<Option<<C as SelectChain<<E as Vertex>::Stored>>::Out>, P>>::at_mut(&mut *ctx.outs) =
            Some(<C as ChainInitial<<E as Vertex>::Stored>>::initial());
    }
}
impl<A: Layout, const P: usize> StepAt<SlotWriteBy<A>, P> for A
where
    A: NthS<crate::store::StateM, P>,
    <A as NthS<crate::store::StateM, P>>::T: SlotStep<A, P>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn step(ctx: &mut PushCtx<'_, A>) {
        <<A as NthS<crate::store::StateM, P>>::T as SlotStep<A, P>>::write(ctx)
    }
}
impl<A: Layout, const P: usize> StepAt<SlotInitPh<A>, P> for A
where
    A: NthS<crate::store::StateM, P>,
    <A as NthS<crate::store::StateM, P>>::T: SlotStep<A, P>,
{
    fn step(ctx: &mut ConstructCtx<'_, A>) {
        <<A as NthS<crate::store::StateM, P>>::T as SlotStep<A, P>>::init(ctx)
    }
}
