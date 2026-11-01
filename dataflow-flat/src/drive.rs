//! Driving a *phase* over the component positions in execution order: the
//! construction pass and a push. A phase is a type with a context; the work at
//! one position is `StepAt<Phase, P>`, a blanket impl with all the bounds, and
//! the driver recurses over the balanced ranges of positions (the same shape as
//! the storage tuples) and calls `step` for each position in order. After
//! inlining a push is straight-line code, as for a hand-written struct of the
//! components.
//!
//! # The condition hook
//!
//! The push step for position `P` is guarded by [`Cond<P>`]: whether the
//! component runs in this step, from the `fired` flags of its upstreams, and
//! whether it must record that it fired. In phase 1a every condition is
//! "always", so the guard, the flags and their upkeep are all constant-folded
//! away; phase 1b replaces `Cond`'s impl with the plan's conditions.

use std::any::type_name;
use std::marker::PhantomData;

use crate::build::ConstructPass;
use crate::component::{Component, ComponentSubscription};
use crate::flat::{bound, class, ClassOf, Inner, Leaf, Loc, NthT, Whole};
use crate::gather::Ready;
use crate::gather::{GatherIn, GatherInit, Layout};
use crate::node::{Comp, Node};
use crate::plan::{Planned, MAX_COMPONENTS, NO_FLAG};
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
pub struct Fired {
    bits: [bool; MAX_COMPONENTS],
}
/// Operations on the `fired` flags (of whichever size the pipeline needs).
#[doc(hidden)]
pub trait FiredOps {
    fn new() -> Self;
    fn set(&mut self, flag: usize);
    fn get(&self, flag: usize) -> bool;
}
impl FiredOps for Fired {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn new() -> Fired {
        Fired { bits: [false; MAX_COMPONENTS] }
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

/// The condition of the component at component position `P`, from the plan: a
/// component runs iff `ALWAYS || ready(..)` (the OR over its edges, see
/// [`Ready`]); if it has a `FLAG`, running it records that it fired.
#[doc(hidden)]
pub struct CondAt<A, const P: usize>(PhantomData<A>);
impl<A: Planned, const P: usize> CondAt<A, P> {
    pub const ALWAYS: bool = A::PLAN.always_idx(Loc::<A, P>::IDX);
    pub const FLAG: usize = A::PLAN.flag_idx(Loc::<A, P>::IDX);
}

#[doc(hidden)]
pub struct PushPh<A>(PhantomData<A>);
#[doc(hidden)]
pub struct PushCtx<'c, A: Layout> {
    pub(crate) root: &'c A::Root,
    pub(crate) states: &'c mut A::States,
    pub(crate) outs: &'c mut A::Outs,
    pub(crate) fired: FiredOf<A>,
}
impl<A: Layout> Phase for PushPh<A> {
    type Ctx<'c> = PushCtx<'c, A>;
}

impl<A: Layout, const P: usize> StepAt<PushPh<A>, P> for A
where
    A: NthT<P>,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: Ready<A>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as ComponentSubscription>::Deps: GatherIn<A>,
    A::States: AtPos<<A as NthT<P>>::T, P>,
    A::Outs: AtPos<<<A as NthT<P>>::T as Node>::Output, P>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn step(ctx: &mut PushCtx<'_, A>) {
        type C<A, const P: usize> = <A as NthT<P>>::T;
        if CondAt::<A, P>::ALWAYS
            || <<<A as NthT<P>>::T as ComponentSubscription>::Deps as Ready<A>>::ready(ctx.root, &*ctx.outs, &ctx.fired)
        {
            // The inputs borrow the stored outputs and the root only while
            // `update` runs; the write to the output comes after.
            let out = {
                let inputs = <<C<A, P> as ComponentSubscription>::Deps as GatherIn<A>>::gather(ctx.root, &*ctx.outs);
                <A::States as AtPos<C<A, P>, P>>::at_mut(&mut *ctx.states)
                    .update(<C<A, P> as ComponentSubscription>::input(inputs))
            };
            *<A::Outs as AtPos<<C<A, P> as Node>::Output, P>>::at_mut(&mut *ctx.outs) = out;
            if CondAt::<A, P>::FLAG != NO_FLAG {
                ctx.fired.set(CondAt::<A, P>::FLAG);
            }
        }
    }
}
