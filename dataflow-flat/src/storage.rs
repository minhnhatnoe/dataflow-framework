//! The states tree: every component stored once, where the walk first reaches
//! it. A node stores only the entries whose edge is the first one reaching them
//! (no placeholders for the others); choosing which is the one place the plan's
//! constants choose types (`generic_const_exprs`).

use std::any::type_name;

use crate::build::ConstructPass;
use crate::component::{Component, ComponentSubscription, DepList};
use crate::lookup::{Done, Span};
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

/// Which of a node's entries are stored here (the edge that first reaches them
/// is this one): `MaskN<FIRST_0, .., FIRST_{N-1}>`, with the constants from the
/// plan. This is the one place the plan's constants select types
/// (`generic_const_exprs`).
#[doc(hidden)]
pub struct Mask0;
#[doc(hidden)]
pub struct Mask1<const A: bool>;
#[doc(hidden)]
pub struct Mask2<const A: bool, const B: bool>;
#[doc(hidden)]
pub struct Mask3<const A: bool, const B: bool, const C: bool>;
#[doc(hidden)]
pub struct Mask4<const A: bool, const B: bool, const C: bool, const D: bool>;

/// What a leaf node of N entries stores for a mask: a tuple of the `Tree`s of
/// the entries whose bit is set, in listing order (`()` if none). Only
/// the first-edge entries are stored; every other edge costs nothing at run
/// time and nothing in the type.
macro_rules! stored_trait {
    ($Tr:ident [$($E:ident)*]) => {
        #[doc(hidden)]
        pub trait $Tr<R: Planned, $($E: Placed<R>),*> {
            /// The stored subtrees.
            type S;
            /// What walking `S` produces.
            type O: Span<R> + 'static;
            fn make(pass: &mut ConstructPass) -> Self::S;
        }
    };
}
stored_trait!(Stored0 []);
stored_trait!(Stored1 [A]);
stored_trait!(Stored2 [A B]);
stored_trait!(Stored3 [A B C]);
stored_trait!(Stored4 [A B C D]);

macro_rules! stored_impl {
    ($Tr:ident, $Mask:ty, [$($E:ident)*] => [$($P:ident)*]) => {
        impl<R: Planned, $($E: Placed<R>),*> $Tr<R, $($E),*> for $Mask {
            type S = ($(Tree<R, $P>,)*);
            type O = ($(Done<R, $P>,)*);
            #[allow(unused_variables)]
            fn make(pass: &mut ConstructPass) -> Self::S {
                ($(Tree::<R, $P>::make(pass),)*)
            }
        }
    };
}
stored_impl!(Stored0, Mask0, [] => []);
stored_impl!(Stored1, Mask1<true>, [A] => [A]);
stored_impl!(Stored1, Mask1<false>, [A] => []);
stored_impl!(Stored2, Mask2<true, true>, [A B] => [A B]);
stored_impl!(Stored2, Mask2<true, false>, [A B] => [A]);
stored_impl!(Stored2, Mask2<false, true>, [A B] => [B]);
stored_impl!(Stored2, Mask2<false, false>, [A B] => []);
stored_impl!(Stored3, Mask3<true, true, true>, [A B C] => [A B C]);
stored_impl!(Stored3, Mask3<true, true, false>, [A B C] => [A B]);
stored_impl!(Stored3, Mask3<true, false, true>, [A B C] => [A C]);
stored_impl!(Stored3, Mask3<true, false, false>, [A B C] => [A]);
stored_impl!(Stored3, Mask3<false, true, true>, [A B C] => [B C]);
stored_impl!(Stored3, Mask3<false, true, false>, [A B C] => [B]);
stored_impl!(Stored3, Mask3<false, false, true>, [A B C] => [C]);
stored_impl!(Stored3, Mask3<false, false, false>, [A B C] => []);
stored_impl!(Stored4, Mask4<true, true, true, true>, [A B C D] => [A B C D]);
stored_impl!(Stored4, Mask4<true, true, true, false>, [A B C D] => [A B C]);
stored_impl!(Stored4, Mask4<true, true, false, true>, [A B C D] => [A B D]);
stored_impl!(Stored4, Mask4<true, true, false, false>, [A B C D] => [A B]);
stored_impl!(Stored4, Mask4<true, false, true, true>, [A B C D] => [A C D]);
stored_impl!(Stored4, Mask4<true, false, true, false>, [A B C D] => [A C]);
stored_impl!(Stored4, Mask4<true, false, false, true>, [A B C D] => [A D]);
stored_impl!(Stored4, Mask4<true, false, false, false>, [A B C D] => [A]);
stored_impl!(Stored4, Mask4<false, true, true, true>, [A B C D] => [B C D]);
stored_impl!(Stored4, Mask4<false, true, true, false>, [A B C D] => [B C]);
stored_impl!(Stored4, Mask4<false, true, false, true>, [A B C D] => [B D]);
stored_impl!(Stored4, Mask4<false, true, false, false>, [A B C D] => [B]);
stored_impl!(Stored4, Mask4<false, false, true, true>, [A B C D] => [C D]);
stored_impl!(Stored4, Mask4<false, false, true, false>, [A B C D] => [C]);
stored_impl!(Stored4, Mask4<false, false, false, true>, [A B C D] => [D]);
stored_impl!(Stored4, Mask4<false, false, false, false>, [A B C D] => []);

