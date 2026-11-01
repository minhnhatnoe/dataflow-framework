//! The flat, balanced type-level structure of all components.
//!
//! `NthT<P>::T` is the component type at component position `P` of anchor `A`,
//! found by a halving descent of the first-parent tree (binary lifting):
//! `Down<A, X, J, K>` goes from the node at plan index `X` to its descendant
//! `J`; for a long way it first goes to the ancestor of `J` halfway, then from
//! there, so the solver depth is about log2 of the graph depth, not the graph
//! depth. `FlatSpec` is the balanced 8-ary tuple type over positions, with one
//! `Slot` per component holding whatever a [`Mapper`] makes of its type
//! (the state, the output, or either one while it is being constructed).

use crate::component::Sub;
use crate::node::Node;
use crate::store::{Mapper, Slot};
use crate::plan::Planned;

// ---------------------------------------------------------------------------
// Entries of a subscription tree by position
// ---------------------------------------------------------------------------

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

/// The positions `LO..HI` of the entries `Sub<_, _, I>` below a node.
#[doc(hidden)]
pub trait PosSpan {
    const LO: usize;
    const HI: usize;
}
impl<E: Node, C, const I: usize> PosSpan for Sub<E, C, I> {
    const LO: usize = I;
    const HI: usize = I + 1;
}
macro_rules! pos_span_impl {
    ($($g:ident)*) => {
        impl<$($g: PosSpan),*> PosSpan for ($($g,)*) {
            const LO: usize = min_all(&[$(<$g as PosSpan>::LO),*]);
            const HI: usize = max_all(&[$(<$g as PosSpan>::HI),*]);
        }
    };
}
dataflow_macros::tuple_sizes!(pos_span_impl);

#[doc(hidden)]
pub trait PosSelect<W: Target> {
    const POS: usize;
}
impl<E: Node, C, const I: usize, W: Target> PosSelect<W> for Sub<E, C, I> {
    const POS: usize = 0;
}
macro_rules! pos_select_impl {
    ($($g:ident $i:tt)+) => {
        impl<W: Target, $($g: PosSpan),+> PosSelect<W> for ($($g,)+) {
            const POS: usize = {
                let p = W::POS;
                $(if <$g as PosSpan>::LO <= p && p < <$g as PosSpan>::HI { $i } else)+
                { panic!("position not in this node") }
            };
        }
    };
}
dataflow_macros::select_tuples!(pos_select_impl);

/// The component of the entry at position `P` in this node, which holds it in
/// field `S`.
#[doc(hidden)]
pub trait Pick<W: Target, const S: usize> {
    type T: Node;
}
impl<E: Node, C, const I: usize, W: Target> Pick<W, 0> for Sub<E, C, I> {
    type T = E;
}
macro_rules! pick_impl {
    (($($g:ident)+) $k:tt $s:ident) => {
        impl<W: Target, $($g: PosSpan),+> Pick<W, $k> for ($($g,)+)
        where
            $s: PosSelect<W> + Pick<W, { <$s as PosSelect<W>>::POS }>,
        {
            type T = <$s as Pick<W, { <$s as PosSelect<W>>::POS }>>::T;
        }
    };
}
dataflow_macros::get_tuples!(pick_impl);

// ---------------------------------------------------------------------------
// Down: the component type at plan index J below the one at plan index A
// ---------------------------------------------------------------------------

#[doc(hidden)]
pub struct Way<R, const A: usize, const J: usize>(std::marker::PhantomData<R>);
impl<R: Planned, const A: usize, const J: usize> Way<R, A, J> {
    pub const MID: usize = R::PLAN.mid(A, J);
    pub const K1: u8 = R::PLAN.kind(A, Self::MID);
    pub const K2: u8 = R::PLAN.kind(Self::MID, J);
}
/// The position of the entry that first reached `J`, as a constant of a type.
#[doc(hidden)]
pub trait Target {
    const POS: usize;
}
impl<R: Planned, const A: usize, const J: usize> Target for Way<R, A, J> {
    const POS: usize = R::PLAN.first_pos_of(J) as usize;
}

/// The kind of way as a type, so that an unevaluated kind in a generic context
/// can't match an impl.
#[doc(hidden)]
pub struct Same;
#[doc(hidden)]
pub struct Child;
#[doc(hidden)]
pub struct Far;
#[doc(hidden)]
pub trait KindOf<const K: u8> {
    type Ty;
}
impl KindOf<0> for () {
    type Ty = Same;
}
impl KindOf<1> for () {
    type Ty = Child;
}
impl KindOf<2> for () {
    type Ty = Far;
}

