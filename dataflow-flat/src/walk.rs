//! The run: a fold over the states tree, and the [`Root`] bound that ties a
//! root component to its storage and walk.

use std::any::TypeId;

use crate::build::ConstructPass;
use crate::component::Component;
use crate::lookup::{Context, Done, GatherInputs, Node, Span};
use crate::plan::{Planned, Upstream};
use crate::storage::{DepsDone, DepsStored, Placed, Tree};

/// Run a (piece of a) states tree under context `T`, producing its finished
/// outputs by value.
///
/// A piece is a [`Tree`], a node of a subscription tree's storage (a tuple), or
/// `()` (a non-first edge or an unused child). Walking a piece also says what a
/// later sibling sees: `After`, the context extended by this piece's finished
/// outputs (one entry per piece: a `Done` for a tree, a `Node` for a tuple,
/// nothing for `()`).
///
/// The `on_unimplemented` text is what users see when a type that is not a
/// `Component` (or generic code without the `Root` bound) is used as the root
/// of a `Pipeline`: the failing obligation the compiler reports is this one.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "`{R}` can't be the root of a `Pipeline`: its dataflow graph can't be run",
    label = "`{R}` must implement `Component`; in generic code add the bound `{R}: Root`"
)]
pub trait Walk<R, T: Context> {
    type Out: Span<R> + 'static;
    type After: Context;
    fn walk<'a>(&mut self, ctx: T::Refs<'a>) -> Self::Out;
    /// The context seen by a later sibling, given this piece's outputs.
    fn after<'a, 'b>(out: &'b Self::Out, ctx: T::Refs<'a>) -> <Self::After as Context>::Refs<'b>
    where
        'a: 'b;
}

/// Nothing stored: a non-first edge or an unused child.
impl<R, T: Context> Walk<R, T> for () {
    type Out = ();
    type After = T;
    #[inline(always)]
    fn walk<'a>(&mut self, _: T::Refs<'a>) {}
    #[inline(always)]
    fn after<'a, 'b>(_: &'b (), ctx: T::Refs<'a>) -> T::Refs<'b>
    where
        'a: 'b,
    {
        T::shorten(ctx)
    }
}

/// Walk fields one after another, each seeing what the previous ones finished.
/// The last field's `after` is not needed and not computed.
macro_rules! walk_seq {
    ($self:ident, $ctx:ident;) => {};
    ($self:ident, $ctx:ident; $E:ident $e:ident $i:tt $Ci:ty;) => {
        let $e = $self.$i.walk($ctx);
    };
    ($self:ident, $ctx:ident; $E:ident $e:ident $i:tt $Ci:ty; $($rest:tt)+) => {
        let $e = $self.$i.walk($ctx);
        let $ctx = <$E as Walk<R, $Ci>>::after(&$e, $ctx);
        walk_seq!($self, $ctx; $($rest)+);
    };
}

/// A leaf node: its entries in order, each seeing the finished ones before it.
/// Each entry is `E e i O Cin Cout`: type, variable, field, output type, the
/// context it is walked under, and the context it leaves.
macro_rules! walk_leaf {
    ($($E:ident $e:ident $i:tt $O:ident $Ci:ident $Co:ident;)+) => {
        impl<R: Planned, T: Context, $($E, $O: Span<R> + 'static, $Co: Context),+> Walk<R, T> for ($($E,)+)
        where
            $($E: Walk<R, $Ci, Out = $O, After = $Co>,)+
        {
            type Out = ($($O,)+);
            type After = (Node<R, ($($O,)+)>, T);
            #[inline(always)]
            fn walk<'a>(&mut self, ctx: T::Refs<'a>) -> Self::Out {
                walk_seq!(self, ctx; $($E $e $i $Ci;)+);
                ($($e,)+)
            }
            #[inline(always)]
            fn after<'a, 'b>(out: &'b Self::Out, ctx: T::Refs<'a>) -> <Self::After as Context>::Refs<'b>
            where
                'a: 'b,
            {
                (out, T::shorten(ctx))
            }
        }
    };
}
walk_leaf!(A a 0 OA T XA;);
walk_leaf!(A a 0 OA T XA; B b 1 OB XA XB;);
walk_leaf!(A a 0 OA T XA; B b 1 OB XA XB; C c 2 OC XB XC;);
walk_leaf!(A a 0 OA T XA; B b 1 OB XA XB; C c 2 OC XB XC; D d 3 OD XC XD;);

