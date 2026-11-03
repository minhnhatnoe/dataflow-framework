//! Asking again: the conditions the push program cannot decide by position.
//!
//! Whether an edge fired is never stored. A component's condition is a set of *terms* (the plan,
//! `Plan::cond_comp`), each a set of *decisions*: the match of one selector, with the ones before
//! it in its chain, on the output of one node. Inside a `Switch` arm the decisions of the switches
//! above are held, and what the arm runs is known. What is left (a decision on a guard, a
//! condition that spans several switches, a handler between a conditional parent and a
//! conditional child) is a *guard* of the program node: any of its terms, all of whose decisions
//! are true. A decision is recomputed from the pushed root and the stored outputs by
//! [`ChainAtom`], which is a pure function of them (selectors must be pure: see
//! [`Select`](crate::Select)). The terms hold the decisions of the condition of every ancestor, so
//! the stored output of a component that did not run in this step, which can still match, does not
//! make a term true.
//!
//! Guards are plain code at the place they are used (`||` of `&&`), so the compiler keeps what it
//! can in registers; a push with no guard has none of this.

use std::marker::PhantomData;

use crate::drive::{Ch, EntryOf, Rg, RgCl};
use crate::flat::{Inner, Leaf};
use crate::gather::{ChainAtom, Layout, Source};
use crate::plan::Planned;

/// Decision of the plan of `A` with the facts the test needs: where its types come from (the
/// component position `P` and entry position `I` of the edge that first walked it), its depth `J`
/// in the chain and its key `K`. They are parameters, not looked up in the impl, so that the
/// impl's bounds are on plain parameters and the proof is one trait less deep.
#[doc(hidden)]
pub struct Dec<A, Ph, const P: usize, const I: usize, const J: usize, const K: u32>(PhantomData<(A, Ph)>);

/// Is the decision true of the current outputs?
#[doc(hidden)]
pub trait AtomTest<A: Layout, Ph> {
    fn test(root: &A::Root, states: &A::States, outs: &A::Outs) -> bool;
}
impl<A: Layout, Ph, const P: usize, const I: usize, const J: usize, const K: u32> AtomTest<A, Ph> for Dec<A, Ph, P, I, J, K>
where
    A: EntryOf<Ph, P, I>,
    <A as EntryOf<Ph, P, I>>::E: Source<A>,
    <A as EntryOf<Ph, P, I>>::Ch: ChainAtom<<A as EntryOf<Ph, P, I>>::E, A, J, K>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn test(root: &A::Root, states: &A::States, outs: &A::Outs) -> bool {
        <<A as EntryOf<Ph, P, I>>::Ch as ChainAtom<<A as EntryOf<Ph, P, I>>::E, A, J, K>>::atom(root, states, outs)
    }
}

/// The terms of guard `G` as a range.
#[doc(hidden)]
pub struct TermsOf<A, const G: usize>(PhantomData<A>);
impl<A: Planned, const G: usize> Rg for TermsOf<A, G> {
    const LO: usize = A::PLAN.gd_first(G);
    const HI: usize = A::PLAN.gd_first(G) + A::PLAN.gd_count(G);
}
/// Term `K` of the range `Q` (a range of terms), as a range of decisions.
#[doc(hidden)]
pub struct TermAt<A, Q, const K: usize>(PhantomData<(A, Q)>);
impl<A: Planned, Q: Rg, const K: usize> Rg for TermAt<A, Q, K> {
    const LO: usize = A::PLAN.gt_first(Q::LO + K);
    const HI: usize = A::PLAN.gt_first(Q::LO + K) + A::PLAN.gt_count(Q::LO + K);
}
/// The decision at entry `K` of the range `Q` (a range of the decision list).
#[doc(hidden)]
pub struct AtomAt<A, Q, const K: usize>(PhantomData<(A, Q)>);
impl<A: Planned, Q: Rg, const K: usize> AtomAt<A, Q, K> {
    pub const D: usize = A::PLAN.ga_dec(Q::LO + K);
    pub const P: usize = A::PLAN.dec_pos(Self::D);
    pub const I: usize = A::PLAN.dec_entry(Self::D);
    pub const J: usize = A::PLAN.dec_depth(Self::D);
    pub const KEY: u32 = A::PLAN.dec_key(Self::D);
}