/// The kinds of the two halves of a long way, as types.
#[doc(hidden)]
pub trait Route {
    type K1;
    type K2;
}
impl<R: Planned, const A: usize, const J: usize> Route for Way<R, A, J>
where
    (): KindOf<{ Way::<R, A, J>::K1 }> + KindOf<{ Way::<R, A, J>::K2 }>,
{
    type K1 = <() as KindOf<{ Way::<R, A, J>::K1 }>>::Ty;
    type K2 = <() as KindOf<{ Way::<R, A, J>::K2 }>>::Ty;
}

#[doc(hidden)]
pub trait Down<R: Planned, const A: usize, const J: usize, Kd>: Node {
    type T: Node;
}
impl<R: Planned, C: Node, const A: usize, const J: usize> Down<R, A, J, Same> for C {
    type T = C;
}
impl<R: Planned, C: Node, const A: usize, const J: usize> Down<R, A, J, Child> for C
where
    C::Subs: PosSelect<Way<R, A, J>> + Pick<Way<R, A, J>, { <C::Subs as PosSelect<Way<R, A, J>>>::POS }>,
{
    type T = <C::Subs as Pick<Way<R, A, J>, { <C::Subs as PosSelect<Way<R, A, J>>>::POS }>>::T;
}
/// The first half of a long way.
#[doc(hidden)]
pub trait Step1<R: Planned, const A: usize, const J: usize>: Node {
    type X: Node;
}
impl<R: Planned, C: Node, const A: usize, const J: usize> Step1<R, A, J> for C
where
    Way<R, A, J>: Route,
    C: Down<R, A, { Way::<R, A, J>::MID }, <Way<R, A, J> as Route>::K1>,
{
    type X = <C as Down<R, A, { Way::<R, A, J>::MID }, <Way<R, A, J> as Route>::K1>>::T;
}
impl<R: Planned, C: Node, const A: usize, const J: usize> Down<R, A, J, Far> for C
where
    Way<R, A, J>: Route,
    C: Step1<R, A, J>,
    <C as Step1<R, A, J>>::X: Down<R, { Way::<R, A, J>::MID }, J, <Way<R, A, J> as Route>::K2>,
{
    type T = <<C as Step1<R, A, J>>::X as Down<R, { Way::<R, A, J>::MID }, J, <Way<R, A, J> as Route>::K2>>::T;
}

/// Where component position `P` is: its plan index and the way from the
/// anchor. (Execution position 0 is the root, so component position `P` is
/// execution position `P + 1`.)
#[doc(hidden)]
pub struct Loc<R, const P: usize>(std::marker::PhantomData<R>);
impl<R: Planned, const P: usize> Loc<R, P> {
    pub const IDX: usize = R::PLAN.idx_at_post(P + 1);
    pub const KIND: u8 = R::PLAN.kind(0, Self::IDX);
}
#[doc(hidden)]
pub trait LocKind {
    type Kd;
}
impl<R: Planned, const P: usize> LocKind for Loc<R, P>
where
    (): KindOf<{ Loc::<R, P>::KIND }>,
{
    type Kd = <() as KindOf<{ Loc::<R, P>::KIND }>>::Ty;
}

/// The node type at component position `P`.
#[doc(hidden)]
pub trait NthT<const P: usize>: Planned {
    type T: Node;
}
impl<R: Planned, const P: usize> NthT<P> for R
where
    Loc<R, P>: LocKind,
    R: Down<R, 0, { Loc::<R, P>::IDX }, <Loc<R, P> as LocKind>::Kd>,
{
    type T = <R as Down<R, 0, { Loc::<R, P>::IDX }, <Loc<R, P> as LocKind>::Kd>>::T;
}

// ---------------------------------------------------------------------------
// The balanced 8-ary type over component positions
// ---------------------------------------------------------------------------

pub(crate) const fn class(lo: usize, hi: usize) -> usize {
    if hi <= lo {
        0
    } else if hi - lo <= 8 {
        hi - lo
    } else {
        9
    }
}

/// Components per child of a node of width `w > 8`: 8, 64, 512, ... whichever
/// is the least with `8 * cap >= w`.
const fn cap(w: usize) -> usize {
    let mut c = 8;
    while c * 8 < w {
        c *= 8;
    }
    c
}

