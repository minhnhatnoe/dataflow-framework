//! The flat, balanced type-level structure of all components.
//!
//! `NthT<P>::T` is the component type at component position `P` of anchor `A`,
//! found by a halving descent of the first-parent tree (binary lifting):
//! `Down<A, X, J, K>` goes from the node at plan index `X` to its descendant
//! `J`; for a long way it first goes to the ancestor of `J` halfway, then from
//! there, so the solver depth is about log2 of the graph depth, not the graph
//! depth. `FlatSpec` is the balanced 8-ary tuple type over positions, with one
//! element per component holding whatever a [`Mapper`] makes of its type
//! (the state, the output, or either one while it is being constructed).

use crate::component::Sub;
use crate::node::Vertex;
use crate::store::Mapper;
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
impl<E: Vertex, C, const I: usize> PosSpan for Sub<E, C, I> {
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
impl<E: Vertex, C, const I: usize, W: Target> PosSelect<W> for Sub<E, C, I> {
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
    type T: Vertex;
}
impl<E: Vertex, C, const I: usize, W: Target> Pick<W, 0> for Sub<E, C, I> {
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

/// The position `I` as a [`Target`].
#[doc(hidden)]
pub struct EntryPos<const I: usize>;
impl<const I: usize> Target for EntryPos<I> {
    const POS: usize = I;
}

/// The node and selector chain of the entry at position `P` in this node,
/// which holds it in field `S`.
#[doc(hidden)]
pub trait PickEdge<W: Target, const S: usize> {
    type E: Vertex;
    type Ch: crate::select::SelectChain<<Self::E as Vertex>::Stored>;
}
impl<E: Vertex, C: crate::select::SelectChain<E::Stored>, const I: usize, W: Target> PickEdge<W, 0> for Sub<E, C, I> {
    type E = E;
    type Ch = C;
}
macro_rules! pick_edge_impl {
    (($($g:ident)+) $k:tt $s:ident) => {
        impl<W: Target, $($g: PosSpan),+> PickEdge<W, $k> for ($($g,)+)
        where
            $s: PosSelect<W> + PickEdge<W, { <$s as PosSelect<W>>::POS }>,
        {
            type E = <$s as PickEdge<W, { <$s as PosSelect<W>>::POS }>>::E;
            type Ch = <$s as PickEdge<W, { <$s as PosSelect<W>>::POS }>>::Ch;
        }
    };
}
dataflow_macros::get_tuples!(pick_edge_impl);

// ---------------------------------------------------------------------------
// Down: the component type at plan index J below the one at plan index A
// ---------------------------------------------------------------------------

#[doc(hidden)]
pub struct Way<R, const A: usize, const J: usize>(std::marker::PhantomData<R>);
impl<R: Planned, const A: usize, const J: usize> Way<R, A, J> {
    pub const MID: usize = R::PLAN.mid(A, J);
    /// Both kinds of the halves in one constant: `3 * K1 + K2`.
    pub const KK: u8 = 3 * R::PLAN.kind(A, Self::MID) + R::PLAN.kind(Self::MID, J);
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

#[doc(hidden)]
pub trait CutAt<R, const A: usize, const J: usize, const M: usize> {
    type Both: Pair;
}
impl<R, const A: usize, const J: usize, const M: usize> CutAt<R, A, J, M> for () {
    type Both = (Way<R, A, M>, Way<R, M, J>);
}
#[doc(hidden)]
pub trait Pair {
    type First;
    type Second;
}
impl<X, Y> Pair for (X, Y) {
    type First = X;
    type Second = Y;
}

/// The two halves of a long way, as types: the ways to and from the middle
/// node, and the kinds of both. One impl, so one place for the constants.
#[doc(hidden)]
pub trait Split {
    type Both: Pair;
    type Ks: Pair;
}
impl<R: Planned, const A: usize, const J: usize> Split for Way<R, A, J>
where
    (): KindsOf<{ Way::<R, A, J>::KK }>,
    (): CutAt<R, A, J, { Way::<R, A, J>::MID }>,
{
    type Both = <() as CutAt<R, A, J, { Way::<R, A, J>::MID }>>::Both;
    type Ks = <() as KindsOf<{ Way::<R, A, J>::KK }>>::Both;
}
/// The kinds of both halves as a pair of types, from `3 * K1 + K2`.
#[doc(hidden)]
pub trait KindsOf<const N: u8> {
    type Both: Pair;
}
macro_rules! kinds_of {
    ($($n:literal $a:ident $b:ident)*) => { $(impl KindsOf<$n> for () { type Both = ($a, $b); })* };
}
kinds_of!(0 Same Same 1 Same Child 2 Same Far 3 Child Same 4 Child Child 5 Child Far 6 Far Same 7 Far Child 8 Far Far);

/// The node type at the end of a way `W`, by kind.
#[doc(hidden)]
pub trait DownW<W, Kd>: Vertex {
    type T: Vertex;
}
impl<C: Vertex, W> DownW<W, Same> for C {
    type T = C;
}
impl<R: Planned, C: Vertex, const A: usize, const J: usize> DownW<Way<R, A, J>, Child> for C
where
    C::Subs: PosSelect<Way<R, A, J>> + Pick<Way<R, A, J>, { <C::Subs as PosSelect<Way<R, A, J>>>::POS }>,
{
    type T = <C::Subs as Pick<Way<R, A, J>, { <C::Subs as PosSelect<Way<R, A, J>>>::POS }>>::T;
}
impl<R: Planned, C: Vertex, const A: usize, const J: usize> DownW<Way<R, A, J>, Far> for C
where
    Way<R, A, J>: Split,
    C: DownW<<<Way<R, A, J> as Split>::Both as Pair>::First, <<Way<R, A, J> as Split>::Ks as Pair>::First>,
    <C as DownW<<<Way<R, A, J> as Split>::Both as Pair>::First, <<Way<R, A, J> as Split>::Ks as Pair>::First>>::T:
        DownW<<<Way<R, A, J> as Split>::Both as Pair>::Second, <<Way<R, A, J> as Split>::Ks as Pair>::Second>,
{
    type T = <<C as DownW<<<Way<R, A, J> as Split>::Both as Pair>::First, <<Way<R, A, J> as Split>::Ks as Pair>::First>>::T as DownW<
        <<Way<R, A, J> as Split>::Both as Pair>::Second,
        <<Way<R, A, J> as Split>::Ks as Pair>::Second,
    >>::T;
}

#[doc(hidden)]
pub trait Down<R: Planned, const A: usize, const J: usize, Kd>: Vertex {
    type T: Vertex;
}
impl<R: Planned, C: Vertex + DownW<Way<R, A, J>, Kd>, const A: usize, const J: usize, Kd> Down<R, A, J, Kd> for C {
    type T = <C as DownW<Way<R, A, J>, Kd>>::T;
}

// ---------------------------------------------------------------------------
// Storage positions after the components: stored selected values (slots)
// ---------------------------------------------------------------------------
//
// The traits below carry a mapper type `Ph` that only serves to keep their blanket
// impls from matching a bound with a generic `Ph` (the same device as `EntryOf`'s
// phase): without it the solver tries the impl while checking a generic impl whose
// bound has an unevaluated constant, and recurses without end through `Down`.

/// The node and selector chain of entry `I` of the component at position `P`
/// (like `EntryOf`, but needing only the plan).
#[doc(hidden)]
pub trait EdgeAt<Ph, const P: usize, const I: usize>: Planned {
    type E: Vertex;
    type Ch: crate::select::SelectChain<<Self::E as Vertex>::Stored>;
}
macro_rules! edge_at_impl {
    ($($ph:ty),*) => {$(
        impl<R: Planned, const P: usize, const I: usize> EdgeAt<$ph, P, I> for R
        where
            R: NthT<P>,
            <R as NthT<P>>::T: crate::component::ComponentSubscription,
            <<R as NthT<P>>::T as crate::component::ComponentSubscription>::Deps:
                PosSelect<EntryPos<I>> + PickEdge<EntryPos<I>, { <<<R as NthT<P>>::T as crate::component::ComponentSubscription>::Deps as PosSelect<EntryPos<I>>>::POS }>,
        {
            type E = <<<R as NthT<P>>::T as crate::component::ComponentSubscription>::Deps as PickEdge<
                EntryPos<I>,
                { <<<R as NthT<P>>::T as crate::component::ComponentSubscription>::Deps as PosSelect<EntryPos<I>>>::POS },
            >>::E;
            type Ch = <<<R as NthT<P>>::T as crate::component::ComponentSubscription>::Deps as PickEdge<
                EntryPos<I>,
                { <<<R as NthT<P>>::T as crate::component::ComponentSubscription>::Deps as PosSelect<EntryPos<I>>>::POS },
            >>::Ch;
        }
    )*};
}
edge_at_impl!(crate::store::StateM, crate::store::OutM, crate::store::StateOptM, crate::store::OutOptM);

/// The reader and entry that name the slot at storage position `P` (after the
/// components): the first edge that stores it.
#[doc(hidden)]
pub struct SlotRep<R, const P: usize>(std::marker::PhantomData<R>);
impl<R: Planned, const P: usize> SlotRep<R, P> {
    pub const SUB: usize = R::PLAN.slot_rep(P).0;
    pub const ENTRY: usize = R::PLAN.slot_rep(P).1;
}
#[doc(hidden)]
pub trait SlotEdge<Ph> {
    type E: Vertex;
    type Ch: crate::select::SelectChain<<Self::E as Vertex>::Stored>;
}
impl<Ph, R: Planned, const P: usize> SlotEdge<Ph> for SlotRep<R, P>
where
    R: EdgeAt<Ph, { SlotRep::<R, P>::SUB }, { SlotRep::<R, P>::ENTRY }>,
{
    type E = <R as EdgeAt<Ph, { SlotRep::<R, P>::SUB }, { SlotRep::<R, P>::ENTRY }>>::E;
    type Ch = <R as EdgeAt<Ph, { SlotRep::<R, P>::SUB }, { SlotRep::<R, P>::ENTRY }>>::Ch;
}
/// The slot node at storage position `P`.
#[doc(hidden)]
pub trait SlotTy<Ph, const P: usize>: Planned {
    type T: Vertex;
}
impl<Ph, R: Planned, const P: usize> SlotTy<Ph, P> for R
where
    SlotRep<R, P>: SlotEdge<Ph>,
{
    type T = crate::node::SlotNode<<SlotRep<R, P> as SlotEdge<Ph>>::E, <SlotRep<R, P> as SlotEdge<Ph>>::Ch>;
}

/// Is storage position `P` a component or a slot? As a type, so that an
/// unevaluated constant in a generic context can't match an impl.
#[doc(hidden)]
pub struct CompPos;
#[doc(hidden)]
pub struct SlotPos;
#[doc(hidden)]
pub trait PosKindOf<const K: u8> {
    type Ty;
}
impl PosKindOf<0> for () {
    type Ty = CompPos;
}
impl PosKindOf<1> for () {
    type Ty = SlotPos;
}
#[doc(hidden)]
pub struct IsSlot<R, const P: usize>(std::marker::PhantomData<R>);
impl<R: Planned, const P: usize> IsSlot<R, P> {
    /// (`len - 2` components as in `Whole::N`: also right for a plan that failed.)
    pub const K: u8 = if P >= R::PLAN.len() - 2 { 1 } else { 0 };
}
#[doc(hidden)]
pub trait PosClass {
    type Ty;
}
impl<R: Planned, const P: usize> PosClass for IsSlot<R, P>
where
    (): PosKindOf<{ IsSlot::<R, P>::K }>,
{
    type Ty = <() as PosKindOf<{ IsSlot::<R, P>::K }>>::Ty;
}
#[doc(hidden)]
pub trait NthBy<Ph, const P: usize, K>: Planned {
    type T: Vertex;
}
impl<Ph, R: Planned, const P: usize> NthBy<Ph, P, CompPos> for R
where
    R: NthT<P>,
{
    type T = <R as NthT<P>>::T;
}
impl<Ph, R: Planned, const P: usize> NthBy<Ph, P, SlotPos> for R
where
    R: SlotTy<Ph, P>,
{
    type T = <R as SlotTy<Ph, P>>::T;
}
/// The node type at storage position `P`: the component at component position
/// `P` (see [`NthT`]), or, after the components, a stored selected value.
#[doc(hidden)]
pub trait NthS<Ph, const P: usize>: Planned {
    type T: Vertex;
}
macro_rules! nth_s_impl {
    ($($ph:ty),*) => {$(
        impl<R: Planned, const P: usize> NthS<$ph, P> for R
        where
            IsSlot<R, P>: PosClass,
            R: NthBy<$ph, P, <IsSlot<R, P> as PosClass>::Ty>,
        {
            type T = <R as NthBy<$ph, P, <IsSlot<R, P> as PosClass>::Ty>>::T;
        }
    )*};
}
nth_s_impl!(crate::store::StateM, crate::store::OutM, crate::store::StateOptM, crate::store::OutOptM);

/// Where component position `P` is: its plan index and the way from the
/// anchor. (Execution position 0 is the root, so component position `P` is
/// execution position `P + 1`.)
#[doc(hidden)]
pub struct Loc<R, const P: usize>(std::marker::PhantomData<R>);
impl<R: Planned, const P: usize> Loc<R, P> {
    pub const IDX: usize = {
        R::CHECK;
        R::PLAN.idx_at_post(P + 1)
    };
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
    type T: Vertex;
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

/// Children of an inner node of the flat storage: `ARITY` of `dataflow-macros`.
pub(crate) const ARITY: usize = dataflow_macros::arity!();

pub(crate) const fn class(lo: usize, hi: usize) -> usize {
    if hi <= lo {
        0
    } else if hi - lo <= ARITY {
        hi - lo
    } else {
        ARITY + 1
    }
}

/// Components per child of a node of width `w > 8`: 8, 64, 512, ... whichever
/// is the least with `8 * cap >= w`.
pub(crate) const fn cap(w: usize) -> usize {
    let mut c = ARITY;
    while c * ARITY < w {
        c *= ARITY;
    }
    c
}

/// Field index as a type (see `store::PathTo`).
#[doc(hidden)]
pub struct Ix<const K: usize>;

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
dataflow_macros::arity_classes!(class_of);
impl ClassOf<{ ARITY + 1 }> for () {
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

/// The piece of the flat storage over component positions `LO..HI`, of class
/// `Cl`: `()` for an empty range, otherwise the *nominal* block
/// [`Blk<R, M, LO, HI>`](Blk), whose element at position `P` is
/// `M::Of<the node at P>`.
///
/// The block is a named struct so that its type (and so every symbol and
/// query key that mentions it) stays small however many components lie below
/// it. A plain nested tuple would spell out all its descendants: the symbol
/// names of the lookup functions would then be as long as the range, which
/// made compile time and memory quadratic in the number of components.
#[doc(hidden)]
pub trait FlatSpec<M: Mapper, const LO: usize, const HI: usize, Cl>: Planned {
    type T;
}
impl<R: Planned, M: Mapper, const LO: usize, const HI: usize> FlatSpec<M, LO, HI, Leaf<0>> for R {
    type T = ();
}
macro_rules! flat_piece {
    ($($n:literal)*) => {$(
        impl<R: Planned, M: Mapper, const LO: usize, const HI: usize> FlatSpec<M, LO, HI, Leaf<$n>> for R
        where
            R: Fields<M, LO, HI>,
        {
            type T = Blk<R, M, LO, HI>;
        }
    )*};
}
dataflow_macros::arity_classes1!(flat_piece);
impl<R: Planned, M: Mapper, const LO: usize, const HI: usize> FlatSpec<M, LO, HI, Inner> for R
where
    R: Fields<M, LO, HI>,
{
    type T = Blk<R, M, LO, HI>;
}

/// The fields of a block of class `Cl`: a tuple of slots (leaf) or of the
/// child pieces (inner).
#[doc(hidden)]
pub trait BlkF<M: Mapper, const LO: usize, const HI: usize, Cl>: Planned {
    type F;
}

macro_rules! flat_leaf {
    ($n:literal; $($k:literal)*) => {
        impl<R: Planned, M: Mapper, const LO: usize, const HI: usize> BlkF<M, LO, HI, Leaf<$n>> for R
        where
            $(R: NthS<M, { LO + $k }>,)*
        {
            type F = ($(M::Of<<R as NthS<M, { LO + $k }>>::T>,)*);
        }
    };
}
dataflow_macros::leaf_idx!(flat_leaf);

macro_rules! flat_inner {
    ($($k:literal)*) => {
        impl<R: Planned, M: Mapper, const LO: usize, const HI: usize> BlkF<M, LO, HI, Inner> for R
        where
            $(Rng<LO, HI, $k>: RngTy,
              R: FlatSpec<M, { Rng::<LO, HI, $k>::L }, { Rng::<LO, HI, $k>::H }, <Rng<LO, HI, $k> as RngTy>::Cl>,)*
        {
            type F = ($(<R as FlatSpec<M, { Rng::<LO, HI, $k>::L }, { Rng::<LO, HI, $k>::H }, <Rng<LO, HI, $k> as RngTy>::Cl>>::T,)*);
        }
    };
}
dataflow_macros::inner_idx!(flat_inner);

#[doc(hidden)]
pub struct Span2<const LO: usize, const HI: usize>;
impl<const LO: usize, const HI: usize> Span2<LO, HI> {
    pub const C: usize = class(LO, HI);
}

#[doc(hidden)]
pub trait Cls2 {
    type Cl;
}
impl<const LO: usize, const HI: usize> Cls2 for Span2<LO, HI>
where
    (): ClassOf<{ Span2::<LO, HI>::C }>,
{
    type Cl = <() as ClassOf<{ Span2::<LO, HI>::C }>>::Ty;
}

/// [`BlkF`] with the class of the range resolved.
#[doc(hidden)]
pub trait Fields<M: Mapper, const LO: usize, const HI: usize>: Planned {
    type F;
}
impl<R: Planned, M: Mapper, const LO: usize, const HI: usize> Fields<M, LO, HI> for R
where
    Span2<LO, HI>: Cls2,
    R: BlkF<M, LO, HI, <Span2<LO, HI> as Cls2>::Cl>,
{
    type F = <R as BlkF<M, LO, HI, <Span2<LO, HI> as Cls2>::Cl>>::F;
}

/// A block of the flat storage: the slots of the component positions
/// `LO..HI` of pipeline `R`, as a leaf tuple of slots or a tuple of 8 child
/// pieces.
#[doc(hidden)]
pub struct Blk<R: Fields<M, LO, HI>, M: Mapper, const LO: usize, const HI: usize>(
    pub <R as Fields<M, LO, HI>>::F,
);

/// The range of all component positions of `R`.
#[doc(hidden)]
pub struct Whole<R>(std::marker::PhantomData<R>);
impl<R: Planned> Whole<R> {
    /// Components: all nodes but the root and the anchor.
    pub const N: usize = R::PLAN.len() - 2;
    pub const C: usize = class(0, Self::N);
    /// Storage positions: the components, then the stored selected values.
    pub const T: usize = Self::N + R::PLAN.n_slots();
    pub const CT: usize = class(0, Self::T);
}
#[doc(hidden)]
pub trait TotalTy {
    type Cl;
}
impl<R: Planned> TotalTy for Whole<R>
where
    (): ClassOf<{ Whole::<R>::CT }>,
{
    type Cl = <() as ClassOf<{ Whole::<R>::CT }>>::Ty;
}

#[doc(hidden)]
pub trait FlatTop<M: Mapper> {
    type Of;
}
impl<R: Planned, M: Mapper> FlatTop<M> for R
where
    Whole<R>: TotalTy,
    R: FlatSpec<M, 0, { Whole::<R>::T }, <Whole<R> as TotalTy>::Cl>,
{
    type Of = <R as FlatSpec<M, 0, { Whole::<R>::T }, <Whole<R> as TotalTy>::Cl>>::T;
}

/// The balanced 8-ary tuple of what mapper `M` makes of each component of
/// pipeline `R`, in execution order.
#[allow(dead_code)]
pub type FlatOf<R, M> = <R as FlatTop<M>>::Of;
