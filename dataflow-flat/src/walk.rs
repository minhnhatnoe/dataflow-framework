//! The run: a fold over the states tree, and the [`Root`] bound that ties a
//! root component to its storage and walk.

use crate::build::ConstructPass;
use crate::component::Component;
use crate::lookup::{Context, Done, GatherInputs, Pending};
use crate::plan::Planned;
use crate::storage::{DepsDone, DepsStored, If, Placed, Slot, Tree};

/// Run a (piece of a) states tree under context `T`, producing its finished
/// outputs by value.
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
    type Out: 'static;
    fn walk<'a>(&mut self, ctx: T::Refs<'a>) -> Self::Out;
}

/// The end of a dependency list.
impl<R, T: Context> Walk<R, T> for () {
    type Out = ();
    #[inline]
    fn walk<'a>(&mut self, _: T::Refs<'a>) {}
}

/// A non-first edge: nothing stored, nothing to run.
impl<R, T: Context, B: Walk<R, T>> Walk<R, T> for ((), B) {
    type Out = ((), B::Out);
    #[inline]
    fn walk<'a>(&mut self, ctx: T::Refs<'a>) -> Self::Out {
        ((), self.1.walk(ctx))
    }
}

/// A first edge: run the subtree, then the later siblings with it in context.
impl<R: Planned, C: Placed<R>, T: Context, B> Walk<R, T> for (Tree<R, C>, B)
where
    Tree<R, C>: Walk<R, T, Out = Done<R, C>>,
    B: Walk<R, (Done<R, C>, T)>,
{
    type Out = (Done<R, C>, <B as Walk<R, (Done<R, C>, T)>>::Out);
    #[inline]
    fn walk<'a>(&mut self, ctx: T::Refs<'a>) -> Self::Out {
        let a = self.0.walk(ctx);
        // Later siblings may read anything first reached inside `a`.
        let b = self.1.walk((&a, T::shorten(ctx)));
        (a, b)
    }
}

/// One component: upstream first, then `update` on the component field.
impl<R: Planned, C: Placed<R>, T: Context> Walk<R, T> for Tree<R, C>
where
    DepsStored<R, C>: Walk<R, T, Out = DepsDone<R, C>>,
    C::Deps: GatherInputs<R, (Pending<R, C>, T)>,
{
    type Out = Done<R, C>;
    #[inline]
    fn walk<'a>(&mut self, ctx: T::Refs<'a>) -> Self::Out {
        // Upstream first; first-edge dependencies come back by value, the
        // others are already finished somewhere in `ctx`.
        let deps = self.deps.walk(ctx);
        let out = {
            let refs = (&deps, T::shorten(ctx));
            let inputs = <C::Deps as GatherInputs<R, (Pending<R, C>, T)>>::gather(refs);
            self.node.update(C::input(inputs))
        };
        Done { deps, out }
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
    #[doc(hidden)]
    fn run(states: &mut Self::States) -> Self::Output;
}

impl<A: Component> Root for A
where
    A: Placed<A>,
    Tree<A, A>: Walk<A, (), Out = Done<A, A>>,
{
    type States = Tree<A, A>;
    fn make(pass: &mut ConstructPass) -> Tree<A, A> {
        <If<true> as Slot<A, A>>::make(pass)
    }
    #[inline]
    fn run(states: &mut Tree<A, A>) -> A::Output {
        states.walk(()).into_output()
    }
}
