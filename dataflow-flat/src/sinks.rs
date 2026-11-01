//! The anchor of a pipeline: the sinks, and the one bound generic code needs.

use std::any::TypeId;
use std::marker::PhantomData;

use crate::build::ConstructPass;
use crate::component::{DepList, Sub};
use crate::drive::{ConstructCtx, ConstructPh, Drive, Exec, FiredOps, FiredTy, NFlags, Pn, PushCtx, PushPh, RgCl, RootProg};
use crate::flat::{FlatTop, Whole};
use crate::gather::Layout;
use crate::listen::{ListenCtx, ListenPh};
use crate::node::{AnchorKind, Comp, Node, Root, KIND_ANCHOR};
use crate::plan::{Plan, Planned, Upstream};
use crate::store::{Empty, Finish, OutM, OutOptM, StateM, StateOptM};

/// A tuple of 1 to 7 components: the sinks of a pipeline. The graph is
/// everything upstream of the sinks.
///
/// ```ignore
/// Pipeline::<Poll, (OrderSender, Logger)>::build(configs)
/// ```
pub trait SinkList: 'static {
    /// The sinks as the subscription tree of the anchor.
    #[doc(hidden)]
    type Subs: DepList;
}
macro_rules! sink_list_impl {
    ($($C:ident $i:tt)+) => {
        impl<$($C: Comp,)+> SinkList for ($($C,)+) {
            type Subs = ($(Sub<$C, (), $i>,)+);
        }
    };
}
dataflow_macros::sink_tuples!(sink_list_impl);

/// The node that stands for the sinks of `Pipeline<R, S>`: it subscribes to
/// all of them, is the last in the execution order, and has no state, no
/// output and no subscribers. Internal.
#[doc(hidden)]
pub struct Anchor<R, S>(PhantomData<fn() -> (R, S)>);
impl<R: 'static, S: SinkList> crate::node::__private::Sealed for Anchor<R, S> {}
impl<R: 'static, S: SinkList> Node for Anchor<R, S> {
    type Output = ();
    type Subs = S::Subs;
    type Kind = AnchorKind;
    const KIND: u8 = KIND_ANCHOR;
}

impl<R: Root, S: SinkList> Planned for Anchor<R, S>
where
    S::Subs: const Upstream,
{
    const PLAN: &'static Plan = &Plan::of::<Anchor<R, S>>(TypeId::of::<R>());
}

impl<R: Root, S: SinkList> Layout for Anchor<R, S>
where
    Anchor<R, S>: Planned
        + FlatTop<StateM>
        + FlatTop<OutM>
        + FlatTop<StateOptM>
        + FlatTop<OutOptM>,

    NFlags<Anchor<R, S>>: FiredTy,
    <Anchor<R, S> as FlatTop<StateOptM>>::Of: Empty + Finish<Done = <Anchor<R, S> as FlatTop<StateM>>::Of>,
    <Anchor<R, S> as FlatTop<OutOptM>>::Of: Empty + Finish<Done = <Anchor<R, S> as FlatTop<OutM>>::Of>,
{
    type Root = R;
    type States = <Anchor<R, S> as FlatTop<StateM>>::Of;
    type Outs = <Anchor<R, S> as FlatTop<OutM>>::Of;
    type StatesOpt = <Anchor<R, S> as FlatTop<StateOptM>>::Of;
    type OutsOpt = <Anchor<R, S> as FlatTop<OutOptM>>::Of;
    type Fired = <NFlags<Anchor<R, S>> as FiredTy>::T;
}

/// The storage of a pipeline.
#[doc(hidden)]
pub struct Store<St, O> {
    pub(crate) states: St,
    pub(crate) outs: O,
}