/// The storage of a subscription tree `Self` owned by component `P`: for a
/// leaf, the tuple of the stored subtrees of its entries (see [`Stored1`]);
/// for an inner node, `(its entries' tuple, child 0, child 1, child 2, child 3)`.
#[doc(hidden)]
pub trait DepStorage<R: Planned, P: Component>: DepList {
    /// The stored subtrees.
    type S;
    /// The finished outputs of `S`.
    type O: Span<R> + 'static;
    fn make(pass: &mut ConstructPass) -> Self::S;
}
macro_rules! dep_storage_leaf {
    ($Tr:ident, $Mask:ident, [$($E:ident)*]) => {
        impl<R: Planned, P: Component, $($E: Placed<R>),*> DepStorage<R, P> for ($($E,)*)
        where
            $Mask<$({ Edge::<R, P, $E>::FIRST }),*>: $Tr<R, $($E),*>,
        {
            type S = <$Mask<$({ Edge::<R, P, $E>::FIRST }),*> as $Tr<R, $($E),*>>::S;
            type O = <$Mask<$({ Edge::<R, P, $E>::FIRST }),*> as $Tr<R, $($E),*>>::O;
            fn make(pass: &mut ConstructPass) -> Self::S {
                <$Mask<$({ Edge::<R, P, $E>::FIRST }),*> as $Tr<R, $($E),*>>::make(pass)
            }
        }
    };
}
dep_storage_leaf!(Stored0, Mask0, []);
dep_storage_leaf!(Stored1, Mask1, [A]);
dep_storage_leaf!(Stored2, Mask2, [A B]);
dep_storage_leaf!(Stored3, Mask3, [A B C]);
dep_storage_leaf!(Stored4, Mask4, [A B C D]);

impl<R: Planned, P: Component, A: Placed<R>, B: Placed<R>, C: Placed<R>, D: Placed<R>,
     K0: DepStorage<R, P>, K1: DepStorage<R, P>, K2: DepStorage<R, P>, K3: DepStorage<R, P>>
    DepStorage<R, P> for (A, B, C, D, K0, K1, K2, K3)
where
    (A, B, C, D): DepStorage<R, P>,
{
    type S = (<(A, B, C, D) as DepStorage<R, P>>::S, K0::S, K1::S, K2::S, K3::S);
    type O = (<(A, B, C, D) as DepStorage<R, P>>::O, K0::O, K1::O, K2::O, K3::O);
    fn make(pass: &mut ConstructPass) -> Self::S {
        (
            <(A, B, C, D) as DepStorage<R, P>>::make(pass),
            K0::make(pass),
            K1::make(pass),
            K2::make(pass),
            K3::make(pass),
        )
    }
}

const _: () = assert!(
    crate::ARITY == 4,
    "the node-shape impls in this file are written for ARITY = 4; update them"
);