pub(crate) const fn bound(lo: usize, hi: usize, k: usize) -> usize {
    let b = lo + k * cap(hi - lo);
    if b < hi {
        b
    } else {
        hi
    }
}

#[doc(hidden)]
pub struct Rng<const LO: usize, const HI: usize, const K: usize>;
impl<const LO: usize, const HI: usize, const K: usize> Rng<LO, HI, K> {
    pub const L: usize = bound(LO, HI, K);
    pub const H: usize = bound(LO, HI, K + 1);
    pub const C: usize = class(Self::L, Self::H);
}

/// Class of a range as a type (see `KindOf`): a leaf of `N` elements, or an
/// inner node of 8 children.
#[doc(hidden)]
pub struct Leaf<const N: usize>;
#[doc(hidden)]
pub struct Inner;
#[doc(hidden)]
pub trait ClassOf<const C: usize> {
    type Ty;
}
macro_rules! class_of {
    ($($n:literal)*) => { $(impl ClassOf<$n> for () { type Ty = Leaf<$n>; })* };
}
class_of!(0 1 2 3 4 5 6 7 8);
impl ClassOf<9> for () {
    type Ty = Inner;
}

#[doc(hidden)]
pub trait RngTy {
    type Cl;
}
impl<const LO: usize, const HI: usize, const K: usize> RngTy for Rng<LO, HI, K>
where
    (): ClassOf<{ Rng::<LO, HI, K>::C }>,
{
    type Cl = <() as ClassOf<{ Rng::<LO, HI, K>::C }>>::Ty;
}

/// The balanced tuple over component positions `LO..HI`, of class `Cl`, whose
/// element at position `P` is `Slot<P, M::Of<the node at P>>`.
#[doc(hidden)]
pub trait FlatSpec<M: Mapper, const LO: usize, const HI: usize, Cl>: Planned {
    type T;
}

macro_rules! flat_leaf {
    ($n:literal; $($k:literal)*) => {
        impl<R: Planned, M: Mapper, const LO: usize, const HI: usize> FlatSpec<M, LO, HI, Leaf<$n>> for R
        where
            $(R: NthT<{ LO + $k }>,)*
        {
            type T = ($(Slot<{ LO + $k }, M::Of<<R as NthT<{ LO + $k }>>::T>>,)*);
        }
    };
}
dataflow_macros::leaf_idx!(flat_leaf);

macro_rules! flat_inner {
    ($($k:literal)*) => {
        impl<R: Planned, M: Mapper, const LO: usize, const HI: usize> FlatSpec<M, LO, HI, Inner> for R
        where
            $(Rng<LO, HI, $k>: RngTy,
              R: FlatSpec<M, { Rng::<LO, HI, $k>::L }, { Rng::<LO, HI, $k>::H }, <Rng<LO, HI, $k> as RngTy>::Cl>,)*
        {
            type T = ($(<R as FlatSpec<M, { Rng::<LO, HI, $k>::L }, { Rng::<LO, HI, $k>::H }, <Rng<LO, HI, $k> as RngTy>::Cl>>::T,)*);
        }
    };
}
dataflow_macros::inner_idx!(flat_inner);

/// The range of all component positions of `R`.
#[doc(hidden)]
pub struct Whole<R>(std::marker::PhantomData<R>);
impl<R: Planned> Whole<R> {
    /// Components: all nodes but the root and the anchor.
    pub const N: usize = R::PLAN.len() - 2;
    pub const C: usize = class(0, Self::N);
}
#[doc(hidden)]
pub trait WholeTy {
    type Cl;
}
impl<R: Planned> WholeTy for Whole<R>
where
    (): ClassOf<{ Whole::<R>::C }>,
{
    type Cl = <() as ClassOf<{ Whole::<R>::C }>>::Ty;
}

#[doc(hidden)]
pub trait FlatTop<M: Mapper> {
    type Of;
}
impl<R: Planned, M: Mapper> FlatTop<M> for R
where
    Whole<R>: WholeTy,
    R: FlatSpec<M, 0, { Whole::<R>::N }, <Whole<R> as WholeTy>::Cl>,
{
    type Of = <R as FlatSpec<M, 0, { Whole::<R>::N }, <Whole<R> as WholeTy>::Cl>>::T;
}

/// The balanced 8-ary tuple of what mapper `M` makes of each component of
/// pipeline `R`, in execution order.
#[allow(dead_code)]
pub type FlatOf<R, M> = <R as FlatTop<M>>::Of;
