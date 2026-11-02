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
/// value, a view (`HandleComponent::View<'a>`) for a handle component.
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
    /// The names of the inputs of a component, comma-separated in the order of its unique entries
    /// (the plan names an input in an error message).
    const INPUTS: &'static str = "";
    /// Is this a handle component? Its subscribers read a view of its state, so the plan gives
    /// them the `PG_RUN_VIEW` kinds (by-value components keep `PG_RUN`/`PG_RUN_IF`).
    const VIEW: bool = false;
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
    note = "a `Node<Output = ..>` bound means \"subscribers read `&Output`\", which a `#[handle]` component does not give (its subscribers read a view)"
)]
pub trait Node: Vertex<Kind: ValueKind>
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
    /// Is the node a handle component (its stored value is a handle, and its subscribers read a
    /// view of its state)?
    const IS_HANDLE: bool = false;
}

#[doc(hidden)]
pub type ReadOf<'a, N> = <N as Edge>::Read<'a>;

/// The kinds of the nodes whose subscribers read `&Output` (the root and components): `Node` pins
/// its `Kind` to these, so that generic code written for `X: Node<Output = T>` sees a plain
/// selector chain (`ChainWrap` below).
#[doc(hidden)]
pub trait ValueKind: NodeKind {}
impl ValueKind for RootKind {}
impl ValueKind for CompKind {}

/// The selector chain `C` of an entry whose source is a node `N`, as the plan and the drivers
/// see it: `C` itself for a by-value node (the chain selects from the stored output), a
/// [`ViewC`](crate::select::ViewC) for a handle component (the chain selects from the view). The
/// derive wraps every non-empty chain in `<N::Kind as ChainWrap<N, C>>::Out`. The by-value impl is
/// a blanket over `ValueKind` (so a generic `X: Node` normalizes it without knowing the kind), and
/// the handle impl is for `HandleKind`, which is not a `ValueKind`.
#[doc(hidden)]
pub trait ChainWrap<N: Vertex, C: 'static> {
    type Out: 'static;
}
impl<K: ValueKind, N: Vertex, C: 'static> ChainWrap<N, C> for K {
    type Out = C;
}
impl<N: Vertex, C: 'static> ChainWrap<N, C> for HandleKind {
    type Out = crate::select::ViewC<N, C>;
}

/// The root: nothing is known about it before the first push.
#[doc(hidden)]
pub struct RootKind;
impl NodeKind for RootKind {
    type Init<'a, N: Vertex> = ();
}
/// A component: its first output exists when later components are constructed.
#[doc(hidden)]
pub struct CompKind;
impl NodeKind for CompKind {
    type Init<'a, N: Vertex> = &'a N::Stored;
}
/// A handle component: `new` of a subscriber gets the view computed from the constructed
/// component and its first handle.
#[doc(hidden)]
pub struct HandleKind;
impl NodeKind for HandleKind {
    type Init<'a, N: Vertex> = <N as Edge>::Read<'a>;
    const IS_HANDLE: bool = true;
}
/// The anchor. Never subscribed to.
#[doc(hidden)]
pub struct AnchorKind;
impl NodeKind for AnchorKind {
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

/// A handle component's view of its state and stored handle (`HandleComponent::view`).
#[doc(hidden)]
pub trait HandleView: Vertex<Kind = HandleKind> {
    fn __view<'a>(&'a self, handle: &'a Self::Stored) -> <Self as Edge>::Read<'a>;
}

/// A node that is a component, with its stored value seen as its `Component::Output`. The drivers
/// call `Component::new` and `Component::update` through this bound, with no wrapper function in
/// between (a wrapper per component is one more item for the compiler to collect and for LLVM to
/// keep). A handle component gets its `Component` impl from the derive, with the handle as the
/// `Output`.
#[doc(hidden)]
pub trait Comp: crate::component::Component + Vertex<Stored = <Self as crate::component::Component>::Output> {}
impl<C> Comp for C where C: crate::component::Component + Vertex<Stored = <C as crate::component::Component>::Output> {}

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
