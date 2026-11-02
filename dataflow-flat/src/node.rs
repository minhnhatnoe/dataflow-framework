//! Nodes: everything a component can subscribe to.
//!
//! [`Node`] is sealed (the derives implement it). In phase 1a there are two
//! kinds of node a user can name: the **root** (`#[derive(Root)]`, the value
//! passed to `Pipeline::push`) and **components** (`#[derive(ComponentSubscription)]`
//! + `impl Component`). The pipeline also has one internal node, the `Anchor`,
//! that stands for the sinks.
//!
//! What a subscriber receives is decided by the *kind* of the node:
//!
//! * `update` gets `&Output`: for a component the stored output of its last
//!   run, for the root the value of the current push;
//! * `new` gets [`InitOf`]: `&Output` of the already constructed component, or
//!   `()` for the root (it has no value before the first push).

use crate::component::DepList;

/// Marker: implementors can only be written by this crate's derives.
#[doc(hidden)]
pub mod __private {
    pub trait Sealed {}
}

/// Kind numbers, as stored in the plan.
#[doc(hidden)]
pub const KIND_ROOT: u8 = 0;
#[doc(hidden)]
pub const KIND_COMPONENT: u8 = 1;
#[doc(hidden)]
pub const KIND_ANCHOR: u8 = 3;

/// A thing that can be subscribed to. Sealed: use `#[derive(Root)]` or
/// `#[derive(ComponentSubscription)]`.
pub trait Node: __private::Sealed + Sized + 'static {
    /// What subscribers read in `update`.
    type Output: 'static;
    /// The node's subscriptions, as a [`DepList`].
    #[doc(hidden)]
    type Subs: DepList;
    /// Root, component or anchor.
    #[doc(hidden)]
    type Kind: NodeKind;
    #[doc(hidden)]
    const KIND: u8;
    /// Is this a handler component (`#[derive(Handler)]`)? The plan treats its
    /// handled edges differently (see `handler`).
    #[doc(hidden)]
    const HANDLER: bool = false;
    /// Is this a [`KeyListener`](crate::KeyListener) (`#[key_listener]` on the derive)?
    #[doc(hidden)]
    const LISTENER: bool = false;
    /// Is this a [`Listener`](crate::Listener) (`#[listener]`)?
    #[doc(hidden)]
    const LISTENS: bool = false;
    /// The listener's `Requests` (`NoRequests` for other nodes).
    #[doc(hidden)]
    type Requests: 'static;
    /// `KeyListener::subscribe`, for a key listener; nothing otherwise.
    #[doc(hidden)]
    fn __subscribe(_this: &mut Self, _keys: &[u32]) {}
}

/// What the kind of a node decides at the type level: what its subscribers
/// are given in `new`.
#[doc(hidden)]
pub trait NodeKind: 'static {
    type Init<'a, N: Node>;
}

/// The root: nothing is known about it before the first push.
#[doc(hidden)]
pub struct RootKind;
impl NodeKind for RootKind {
    type Init<'a, N: Node> = ();
}
/// A component: its first output exists when later components are constructed.
#[doc(hidden)]
pub struct CompKind;
impl NodeKind for CompKind {
    type Init<'a, N: Node> = &'a N::Output;
}
/// The anchor. Never subscribed to.
#[doc(hidden)]
pub struct AnchorKind;
impl NodeKind for AnchorKind {
    type Init<'a, N: Node> = ();
}

/// `Node::Init<'a>` of the design: what `new` of a subscriber is given for
/// the subscription `N`.
pub type InitOf<'a, N> = <<N as Node>::Kind as NodeKind>::Init<'a, N>;

/// The root of a pipeline: the type passed to `Pipeline::push`. Derive it:
///
/// ```
/// #[derive(dataflow_flat::Root)]
/// struct Poll;
///
/// #[derive(dataflow_flat::Root)]
/// enum Wake { Nic0, Nic1 }
/// ```
pub trait Root: Node<Kind = RootKind, Output = Self> + Sized {}

/// A type-level proof that two types are the same, used where the plan has
/// already checked that they are (the root of the pipeline is the root named
/// by the subscriptions).
#[doc(hidden)]
pub trait Is<T> {
    fn conv(&self) -> &T;
}
impl<T> Is<T> for T {
    #[inline]
    fn conv(&self) -> &T {
        self
    }
}

/// A node that is a component, with its output seen as a `Node`.
#[doc(hidden)]
pub trait Comp: crate::component::Component + Node<Output = <Self as crate::component::Component>::Output, Kind = CompKind> {}
impl<C> Comp for C where
    C: crate::component::Component + Node<Output = <C as crate::component::Component>::Output, Kind = CompKind>
{
}
