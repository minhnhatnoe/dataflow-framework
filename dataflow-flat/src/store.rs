//! Flat storage: balanced tuples whose elements are `Slot<P, V>`, one per
//! component position `P`.
//!
//! The states of the components and their stored outputs are two such tuples
//! of the same shape (`flat::FlatOf` with the [`StateM`] and [`OutM`]
//! mappers). Both are indexed by position, so reading or writing the element
//! at position `P` is a *constant* descent through at most `log8(N)` levels of
//! tuple fields: after inlining it is one field offset. How the descent knows
//! which field to take: every node of the tree knows the range of positions
//! below it ([`Span`], from the `Slot`s' const parameter), and [`SelP`] picks
//! the field whose range holds `P`.
//!
//! While the pipeline is built, the same shapes exist with every value wrapped
//! in an `Option` ([`StateOptM`], [`OutOptM`]): construction fills them in
//! execution order, each constructor reading the outputs filled so far, and
//! [`Finish`] turns them into the final tuples (this is the only place that
//! unwraps; it is part of `Pipeline::build`, not of a step).

use crate::node::Node;

/// An element of the flat storage: the value for the component at position
/// `P`. Zero cost.
#[doc(hidden)]
pub struct Slot<const P: usize, V>(pub V);

// ---------------------------------------------------------------------------
// What is stored
// ---------------------------------------------------------------------------

/// What a storage tuple holds for a node.
#[doc(hidden)]
pub trait Mapper: 'static {
    type Of<N: Node>: 'static;
}
/// The component itself.
#[doc(hidden)]
pub struct StateM;
impl Mapper for StateM {
    type Of<N: Node> = N;
}
/// The component's last output.
#[doc(hidden)]
pub struct OutM;
impl Mapper for OutM {
    type Of<N: Node> = N::Output;
}
/// The component, while it is not constructed yet.
#[doc(hidden)]
pub struct StateOptM;
impl Mapper for StateOptM {
    type Of<N: Node> = Option<N>;
}
/// The output, while it is not produced yet.
#[doc(hidden)]
pub struct OutOptM;
impl Mapper for OutOptM {
    type Of<N: Node> = Option<N::Output>;
}

// ---------------------------------------------------------------------------
// Which positions are below a node
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

/// The positions `LO..HI` of the slots below a node. The empty tuple has the
/// empty range (`LO = usize::MAX`, `HI = 0`), which holds no position.
#[doc(hidden)]
pub trait Span {
    const LO: usize;
    const HI: usize;
}
impl<const P: usize, V> Span for Slot<P, V> {
    const LO: usize = P;
    const HI: usize = P + 1;
}
macro_rules! span_impl {
    ($($g:ident)*) => {
        impl<$($g: Span),*> Span for ($($g,)*) {
            const LO: usize = min_all(&[$(<$g as Span>::LO),*]);
            const HI: usize = max_all(&[$(<$g as Span>::HI),*]);
        }
    };
}
dataflow_macros::tuple_sizes!(span_impl);

/// Which field of a node holds position `P`.
#[doc(hidden)]
pub trait SelP<const P: usize> {
    const POS: usize;
}
impl<const P: usize, const Q: usize, V> SelP<P> for Slot<Q, V> {
    const POS: usize = 0;
}
macro_rules! selp_impl {
    ($($g:ident $i:tt)+) => {
        impl<const P: usize, $($g: Span),+> SelP<P> for ($($g,)+) {
            const POS: usize = {
                $(if <$g as Span>::LO <= P && P < <$g as Span>::HI { $i } else)+
                { panic!("position not in this node") }
            };
        }
    };
}
dataflow_macros::select_tuples!(selp_impl);

// ---------------------------------------------------------------------------
// Access by position
// ---------------------------------------------------------------------------

/// The value `V` at position `P`, which a node holds in its field `S`.
#[doc(hidden)]
pub trait Access<V, const P: usize, const S: usize> {
    fn get(&self) -> &V;
    fn get_mut(&mut self) -> &mut V;
}
impl<const Q: usize, const P: usize, V> Access<V, P, 0> for Slot<Q, V> {
    #[inline]
    fn get(&self) -> &V {
        &self.0
    }
    #[inline]
    fn get_mut(&mut self) -> &mut V {
        &mut self.0
    }
}
macro_rules! access_impl {
    (($($g:ident)+) $k:tt $s:ident) => {
        impl<V, const P: usize, $($g: Span),+> Access<V, P, $k> for ($($g,)+)
        where
            $s: SelP<P> + Access<V, P, { <$s as SelP<P>>::POS }>,
        {
            #[inline]
            fn get(&self) -> &V {
                Access::<V, P, { <$s as SelP<P>>::POS }>::get(&self.$k)
            }
            #[inline]
            fn get_mut(&mut self) -> &mut V {
                Access::<V, P, { <$s as SelP<P>>::POS }>::get_mut(&mut self.$k)
            }
        }
    };
}
dataflow_macros::get_tuples!(access_impl);

/// The value `V` at position `P` of a whole storage tuple (a descent from the
/// top).
#[doc(hidden)]
pub trait AtPos<V, const P: usize> {
    fn at(&self) -> &V;
    fn at_mut(&mut self) -> &mut V;
}
impl<V, const P: usize, T> AtPos<V, P> for T
where
    T: Access<V, P, 0>,
{
    #[inline]
    fn at(&self) -> &V {
        Access::<V, P, 0>::get(self)
    }
    #[inline]
    fn at_mut(&mut self) -> &mut V {
        Access::<V, P, 0>::get_mut(self)
    }
}

