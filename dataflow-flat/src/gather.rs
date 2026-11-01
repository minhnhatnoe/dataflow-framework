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
use crate::identity::At;
use crate::node::{CompKind, Is, Node, NodeKind, RootKind};
use crate::plan::Planned;
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
}

/// Component position of node `N` in the plan of `A`.
#[doc(hidden)]
pub struct PosOf<A, N>(PhantomData<(A, N)>);
impl<A: Planned, N: Node> PosOf<A, N> {
    /// Execution position 0 is the root, so component position = post - 1.
    pub const P: usize = A::PLAN.post_of_idx(At::<A, N>::INDEX) - 1;
}

/// How a node of kind `Self` is read.
#[doc(hidden)]
pub trait SourceKind<N: Node, A: Layout>: NodeKind {
    /// What `update` reads.
    fn fetch<'a>(root: &'a A::Root, outs: &'a A::Outs) -> &'a N::Output;
    /// What `new` is given while the pipeline is built.
    fn seed<'a>(outs: &'a A::OutsOpt) -> <Self as NodeKind>::Init<'a, N>;
}

impl<N: Node<Kind = RootKind>, A: Layout> SourceKind<N, A> for RootKind
where
    A::Root: Is<N::Output>,
{
    #[inline]
    fn fetch<'a>(root: &'a A::Root, _: &'a A::Outs) -> &'a N::Output {
        root.conv()
    }
    #[inline]
    fn seed<'a>(_: &'a A::OutsOpt) {}
}

impl<N: Node<Kind = CompKind>, A: Layout> SourceKind<N, A> for CompKind
where
    A::Outs: AtPos<N::Output, { PosOf::<A, N>::P }>,
    A::OutsOpt: AtPos<Option<N::Output>, { PosOf::<A, N>::P }>,
{
    #[inline]
    fn fetch<'a>(_: &'a A::Root, outs: &'a A::Outs) -> &'a N::Output {
        <A::Outs as AtPos<N::Output, { PosOf::<A, N>::P }>>::at(outs)
    }
    #[inline]
    fn seed<'a>(outs: &'a A::OutsOpt) -> &'a N::Output {
        <A::OutsOpt as AtPos<Option<N::Output>, { PosOf::<A, N>::P }>>::at(outs)
            .as_ref()
            .expect("a subscription is constructed before its subscriber")
    }
}

/// A node that can be read in the pipeline `A`.
#[doc(hidden)]
pub trait Source<A: Layout>: Node {
    fn fetch<'a>(root: &'a A::Root, outs: &'a A::Outs) -> &'a Self::Output;
    fn seed<'a>(outs: &'a A::OutsOpt) -> crate::node::InitOf<'a, Self>;
}
impl<A: Layout, N: Node> Source<A> for N
where
    N::Kind: SourceKind<N, A>,
{
    #[inline]
    fn fetch<'a>(root: &'a A::Root, outs: &'a A::Outs) -> &'a N::Output {
        <N::Kind as SourceKind<N, A>>::fetch(root, outs)
    }
    #[inline]
    fn seed<'a>(outs: &'a A::OutsOpt) -> crate::node::InitOf<'a, N> {
        <N::Kind as SourceKind<N, A>>::seed(outs)
    }
}

/// The inputs of one component's `update`, read from the pushed root and the
/// stored outputs.
#[doc(hidden)]
pub trait GatherIn<A: Layout>: DepList {
    fn gather<'a>(root: &'a A::Root, outs: &'a A::Outs) -> Self::Inputs<'a>;
}
macro_rules! gather_in_leaf {
    ([$($E:ident $I:ident)*]) => {
        impl<A: Layout, $($E: Source<A>, const $I: usize,)*> GatherIn<A> for ($(Sub<$E, $I>,)*) {
            #[inline]
            #[allow(unused_variables)]
            fn gather<'a>(root: &'a A::Root, outs: &'a A::Outs) -> Self::Inputs<'a> {
                ($(<$E as Source<A>>::fetch(root, outs),)*)
            }
        }
    };
}
dataflow_macros::leaf_shapes!(gather_in_leaf);
macro_rules! gather_in_inner {
    ([$($K:ident)*]) => {
        impl<A: Layout, $($K: GatherIn<A>,)*> GatherIn<A> for ($($K,)*) {
            #[inline]
            fn gather<'a>(root: &'a A::Root, outs: &'a A::Outs) -> Self::Inputs<'a> {
                ($(<$K as GatherIn<A>>::gather(root, outs),)*)
            }
        }
    };
}
dataflow_macros::inner_shape!(gather_in_inner);

/// What one component's `new` is given, read from the outputs built so far.
#[doc(hidden)]
pub trait GatherInit<A: Layout>: DepList {
    fn gather<'a>(outs: &'a A::OutsOpt) -> Self::Inits<'a>;
}
macro_rules! gather_init_leaf {
    ([$($E:ident $I:ident)*]) => {
        impl<A: Layout, $($E: Source<A>, const $I: usize,)*> GatherInit<A> for ($(Sub<$E, $I>,)*) {
            #[inline]
            #[allow(unused_variables)]
            fn gather<'a>(outs: &'a A::OutsOpt) -> Self::Inits<'a> {
                ($(<$E as Source<A>>::seed(outs),)*)
            }
        }
    };
}
dataflow_macros::leaf_shapes!(gather_init_leaf);
macro_rules! gather_init_inner {
    ([$($K:ident)*]) => {
        impl<A: Layout, $($K: GatherInit<A>,)*> GatherInit<A> for ($($K,)*) {
            #[inline]
            fn gather<'a>(outs: &'a A::OutsOpt) -> Self::Inits<'a> {
                ($(<$K as GatherInit<A>>::gather(outs),)*)
            }
        }
    };
}
dataflow_macros::inner_shape!(gather_init_inner);