/// `S: Sinks<R>`: the sinks `S` form a valid pipeline for root `R`. This is the
/// one bound generic code over a `Pipeline<R, S>` writes; it holds for every
/// pipeline that compiles (it is implemented for every [`SinkList`] whose graph
/// can be planned, stored and run).
///
/// ```
/// use dataflow_flat::{Pipeline, Sinks};
///
/// fn feed<R, S: Sinks<R>>(p: &mut Pipeline<R, S>, steps: impl IntoIterator<Item = R>) {
///     for r in steps {
///         p.push(r);
///     }
/// }
/// ```
///
/// The `on_unimplemented` text is what users see when the graph of a pipeline
/// can't be run: the failing obligation the compiler reports is this one.
#[diagnostic::on_unimplemented(
    message = "`{Self}` are not valid sinks for a `Pipeline` with root `{R}`: the dataflow graph can't be built",
    label = "every sink must be a component, `{R}` must be a `#[derive(Root)]`; in generic code add the bound `{Self}: Sinks<{R}>`"
)]
pub trait Sinks<R>: SinkList {
    /// States and stored outputs.
    #[doc(hidden)]
    type Store;
    /// Is the node with this `TypeId` part of the graph?
    #[doc(hidden)]
    fn in_plan(id: TypeId) -> bool;
    #[doc(hidden)]
    fn make(pass: &mut ConstructPass) -> Self::Store;
    #[doc(hidden)]
    fn push(store: &mut Self::Store, root: R);
    /// Tell each listener the keys its subscribers read. Called once by `build`.
    #[doc(hidden)]
    fn listen(store: &mut Self::Store);
}

impl<R: Root, S: SinkList> Sinks<R> for S
where
    Anchor<R, S>: Layout<Root = R>,
    Whole<Anchor<R, S>>: RgCl,
    Whole<Anchor<R, S>>:
        Drive<Anchor<R, S>, ConstructPh<Anchor<R, S>>, <Whole<Anchor<R, S>> as RgCl>::Cl>,
    Pn<Anchor<R, S>, { RootProg::<Anchor<R, S>>::N }>: Exec<Anchor<R, S>, PushPh<Anchor<R, S>>>,
    Whole<Anchor<R, S>>: Drive<Anchor<R, S>, ListenPh<Anchor<R, S>>, <Whole<Anchor<R, S>> as RgCl>::Cl>,
{
    type Store = Store<<Anchor<R, S> as Layout>::States, <Anchor<R, S> as Layout>::Outs>;

    fn in_plan(id: TypeId) -> bool {
        <Anchor<R, S> as Planned>::PLAN.contains(id)
    }

    fn make(pass: &mut ConstructPass) -> Self::Store {
        type A<R, S> = Anchor<R, S>;
        // Report an invalid graph (a second root, an unreachable component).
        const { <A<R, S> as Planned>::CHECK };
        let mut states = <<A<R, S> as Layout>::StatesOpt as Empty>::EMPTY;
        let mut outs = <<A<R, S> as Layout>::OutsOpt as Empty>::EMPTY;
        {
            let mut ctx = ConstructCtx::<A<R, S>> { states: &mut states, outs: &mut outs, pass };
            <Whole<A<R, S>> as Drive<A<R, S>, ConstructPh<A<R, S>>, <Whole<A<R, S>> as RgCl>::Cl>>::drive(&mut ctx);
        }
        Store { states: states.finish(), outs: outs.finish() }
    }

    #[cfg_attr(not(debug_assertions), inline(always))]

    #[cfg_attr(debug_assertions, inline)]
    fn push(store: &mut Self::Store, root: R) {
        type A<R, S> = Anchor<R, S>;
        let mut ctx =
            PushCtx::<A<R, S>> { root: &root, states: &mut store.states, outs: &mut store.outs, fired: <<A<R, S> as Layout>::Fired as FiredOps>::new() };
        <Pn<A<R, S>, { RootProg::<A<R, S>>::N }> as Exec<A<R, S>, PushPh<A<R, S>>>>::exec(&mut ctx);
    }

    fn listen(store: &mut Self::Store) {
        type A<R, S> = Anchor<R, S>;
        let mut ctx = ListenCtx::<A<R, S>> { states: &mut store.states };
        <Whole<A<R, S>> as Drive<A<R, S>, ListenPh<A<R, S>>, <Whole<A<R, S>> as RgCl>::Cl>>::drive(&mut ctx);
    }
}
