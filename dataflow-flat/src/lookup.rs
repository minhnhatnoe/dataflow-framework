//! Finished outputs and typed access into them.
//!
//! `run` is a fold: walking a subtree returns its outputs by value as a
//! [`Done`]. A later component reads the outputs of finished ones through a
//! *context* ([`Context`]): a type-level list of finished pieces whose value
//! is a tuple of shared references into the enclosing stack frames.
//!
//! Lookups are monomorphized, not searched. The plan decides, at every step,
//! whether the wanted component is in the head of a list or further on, and the
//! decision is a const parameter of the trait (`HERE`), so each step is one
//! function with one impl per answer, and there are no steps for paths not
//! taken. After inlining, an input is a field access at a known path.
//!
//! * [`Get`] finds a component's output inside one finished piece
//!   (`Done` or a tuple of them).
//! * [`Find`] finds it in a context.
//! * [`GatherInputs`] does that for every entry of a dependency list.

use std::marker::PhantomData;

use crate::component::{Component, DepList};
use crate::plan::{Described, Planned, ROOT};
use crate::storage::{DepsDone, Placed};

// ---------------------------------------------------------------------------
// Finished outputs
// ---------------------------------------------------------------------------

/// The outputs of a finished subtree: `C`'s own and its first-reached deps'.
#[doc(hidden)]
pub struct Done<R: Planned, C: Placed<R>> {
    pub(crate) deps: DepsDone<R, C>,
    pub(crate) out: C::Output,
}

impl<R: Planned, C: Placed<R>> Done<R, C> {
    /// Keep the output, drop the finished outputs of the dependencies.
    #[inline]
    pub(crate) fn into_output(self) -> C::Output {
        let Done { deps, out } = self;
        drop(deps);
        out
    }
}

/// The component being walked, whose dependencies are finished but not yet
/// wrapped in a `Done`. As a context entry its value is a reference to the
/// tuple of its finished dependencies.
#[doc(hidden)]
pub struct Pending<R, C>(PhantomData<(R, C)>);

// ---------------------------------------------------------------------------
// Which way to go
// ---------------------------------------------------------------------------

/// The id of the component at the head of a tuple of finished pieces (storage
/// tuples and contexts alike); `NO_TOP` for `()` heads and the end.
#[doc(hidden)]
pub trait Top {
    const ID: u64;
}
const NO_TOP: u64 = ROOT;
impl Top for () {
    const ID: u64 = NO_TOP;
}
impl<B> Top for ((), B) {
    const ID: u64 = NO_TOP;
}
impl<R: Planned, C: Placed<R>, B> Top for (Done<R, C>, B) {
    const ID: u64 = <C as Described>::ID;
}
impl<R: Planned, C: Component, B> Top for (Pending<R, C>, B) {
    const ID: u64 = <C as Described>::ID;
}

/// Does the head of tuple `L` hold `H`? Picks the branch of `Get`/`Find`.
#[doc(hidden)]
pub struct TopHolds<R, L, H>(PhantomData<(R, L, H)>);
impl<R: Planned, L: Top, H: Component> TopHolds<R, L, H> {
    pub const YES: bool = L::ID != NO_TOP && R::PLAN.holds(L::ID, <H as Described>::ID);
}

/// Is component `C` the component `H`? Picks the branch of `Get` at a node.
#[doc(hidden)]
pub struct IsSame<C, H>(PhantomData<(C, H)>);
impl<C: Component, H: Component> IsSame<C, H> {
    pub const YES: bool = <C as Described>::ID == <H as Described>::ID;
}

// ---------------------------------------------------------------------------
// Get: inside one finished piece
// ---------------------------------------------------------------------------

/// `H`'s output inside this finished storage. `HERE` says whether `H` is this
/// node (or the head of this tuple) rather than further on.
#[doc(hidden)]
pub trait Get<R, H: Component, const HERE: bool> {
    fn get(&self) -> &H::Output;
}
/// The node itself.
impl<R: Planned, H: Placed<R>> Get<R, H, true> for Done<R, H> {
    #[inline]
    fn get(&self) -> &H::Output {
        &self.out
    }
}
/// Somewhere in the node's dependencies.
impl<R: Planned, C: Placed<R>, H: Component> Get<R, H, false> for Done<R, C>
where
    DepsDone<R, C>: Top + Get<R, H, { TopHolds::<R, DepsDone<R, C>, H>::YES }>,
{
    #[inline]
    fn get(&self) -> &H::Output {
        self.deps.get()
    }
}
/// In the head of a tuple.
impl<R: Planned, H: Component, C: Placed<R>, B> Get<R, H, true> for (Done<R, C>, B)
where
    Done<R, C>: Get<R, H, { IsSame::<C, H>::YES }>,
{
    #[inline]
    fn get(&self) -> &H::Output {
        self.0.get()
    }
}
/// Past the head of a tuple.
impl<R: Planned, H: Component, A, B: Top> Get<R, H, false> for (A, B)
where
    B: Get<R, H, { TopHolds::<R, B, H>::YES }>,
{
    #[inline]
    fn get(&self) -> &H::Output {
        self.1.get()
    }
}

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

