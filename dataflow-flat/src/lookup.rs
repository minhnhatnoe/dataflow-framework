//! Finished outputs and typed access into them.
//!
//! `run` is a fold: walking a subtree returns its outputs by value as a
//! [`Done`]. A later component reads the outputs of finished ones through a
//! *context* ([`Context`]): a type-level list of finished pieces whose value
//! is a tuple of shared references into the enclosing stack frames. A piece is
//! one finished component with its subtree ([`Done`]) or one finished node of a
//! subscription tree ([`Node`]), so the context grows with the *depth* of the
//! subscription tree, not with the number of subscriptions.
//!
//! Lookups are monomorphized, not searched. Every finished piece covers a
//! contiguous range of the plan ([`Span`]), so the plan decides, at every step,
//! which entry holds the wanted component (`HERE` for a context, [`Select`] for
//! a piece). The decision is a const parameter of the trait, so each step is one
//! function with one impl per answer, and there are no steps for paths not
//! taken. After inlining, an input is a field access at a known path.
//!
//! * [`Get`] finds a component's output inside one finished piece.
//! * [`Find`] finds it in a context.
//! * [`GatherInputs`] does that for every entry of a subscription tree.

use std::marker::PhantomData;

use crate::component::{Component, DepList, Sub};
use crate::identity::At;
use crate::plan::Planned;
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
    #[inline(always)]
    pub(crate) fn into_output(self) -> C::Output {
        let Done { deps, out } = self;
        drop(deps);
        out
    }
}

/// A context entry for a finished node of a subscription tree (a tuple of
/// [`Done`]s, `()`s and finished child nodes). The value is a reference to it.
/// The node of the component being walked, whose own `Done` does not exist yet,
/// is such an entry too.
#[doc(hidden)]
pub struct Node<R, X>(PhantomData<(R, X)>);

// ---------------------------------------------------------------------------
// Spans: which part of the plan a finished piece covers
// ---------------------------------------------------------------------------

const EMPTY_LO: usize = usize::MAX;

const fn min_all(xs: &[usize]) -> usize {
    let mut m = usize::MAX;
    let mut i = 0;
    while i < xs.len() {
        if xs[i] < m {
            m = xs[i];
        }
        i += 1;
    }
    m
}

const fn max_all(xs: &[usize]) -> usize {
    let mut m = 0;
    let mut i = 0;
    while i < xs.len() {
        if xs[i] > m {
            m = xs[i];
        }
        i += 1;
    }
    m
}

/// The plan indices `LO..HI` covered by the components stored in a finished
/// piece. The components stored under one node are consecutive in first-reached
/// order (each stored subtree is followed directly by the next one), so a piece
/// covers one contiguous range; an empty piece has `LO > HI`.
#[doc(hidden)]
pub trait Span<R> {
    const LO: usize;
    const HI: usize;
}
impl<R: Planned, C: Placed<R>> Span<R> for Done<R, C> {
    const LO: usize = R::PLAN.range_of(At::<R, C>::INDEX).0;
    const HI: usize = R::PLAN.range_of(At::<R, C>::INDEX).1;
}

macro_rules! span_impl {
    ($($g:ident)*) => {
        impl<R, $($g: Span<R>),*> Span<R> for ($($g,)*) {
            const LO: usize = min_all(&[$(<$g as Span<R>>::LO),*]);
            const HI: usize = max_all(&[$(<$g as Span<R>>::HI),*]);
        }
    };
}
span_impl!();
span_impl!(A);
span_impl!(A B);
span_impl!(A B C);
span_impl!(A B C D);
span_impl!(A B C D E);

// ---------------------------------------------------------------------------
// Get: inside one finished piece
// ---------------------------------------------------------------------------

/// Which part of a finished piece holds `H`: for a [`Done`], 0 if it is the
/// component itself and 1 if it is among its dependencies; for a tuple node,
/// the index of the field (entry or child) that holds it.
#[doc(hidden)]
pub trait Select<R, H: Component> {
    const POS: usize;
}
impl<R: Planned, C: Placed<R>, H: Component> Select<R, H> for Done<R, C> {
    const POS: usize = if At::<R, C>::INDEX == At::<R, H>::INDEX { 0 } else { 1 };
}

macro_rules! select_impl {
    ($($g:ident $i:tt)+) => {
        impl<R: Planned, H: Component, $($g: Span<R>),+> Select<R, H> for ($($g,)+) {
            const POS: usize = {
                let h = At::<R, H>::INDEX;
                $(if <$g as Span<R>>::LO <= h && h < <$g as Span<R>>::HI { $i } else)+
                { panic!("component not in this node") }
            };
        }
    };
}
select_impl!(A 0);
select_impl!(A 0 B 1);
select_impl!(A 0 B 1 C 2);
select_impl!(A 0 B 1 C 2 D 3);
select_impl!(A 0 B 1 C 2 D 3 E 4);

/// `H`'s output inside this finished piece, which holds it at position `P`
/// (see [`Select`]).
#[doc(hidden)]
pub trait Get<R, H: Component, const P: usize> {
    fn get(&self) -> &H::Output;
}
/// The component itself.
impl<R: Planned, H: Placed<R>> Get<R, H, 0> for Done<R, H> {
    #[inline(always)]
    fn get(&self) -> &H::Output {
        &self.out
    }
}
/// Somewhere in the component's dependencies.
impl<R: Planned, C: Placed<R>, H: Component> Get<R, H, 1> for Done<R, C>
where
    DepsDone<R, C>: Select<R, H> + Get<R, H, { <DepsDone<R, C> as Select<R, H>>::POS }>,
{
    #[inline(always)]
    fn get(&self) -> &H::Output {
        self.deps.get()
    }
}