// ---------------------------------------------------------------------------
// Construction: empty tuples to fill, and finishing them
// ---------------------------------------------------------------------------

/// A tuple of empty `Option` slots.
#[doc(hidden)]
pub trait Empty: Sized {
    /// A constant, not a fn: no mono item per slot.
    const EMPTY: Self;
}
impl<const P: usize, V> Empty for Slot<P, Option<V>> {
    const EMPTY: Self = Slot(None);
}
macro_rules! empty_impl {
    ($($g:ident)*) => {
        impl<$($g: Empty),*> Empty for ($($g,)*) {
            const EMPTY: Self = ($(<$g as Empty>::EMPTY,)*);
        }
    };
}
dataflow_macros::tuple_sizes!(empty_impl);

/// Unwrap every slot of a filled tuple of `Option` slots.
#[doc(hidden)]
pub trait Finish {
    type Done;
    fn finish(self) -> Self::Done;
}
impl<const P: usize, V> Finish for Slot<P, Option<V>> {
    type Done = Slot<P, V>;
    fn finish(self) -> Slot<P, V> {
        Slot(self.0.expect("every component is constructed in execution order"))
    }
}
impl Finish for () {
    type Done = ();
    fn finish(self) {}
}
macro_rules! finish_impl {
    ($($g:ident $i:tt)+) => {
        impl<$($g: Finish),+> Finish for ($($g,)+) {
            type Done = ($($g::Done,)+);
            fn finish(self) -> Self::Done {
                ($(self.$i.finish(),)+)
            }
        }
    };
}
dataflow_macros::select_tuples!(finish_impl);

// ---------------------------------------------------------------------------
// Blocks: the nominal nodes of the tree (see `flat::Blk`)
// ---------------------------------------------------------------------------

use crate::flat::{Blk, FieldIdx, FieldSel, Fields, Ix};

impl<R: Fields<M, LO, HI>, M: Mapper, const LO: usize, const HI: usize> Span for Blk<R, M, LO, HI> {
    const LO: usize = LO;
    const HI: usize = HI;
}
impl<R: Fields<M, LO, HI>, M: Mapper, const LO: usize, const HI: usize, const P: usize> SelP<P>
    for Blk<R, M, LO, HI>
{
    const POS: usize = 0;
}
impl<R: Fields<M, LO, HI>, M: Mapper, V, const LO: usize, const HI: usize, const P: usize> Access<V, P, 0>
    for Blk<R, M, LO, HI>
where
    FieldIdx<LO, HI, P>: FieldSel,
    <R as Fields<M, LO, HI>>::F: Field<V, P, <FieldIdx<LO, HI, P> as FieldSel>::Ix>,
{
    #[inline]
    fn get(&self) -> &V {
        Field::<V, P, <FieldIdx<LO, HI, P> as FieldSel>::Ix>::get(&self.0)
    }
    #[inline]
    fn get_mut(&mut self) -> &mut V {
        Field::<V, P, <FieldIdx<LO, HI, P> as FieldSel>::Ix>::get_mut(&mut self.0)
    }
}

/// Field `K` of a block's tuple; its element is a slot or a block (both are
/// `Access<.., 0>`). Unlike `Access` for a tuple, it needs no `Span` of the
/// siblings: the field index comes from the block's range alone.
#[doc(hidden)]
pub trait Field<V, const P: usize, I> {
    fn get(&self) -> &V;
    fn get_mut(&mut self) -> &mut V;
}
macro_rules! field_impl {
    (($($g:ident)+) $k:tt $s:ident) => {
        impl<V, const P: usize, $($g),+> Field<V, P, Ix<$k>> for ($($g,)+)
        where
            $s: Access<V, P, 0>,
        {
            #[inline]
            fn get(&self) -> &V {
                Access::<V, P, 0>::get(&self.$k)
            }
            #[inline]
            fn get_mut(&mut self) -> &mut V {
                Access::<V, P, 0>::get_mut(&mut self.$k)
            }
        }
    };
}
dataflow_macros::get_tuples!(field_impl);
impl<R: Fields<M, LO, HI>, M: Mapper, const LO: usize, const HI: usize> Empty for Blk<R, M, LO, HI>
where
    <R as Fields<M, LO, HI>>::F: Empty,
{
    const EMPTY: Self = Blk(<<R as Fields<M, LO, HI>>::F as Empty>::EMPTY);
}

/// A mapper for a value still being constructed, and the mapper it becomes.
#[doc(hidden)]
pub trait OptMapper: Mapper {
    type Done: Mapper;
}
impl OptMapper for StateOptM {
    type Done = StateM;
}
impl OptMapper for OutOptM {
    type Done = OutM;
}
impl<R, M: OptMapper, const LO: usize, const HI: usize> Finish for Blk<R, M, LO, HI>
where
    R: Fields<M, LO, HI> + Fields<M::Done, LO, HI>,
    <R as Fields<M, LO, HI>>::F: Finish<Done = <R as Fields<M::Done, LO, HI>>::F>,
{
    type Done = Blk<R, M::Done, LO, HI>;
    // A function per block: construction runs once, and LLVM must not see one function the size of the graph.
    #[inline(never)]
    fn finish(self) -> Self::Done {
        Blk(self.0.finish())
    }
}
