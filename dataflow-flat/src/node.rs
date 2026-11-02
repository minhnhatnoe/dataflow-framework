//! Nodes: everything a component can subscribe to.
//!
//! [`Node`] is sealed (the derives implement it). There are two
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

/// What a subscriber's field is when it reads this node: `&Output` for a node that stores a
/// value, a view (`HandleComponent::Output<'a>`) for a handle component.
pub trait Edge {
    type Read<'a>
    where
        Self: 'a;
}

/// Everything the plan, the storage and the drivers need to know about a node (root, component,
/// handle component, anchor). Sealed. `Stored` is what its slot in the pipeline holds: the output
/// for a by-value component, the handle for a handle component.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a Node of the graph (the root, a component or a handle component)",
    label = "`{Self}` can't be subscribed to",
    note = "derive `Root` or `ComponentSubscription` (and implement `Component`, or `HandleComponent` with `#[handle]`) for it"
)]
pub trait Vertex: __private::Sealed + Edge + Sized + 'static {
    /// What the pipeline stores for this node.
    type Stored: 'static;
    /// The node's subscriptions, as a [`DepList`].
    type Subs: DepList;
    /// Root, component or anchor.
    type Kind: NodeKind;
    const KIND: u8;
    /// Is this a handler component (a `#[subscribe]` with handled entries)? The plan treats its
    /// handled edges differently (see `handler`).
    const HANDLER: bool = false;
    /// Is this a [`Listener`](crate::Listener) (`#[listener]`)?
    const LISTENS: bool = false;
    /// The listener's `Requests` (`NoRequests` for other nodes).
    type Requests: 'static;
}

/// A thing that can be subscribed to and whose subscribers read `&Output`: the root and
/// by-value components. Sealed: use `#[derive(Root)]` or `#[derive(ComponentSubscription)]`.
/// A handle component (`#[handle]`) is not a `Node`: its subscribers read a view, not `&Output`.
/// Generic code written as `X: Node<Output = T>` sees `&T` as the field type.
#[diagnostic::on_unimplemented(
    message = "`{Self}` is not a by-value Node: its subscribers read a view, not `&Output`",
    label = "`{Self}` is a `#[handle]` component (or not a node)",
    note = "selectors, handled entries and `Node<Output = ..>` bounds don't support `#[handle]` components yet"
)]
pub trait Node: Vertex
where
    for<'a> Self: Edge<Read<'a> = &'a <Self as Node>::Output>,
    Self: Vertex<Stored = <Self as Node>::Output>,
{
    /// What subscribers read in `update`.
    type Output: 'static;
}

/// What the kind of a node decides at the type level: what its subscribers
/// are given in `new`.
#[doc(hidden)]
pub trait NodeKind: 'static {
    type Init<'a, N: Vertex>;
    /// [`ByView`] if reading a node of this kind needs the state of the node (a handle
    /// component), else [`ByVal`]. A component whose subscriptions are all [`ByVal`] is updated
    /// the way it always was (`drive::Upd`).
    type Mode: ModeT;
}

/// Reading only stored outputs and the root.
#[doc(hidden)]
pub struct ByVal;
/// Reading a view of a handle component's state: the reader's state is split off from the
/// states before it.
#[doc(hidden)]
pub struct ByView;
/// A mode, and the mode of a pair of readings (a component is `ByView` if any of its
/// subscriptions is).
#[doc(hidden)]
pub trait ModeT {
    type Or<B: ModeT>: ModeT;
}
impl ModeT for ByVal {
    type Or<B: ModeT> = B;
}
impl ModeT for ByView {
    type Or<B: ModeT> = ByView;
}
#[doc(hidden)]
pub type ReadOf<'a, N> = <N as Edge>::Read<'a>;

/// The root: nothing is known about it before the first push.
#[doc(hidden)]
pub struct RootKind;
impl NodeKind for RootKind {
    type Mode = ByVal;
    type Init<'a, N: Vertex> = ();
}
/// A component: its first output exists when later components are constructed.
#[doc(hidden)]
pub struct CompKind;
impl NodeKind for CompKind {
    type Mode = ByVal;
    type Init<'a, N: Vertex> = &'a N::Stored;
}
/// A handle component: `new` of a subscriber gets the view computed from the constructed
/// component and its first handle.
#[doc(hidden)]
pub struct HandleKind;
impl NodeKind for HandleKind {
    type Mode = ByView;
    type Init<'a, N: Vertex> = <N as Edge>::Read<'a>;
}
/// The anchor. Never subscribed to.
#[doc(hidden)]
pub struct AnchorKind;
impl NodeKind for AnchorKind {
    type Mode = ByVal;
    type Init<'a, N: Vertex> = ();
}

/// `Vertex::Init<'a>` of the design: what `new` of a subscriber is given for
/// the subscription `N`.
pub type InitOf<'a, N: Vertex> = <<N as Vertex>::Kind as NodeKind>::Init<'a, N>;

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

/// A handle component's view of its state and stored handle (`HandleComponent::output`).
#[doc(hidden)]
pub trait HandleView: Vertex<Kind = HandleKind> {
    fn __view<'a>(&'a self, handle: &'a Self::Stored) -> <Self as Edge>::Read<'a>;
}

/// A component, by value or handle: what the drivers call to construct it and to run it. Derived
/// (`#[derive(ComponentSubscription)]` forwards to `Component` or, with `#[handle]`, to
/// `HandleComponent`).
#[doc(hidden)]
pub trait Comp: crate::component::ComponentSubscription + Vertex {
    /// `Component::Config` or `HandleComponent::Config`.
    type Cfg: Default + 'static;
    /// [`ByView`] if a subscription is a handle component, else [`ByVal`] (`DepList::Mode`).
    type Mode: ModeT;
    fn __new(config: Self::Cfg, init: Self::Init<'_>) -> (Self, Self::Stored);
    fn __update(&mut self, input: Self::Input<'_>) -> Self::Stored;
}

// ---------------------------------------------------------------------------
// Stored selected values
// ---------------------------------------------------------------------------

/// Kind number of a slot (see [`SlotNode`]).
#[doc(hidden)]
pub const KIND_SLOT: u8 = 4;

/// The kind of a [`SlotNode`]. Never subscribed to.
#[doc(hidden)]
pub struct SlotKind;
impl NodeKind for SlotKind {
    type Mode = ByVal;
    type Init<'a, N: Vertex> = ();
}

/// A stored selected value: the last value the selector chain `C` matched on
/// the output of `E`, kept for the subscribers that read it in steps where the
/// chain does not match. It is not a component: it has no subscriptions and is
/// not in the plan's walk. It takes a *position* after the components in the
/// flat storage, so that its value (`Output`) is stored like a component's
/// output.
#[doc(hidden)]
pub struct SlotNode<E, C>(pub core::marker::PhantomData<fn() -> (E, C)>);
impl<E: Vertex, C: crate::select::SelectChain<E::Stored>> __private::Sealed for SlotNode<E, C> {}
impl<E: Vertex, C: crate::select::SelectChain<E::Stored>> Edge for SlotNode<E, C> {
    type Read<'a> = &'a C::Out;
}
impl<E: Vertex, C: crate::select::SelectChain<E::Stored>> Vertex for SlotNode<E, C> {
    type Stored = C::Out;
    type Subs = ();
    type Kind = SlotKind;
    type Requests = crate::request::NoRequests;
    const KIND: u8 = KIND_SLOT;
}