/// In field `$k` (of type `$s`) of a tuple node.
macro_rules! get_impl {
    (($($g:ident)+) $k:tt $s:ident) => {
        impl<R: Planned, H: Component, $($g: Span<R>),+> Get<R, H, $k> for ($($g,)+)
        where
            $s: Select<R, H> + Get<R, H, { <$s as Select<R, H>>::POS }>,
        {
            #[inline(always)]
            fn get(&self) -> &H::Output {
                self.$k.get()
            }
        }
    };
}
get_impl!((A) 0 A);
get_impl!((A B) 0 A);
get_impl!((A B) 1 B);
get_impl!((A B C) 0 A);
get_impl!((A B C) 1 B);
get_impl!((A B C) 2 C);
get_impl!((A B C D) 0 A);
get_impl!((A B C D) 1 B);
get_impl!((A B C D) 2 C);
get_impl!((A B C D) 3 D);
get_impl!((A B C D E) 0 A);
get_impl!((A B C D E) 1 B);
get_impl!((A B C D E) 2 C);
get_impl!((A B C D E) 3 D);
get_impl!((A B C D E) 4 E);

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

/// Does the head of context `L` hold `H`? Picks the branch of `Find`.
#[doc(hidden)]
pub struct TopHolds<R, L, H>(PhantomData<(R, L, H)>);
impl<R: Planned, L: Context, H: Component> TopHolds<R, L, H> {
    pub const YES: bool = {
        let h = At::<R, H>::INDEX;
        L::LO <= h && h < L::HI
    };
}

/// A type-level list of finished pieces; its value is references. `LO..HI` is
/// the span of the head entry (empty for `()`).
#[doc(hidden)]
pub trait Context: 'static {
    const LO: usize;
    const HI: usize;
    type Refs<'a>: Copy;
    /// `Refs` is covariant, but the compiler can't see through the GAT.
    fn shorten<'s, 'a: 's>(r: Self::Refs<'a>) -> Self::Refs<'s>;
}
impl Context for () {
    const LO: usize = EMPTY_LO;
    const HI: usize = 0;
    type Refs<'a> = ();
    #[inline(always)]
    fn shorten<'s, 'a: 's>(_: ()) {}
}
impl<R: Planned, X: Span<R> + 'static, T: Context> Context for (Node<R, X>, T) {
    const LO: usize = <X as Span<R>>::LO;
    const HI: usize = <X as Span<R>>::HI;
    type Refs<'a> = (&'a X, T::Refs<'a>);
    #[inline(always)]
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
/// In a finished node or subtree (a context entry is always a `Node`).
impl<R: Planned, H: Component, X: Span<R> + 'static, T: Context> Find<R, H, true> for (Node<R, X>, T)
where
    X: Select<R, H> + Get<R, H, { <X as Select<R, H>>::POS }>,
{
    #[inline(always)]
    fn find<'a>(r: Self::Refs<'a>) -> &'a H::Output {
        r.0.get()
    }
}
/// Further along the context.
impl<R: Planned, H: Component, X: Span<R> + 'static, T: Context> Find<R, H, false> for (Node<R, X>, T)
where
    T: Find<R, H, { TopHolds::<R, T, H>::YES }>,
{
    #[inline(always)]
    fn find<'a>(r: Self::Refs<'a>) -> &'a H::Output {
        T::find(r.1)
    }
}

// ---------------------------------------------------------------------------
// Gather: the inputs of one component
// ---------------------------------------------------------------------------

/// The references `update` receives, found in context `T`: one `Find` per
/// entry of the subscription tree.
#[doc(hidden)]
pub trait GatherInputs<R, T: Context>: DepList {
    fn gather<'a>(r: T::Refs<'a>) -> Self::Inputs<'a>;
}
macro_rules! gather_impl {
    ([$($E:ident $I:ident)*] [$($K:ident)*]) => {
        impl<R: Planned, T: Context, $($E: Component, const $I: usize,)* $($K: GatherInputs<R, T>,)*>
            GatherInputs<R, T> for ($(Sub<$E, $I>,)* $($K,)*)
        where
            $(T: Find<R, $E, { TopHolds::<R, T, $E>::YES }>,)*
        {
            #[inline(always)]
            #[allow(unused_variables)]
            fn gather<'a>(r: T::Refs<'a>) -> Self::Inputs<'a> {
                (
                    $(<T as Find<R, $E, { TopHolds::<R, T, $E>::YES }>>::find(r),)*
                    $(<$K as GatherInputs<R, T>>::gather(r),)*
                )
            }
        }
    };
}
gather_impl!([][]);
gather_impl!([A IA][]);
gather_impl!([A IA B IB][]);
gather_impl!([A IA B IB C IC][]);
gather_impl!([A IA B IB C IC D ID][]);
gather_impl!([A IA B IB C IC D ID][K0 K1 K2 K3]);

const _: () = assert!(
    crate::ARITY == 4,
    "the node-shape impls in this file are written for ARITY = 4; update them"
);