/// A type-level list of finished storage pieces; its value is references.
#[doc(hidden)]
pub trait Context: Top + 'static {
    type Refs<'a>: Copy;
    /// `Refs` is covariant, but the compiler can't see through the GAT.
    fn shorten<'s, 'a: 's>(r: Self::Refs<'a>) -> Self::Refs<'s>;
}
impl Context for () {
    type Refs<'a> = ();
    #[inline]
    fn shorten<'s, 'a: 's>(_: ()) {}
}
impl<R: Planned, C: Placed<R>, T: Context> Context for (Done<R, C>, T) {
    type Refs<'a> = (&'a Done<R, C>, T::Refs<'a>);
    #[inline]
    fn shorten<'s, 'a: 's>(r: Self::Refs<'a>) -> Self::Refs<'s> {
        (r.0, T::shorten(r.1))
    }
}
impl<R: Planned, C: Placed<R>, T: Context> Context for (Pending<R, C>, T) {
    type Refs<'a> = (&'a DepsDone<R, C>, T::Refs<'a>);
    #[inline]
    fn shorten<'s, 'a: 's>(r: Self::Refs<'a>) -> Self::Refs<'s> {
        (r.0, T::shorten(r.1))
    }
}

// ---------------------------------------------------------------------------
// Find: in a context
// ---------------------------------------------------------------------------

/// `H`'s output, found in the context; `HERE` says whether it is in the head
/// entry rather than further on.
#[doc(hidden)]
pub trait Find<R, H: Component, const HERE: bool>: Context {
    fn find<'a>(r: Self::Refs<'a>) -> &'a H::Output;
}
/// In a finished subtree.
impl<R: Planned, H: Component, C: Placed<R>, T: Context> Find<R, H, true> for (Done<R, C>, T)
where
    Done<R, C>: Get<R, H, { IsSame::<C, H>::YES }>,
{
    #[inline]
    fn find<'a>(r: Self::Refs<'a>) -> &'a H::Output {
        r.0.get()
    }
}
/// In the finished dependencies of the component being walked.
impl<R: Planned, H: Component, C: Placed<R>, T: Context> Find<R, H, true> for (Pending<R, C>, T)
where
    DepsDone<R, C>: Top + Get<R, H, { TopHolds::<R, DepsDone<R, C>, H>::YES }>,
{
    #[inline]
    fn find<'a>(r: Self::Refs<'a>) -> &'a H::Output {
        r.0.get()
    }
}
/// Further along the context.
impl<R: Planned, H: Component, C: Placed<R>, T: Context> Find<R, H, false> for (Done<R, C>, T)
where
    T: Find<R, H, { TopHolds::<R, T, H>::YES }>,
{
    #[inline]
    fn find<'a>(r: Self::Refs<'a>) -> &'a H::Output {
        T::find(r.1)
    }
}
impl<R: Planned, H: Component, C: Placed<R>, T: Context> Find<R, H, false> for (Pending<R, C>, T)
where
    T: Find<R, H, { TopHolds::<R, T, H>::YES }>,
{
    #[inline]
    fn find<'a>(r: Self::Refs<'a>) -> &'a H::Output {
        T::find(r.1)
    }
}

// ---------------------------------------------------------------------------
// Gather: the inputs of one component
// ---------------------------------------------------------------------------

/// The references `update` receives, found in context `T`: one `Find` per
/// entry of the dependency list.
#[doc(hidden)]
pub trait GatherInputs<R, T: Context>: DepList {
    fn gather<'a>(r: T::Refs<'a>) -> Self::Inputs<'a>;
}
impl<R, T: Context> GatherInputs<R, T> for () {
    #[inline]
    fn gather<'a>(_: T::Refs<'a>) -> Self::Inputs<'a> {}
}
impl<R: Planned, T: Context, H: Component, Rest: GatherInputs<R, T>> GatherInputs<R, T> for (H, Rest)
where
    T: Find<R, H, { TopHolds::<R, T, H>::YES }>,
{
    #[inline]
    fn gather<'a>(r: T::Refs<'a>) -> Self::Inputs<'a> {
        (T::find(r), Rest::gather(r))
    }
}