/// All the decisions of a term (a range of the decision list) are true.
#[doc(hidden)]
pub trait AllOf<A: Layout, Ph, Cl>: Rg {
    fn all(root: &A::Root, states: &A::States, outs: &A::Outs) -> bool;
}
macro_rules! all_leaf {
    ($n:literal; $($k:literal)*) => {
        impl<A: Layout, Ph, Q: Rg> AllOf<A, Ph, Leaf<$n>> for Q
        where
            $(Dec<A, Ph, { AtomAt::<A, Q, $k>::P }, { AtomAt::<A, Q, $k>::I }, { AtomAt::<A, Q, $k>::J }, { AtomAt::<A, Q, $k>::KEY }>: AtomTest<A, Ph>,)*
        {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            #[allow(unused_variables)]
            fn all(root: &A::Root, states: &A::States, outs: &A::Outs) -> bool {
                true $(&& <Dec<A, Ph, { AtomAt::<A, Q, $k>::P }, { AtomAt::<A, Q, $k>::I }, { AtomAt::<A, Q, $k>::J }, { AtomAt::<A, Q, $k>::KEY }> as AtomTest<A, Ph>>::test(root, states, outs))*
            }
        }
    };
}
dataflow_macros::leaf_idx!(all_leaf);
macro_rules! all_inner {
    ($($k:literal)*) => {
        impl<A: Layout, Ph, Q: Rg> AllOf<A, Ph, Inner> for Q
        where
            $(Ch<Q, $k>: RgCl,
              Ch<Q, $k>: AllOf<A, Ph, <Ch<Q, $k> as RgCl>::Cl>,)*
        {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            fn all(root: &A::Root, states: &A::States, outs: &A::Outs) -> bool {
                true $(&& <Ch<Q, $k> as AllOf<A, Ph, <Ch<Q, $k> as RgCl>::Cl>>::all(root, states, outs))*
            }
        }
    };
}
dataflow_macros::inner_idx!(all_inner);

/// Some term of a guard (a range of terms) has all its decisions true.
#[doc(hidden)]
pub trait AnyOf<A: Layout, Ph, Cl>: Rg {
    fn any(root: &A::Root, states: &A::States, outs: &A::Outs) -> bool;
}
macro_rules! any_leaf {
    ($n:literal; $($k:literal)*) => {
        impl<A: Layout, Ph, Q: Rg> AnyOf<A, Ph, Leaf<$n>> for Q
        where
            $(TermAt<A, Q, $k>: RgCl,
              TermAt<A, Q, $k>: AllOf<A, Ph, <TermAt<A, Q, $k> as RgCl>::Cl>,)*
        {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            #[allow(unused_variables)]
            fn any(root: &A::Root, states: &A::States, outs: &A::Outs) -> bool {
                false $(|| <TermAt<A, Q, $k> as AllOf<A, Ph, <TermAt<A, Q, $k> as RgCl>::Cl>>::all(root, states, outs))*
            }
        }
    };
}
dataflow_macros::leaf_idx!(any_leaf);
macro_rules! any_inner {
    ($($k:literal)*) => {
        impl<A: Layout, Ph, Q: Rg> AnyOf<A, Ph, Inner> for Q
        where
            $(Ch<Q, $k>: RgCl,
              Ch<Q, $k>: AnyOf<A, Ph, <Ch<Q, $k> as RgCl>::Cl>,)*
        {
            #[cfg_attr(not(debug_assertions), inline(always))]
            #[cfg_attr(debug_assertions, inline)]
            fn any(root: &A::Root, states: &A::States, outs: &A::Outs) -> bool {
                false $(|| <Ch<Q, $k> as AnyOf<A, Ph, <Ch<Q, $k> as RgCl>::Cl>>::any(root, states, outs))*
            }
        }
    };
}
dataflow_macros::inner_idx!(any_inner);

/// Guard `G` of the plan of `A`.
#[doc(hidden)]
pub struct Gd<A, Ph, const G: usize>(PhantomData<(A, Ph)>);
/// Does the guard hold of the current outputs?
#[doc(hidden)]
pub trait Holds<A: Layout, Ph> {
    fn holds(root: &A::Root, states: &A::States, outs: &A::Outs) -> bool;
}
impl<A: Layout, Ph, const G: usize> Holds<A, Ph> for Gd<A, Ph, G>
where
    TermsOf<A, G>: RgCl,
    TermsOf<A, G>: AnyOf<A, Ph, <TermsOf<A, G> as RgCl>::Cl>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn holds(root: &A::Root, states: &A::States, outs: &A::Outs) -> bool {
        <TermsOf<A, G> as AnyOf<A, Ph, <TermsOf<A, G> as RgCl>::Cl>>::any(root, states, outs)
    }
}

/// An optional guard: `HAS` false is "no guard" (`G` is then `usize::MAX` and never looked up).
#[doc(hidden)]
pub trait GSel<A: Layout, Ph, const HAS: bool, const G: usize> {
    fn holds(root: &A::Root, states: &A::States, outs: &A::Outs) -> bool;
}
impl<A: Layout, Ph, const G: usize> GSel<A, Ph, false, G> for () {
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn holds(_: &A::Root, _: &A::States, _: &A::Outs) -> bool {
        true
    }
}
impl<A: Layout, Ph, const G: usize> GSel<A, Ph, true, G> for ()
where
    TermsOf<A, G>: RgCl,
    TermsOf<A, G>: AnyOf<A, Ph, <TermsOf<A, G> as RgCl>::Cl>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn holds(root: &A::Root, states: &A::States, outs: &A::Outs) -> bool {
        <TermsOf<A, G> as AnyOf<A, Ph, <TermsOf<A, G> as RgCl>::Cl>>::any(root, states, outs)
    }
}
