//! The states tree: every component stored once, where the walk first reaches
//! it. Selecting between "store the subtree here" and "stored elsewhere" is the
//! one place the plan's constants choose types (`generic_const_exprs`).

use std::any::type_name;

use crate::build::ConstructPass;
use crate::component::{Component, ComponentSubscription, DepList};
use crate::lookup::Done;
use crate::plan::{Edge, Planned};

/// `C: Placed<R>`: the storage of `C`'s dependencies exists for root `R`. This
/// is a one-line spelling of the bound `C::Deps: DepStorage<R, C>`, which
/// holds for every component of a graph that compiles; it exists so that the
/// many impls that need it do not repeat the projection.
#[doc(hidden)]
pub trait Placed<R: Planned>: Component<Deps: DepStorage<R, Self>> {}
impl<R: Planned, C: Component> Placed<R> for C where C::Deps: DepStorage<R, C> {}

/// The storage of component `C` as reached from root `R`: one entry per
/// subscription (the subtree of that dependency on the edge that first
/// reaches it, `()` on every other edge), then `C` itself. Field order is
/// construction order, and drop glue drops in the same order.
#[doc(hidden)]
pub struct Tree<R: Planned, C: Placed<R>> {
    pub(crate) deps: DepsStored<R, C>,
    pub(crate) node: C,
}

/// The stored subtrees of `C`'s dependencies.
#[doc(hidden)]
pub type DepsStored<R, C> = <<C as ComponentSubscription>::Deps as DepStorage<R, C>>::S;
/// The finished outputs of [`DepsStored`].
#[doc(hidden)]
pub type DepsDone<R, C> = <<C as ComponentSubscription>::Deps as DepStorage<R, C>>::O;

impl<R: Planned, C: Placed<R>> Tree<R, C> {
    /// Construct the dependencies (first, so construction order is
    /// topological), then `C` from its config.
    pub(crate) fn make(pass: &mut ConstructPass) -> Self {
        let deps = <C::Deps as DepStorage<R, C>>::make(pass);
        let config = pass.take_config::<C>();
        let node = C::new(config);
        pass.order.push(type_name::<C>());
        Tree { deps, node }
    }
}

/// Type-level `if`: selects between the two `Slot` impls with a constant
/// computed from the plan.
#[doc(hidden)]
pub struct If<const B: bool>;

/// What an edge to `H` stores: `Tree<R, H>` when `If<true>`, `()` otherwise.
#[doc(hidden)]
pub trait Slot<R: Planned, H: Component> {
    /// The stored subtree.
    type Out;
    /// What walking `Out` produces.
    type Fin: 'static;
    fn make(pass: &mut ConstructPass) -> Self::Out;
}
impl<R: Planned, H: Placed<R>> Slot<R, H> for If<true> {
    type Out = Tree<R, H>;
    type Fin = Done<R, H>;
    fn make(pass: &mut ConstructPass) -> Tree<R, H> {
        Tree::make(pass)
    }
}
impl<R: Planned, H: Component> Slot<R, H> for If<false> {
    type Out = ();
    type Fin = ();
    fn make(_: &mut ConstructPass) {}
}

/// The storage of a dependency list `Self` owned by component `P`, as a nested
/// tuple that mirrors the list.
#[doc(hidden)]
pub trait DepStorage<R: Planned, P: Component>: DepList {
    /// The stored subtrees.
    type S;
    /// The finished outputs of `S`.
    type O: 'static;
    fn make(pass: &mut ConstructPass) -> Self::S;
}
impl<R: Planned, P: Component> DepStorage<R, P> for () {
    type S = ();
    type O = ();
    fn make(_: &mut ConstructPass) {}
}
impl<R: Planned, P: Component, H: Placed<R>, T: DepStorage<R, P>> DepStorage<R, P> for (H, T)
where
    If<{ Edge::<R, P, H>::FIRST }>: Slot<R, H>,
{
    type S = (<If<{ Edge::<R, P, H>::FIRST }> as Slot<R, H>>::Out, T::S);
    type O = (<If<{ Edge::<R, P, H>::FIRST }> as Slot<R, H>>::Fin, T::O);
    fn make(pass: &mut ConstructPass) -> Self::S {
        let first = <If<{ Edge::<R, P, H>::FIRST }> as Slot<R, H>>::make(pass);
        let rest = T::make(pass);
        (first, rest)
    }
}