/// An inner node: the tuple of its stored entries, then its four children, in
/// this order. The entries tuple pushes the finished entries as one context
/// entry, and each finished child as one more.
impl<R: Planned, T: Context,
     P, OP: Span<R> + 'static, Y1: Context,
     K0, OK0: Span<R> + 'static, Y2: Context,
     K1, OK1: Span<R> + 'static, Y3: Context,
     K2, OK2: Span<R> + 'static, Y4: Context,
     K3, OK3: Span<R> + 'static, Y5: Context>
    Walk<R, T> for (P, K0, K1, K2, K3)
where
    P: Walk<R, T, Out = OP, After = Y1>,
    K0: Walk<R, Y1, Out = OK0, After = Y2>,
    K1: Walk<R, Y2, Out = OK1, After = Y3>,
    K2: Walk<R, Y3, Out = OK2, After = Y4>,
    K3: Walk<R, Y4, Out = OK3, After = Y5>,
{
    type Out = (OP, OK0, OK1, OK2, OK3);
    type After = (Node<R, Self::Out>, T);
    #[inline(always)]
    fn walk<'a>(&mut self, ctx: T::Refs<'a>) -> Self::Out {
        walk_seq!(self, ctx; P p 0 T; K0 k0 1 Y1; K1 k1 2 Y2; K2 k2 3 Y3; K3 k3 4 Y4;);
        (p, k0, k1, k2, k3)
    }
    #[inline(always)]
    fn after<'a, 'b>(out: &'b Self::Out, ctx: T::Refs<'a>) -> <Self::After as Context>::Refs<'b>
    where
        'a: 'b,
    {
        (out, T::shorten(ctx))
    }
}

/// One component: upstream first, then `update` on the component field.
impl<R: Planned, C: Placed<R>, T: Context> Walk<R, T> for Tree<R, C>
where
    DepsStored<R, C>: Walk<R, T, Out = DepsDone<R, C>>,
    C::Deps: GatherInputs<R, (Node<R, DepsDone<R, C>>, T)>,
{
    type Out = Done<R, C>;
    type After = (Done<R, C>, T);
    #[inline(always)]
    fn walk<'a>(&mut self, ctx: T::Refs<'a>) -> Self::Out {
        // Upstream first; first-edge dependencies come back by value, the
        // others are already finished somewhere in `ctx`.
        let deps = self.deps.walk(ctx);
        let out = {
            let refs = (&deps, T::shorten(ctx));
            let inputs = <C::Deps as GatherInputs<R, (Node<R, DepsDone<R, C>>, T)>>::gather(refs);
            self.node.update(C::input(inputs))
        };
        Done { deps, out }
    }
    #[inline(always)]
    fn after<'a, 'b>(out: &'b Self::Out, ctx: T::Refs<'a>) -> <Self::After as Context>::Refs<'b>
    where
        'a: 'b,
    {
        (out, T::shorten(ctx))
    }
}

/// A component that can be the root of a [`Pipeline`](crate::Pipeline): the
/// single component the application asks for, whose `update` receives what the
/// program wants and whose output is what [`Pipeline::run`](crate::Pipeline::run)
/// returns.
///
/// Implemented automatically for every [`Component`] whose graph can be
/// stored and walked, which is every component in a crate that compiles. You
/// only write `Root` as a bound, in code that is generic over the root:
///
/// ```
/// use dataflow_flat::{Pipeline, Root};
///
/// fn drive<A: Root>(p: &mut Pipeline<A>) -> A::Output {
///     p.run()
/// }
/// ```
pub trait Root: Component {
    /// The states tree.
    #[doc(hidden)]
    type States;
    #[doc(hidden)]
    fn make(pass: &mut ConstructPass) -> Self::States;
    /// Is the component with this `TypeId` part of the graph?
    #[doc(hidden)]
    fn in_plan(id: TypeId) -> bool;
    #[doc(hidden)]
    fn run(states: &mut Self::States) -> Self::Output;
}

impl<A: Component> Root for A
where
    A::Deps: const Upstream,
    A: Placed<A>,
    Tree<A, A>: Walk<A, (), Out = Done<A, A>>,
{
    type States = Tree<A, A>;
    fn make(pass: &mut ConstructPass) -> Tree<A, A> {
        Tree::make(pass)
    }
    fn in_plan(id: TypeId) -> bool {
        <A as Planned>::PLAN.contains(id)
    }
    #[inline(always)]
    fn run(states: &mut Tree<A, A>) -> A::Output {
        states.walk(()).into_output()
    }
}

const _: () = assert!(
    crate::ARITY == 4,
    "the node-shape impls in this file are written for ARITY = 4; update them"
);
