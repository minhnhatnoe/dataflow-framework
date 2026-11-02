//! Flat storage: balanced blocks ([`Blk`]) whose leaves are tuples of one
//! value per component position.
//!
//! The states of the components and their stored outputs are two such trees
//! of the same shape (`flat::FlatOf` with the [`StateM`] and [`OutM`]
//! mappers). Both are indexed by position, so reading or writing the element
//! at position `P` is a *constant* descent through at most four levels of
//! blocks: after inlining it is one field offset. The index of the child to
//! take at each level is computed from the block's range and `P`
//! ([`path`]).
//!
//! While the pipeline is built, the same shapes exist with every value wrapped
//! in an `Option` ([`StateOptM`], [`OutOptM`]): construction fills them in
//! execution order, each constructor reading the outputs filled so far, and
//! [`Finish`] turns them into the final tuples (this is the only place that
//! unwraps; it is part of `Pipeline::build`, not of a step).

use crate::node::Node;

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
// Access by position
// ---------------------------------------------------------------------------
//
// The path from the top block to position `P` is a few digits (`path`): at
// each level, the index of the child that holds `P`. It is walked by `Proj`
// steps, one per level, all in the body of one function (`AtPos::at`), so that
// the functions below the top are shared by every position under the same
// child instead of being instantiated once per (position, level). A path
// shorter than the maximum is padded with `Stop`, the identity.

/// The value at position `P` of a whole storage tuple (a descent from the
/// top).
#[doc(hidden)]
pub trait AtPos<V, const P: usize> {
    fn at(&self) -> &V;
    fn at_mut(&mut self) -> &mut V;
}

// The steps (`TupAt`, `Proj`) are plain `#[inline]` and only `AtPos::at` is `inline(always)`:
// LLVM's AlwaysInliner keeps every function it has inlined until the end of the pass, so its
// memory grows with the number of `alwaysinline` functions (6,114 components: LLVM stage peak
// 6.2 GB with all of them forced, 2.8 GB with the steps left to the ordinary inliner, which
// frees as it goes), while the machine code and the push time are the same.

/// Identity step: the path ended.
#[doc(hidden)]
pub struct Stop;

/// Element `K` of a tuple.
#[doc(hidden)]
pub trait TupAt<K> {
    type Out: 'static;
    fn at_k(&self) -> &Self::Out;
    fn at_k_mut(&mut self) -> &mut Self::Out;
}
macro_rules! tup_at_impl {
    (($($g:ident)+) $k:tt $s:ident) => {
        impl<$($g: 'static),+> TupAt<Ix<$k>> for ($($g,)+) {
            type Out = $s;
            #[inline]
            fn at_k(&self) -> &$s {
                &self.$k
            }
            #[inline]
            fn at_k_mut(&mut self) -> &mut $s {
                &mut self.$k
            }
        }
    };
}
dataflow_macros::get_tuples!(tup_at_impl);

/// One step down: from a block into its child `K`, or `Stop`.
#[doc(hidden)]
pub trait Proj<K> {
    type Out: 'static;
    fn proj(&self) -> &Self::Out;
    fn proj_mut(&mut self) -> &mut Self::Out;
}
impl<T: 'static> Proj<Stop> for T {
    type Out = T;
    #[inline]
    fn proj(&self) -> &T {
        self
    }
    #[inline]
    fn proj_mut(&mut self) -> &mut T {
        self
    }
}
impl<R: Fields<M, LO, HI>, M: Mapper, const LO: usize, const HI: usize, const K: usize> Proj<Ix<K>> for Blk<R, M, LO, HI>
where
    <R as Fields<M, LO, HI>>::F: TupAt<Ix<K>>,
{
    type Out = <<R as Fields<M, LO, HI>>::F as TupAt<Ix<K>>>::Out;
    #[inline]
    fn proj(&self) -> &Self::Out {
        TupAt::<Ix<K>>::at_k(&self.0)
    }
    #[inline]
    fn proj_mut(&mut self) -> &mut Self::Out {
        TupAt::<Ix<K>>::at_k_mut(&mut self.0)
    }
}

/// The digits of the path to position `p` in the block over `lo..hi`, packed
/// 8 bits each (first digit lowest); 255 after the last one, or everywhere
/// when `p` is not in the block (no `Proj` chain then reaches the value, so
/// that is a type error, not a const panic).
#[doc(hidden)]
pub const fn path(mut lo: usize, mut hi: usize, p: usize) -> usize {
    let mut out = 0usize;
    let mut i = 0;
    while i < 4 {
        let d = if p < lo || p >= hi {
            255
        } else if hi - lo <= crate::flat::ARITY {
            let d = p - lo;
            hi = lo; // the last level: the rest of the path is empty
            d
        } else {
            let c = crate::flat::cap(hi - lo);
            let k = (p - lo) / c;
            let nh = crate::flat::bound(lo, hi, k + 1);
            lo += k * c;
            hi = nh;
            k
        };
        out |= d << (8 * i);
        i += 1;
    }
    out
}
#[doc(hidden)]
pub trait DigitOf<const D: usize> {
    type Ty;
}
impl DigitOf<255> for () {
    type Ty = Stop;
}
macro_rules! digit_of {
    ($($n:literal)*) => { $(impl DigitOf<$n> for () { type Ty = Ix<$n>; })* };
}
dataflow_macros::arity_classes!(digit_of);

/// The four digit types of a packed path.
#[doc(hidden)]
pub trait Digits<const N: usize> {
    type D1;
    type D2;
    type D3;
    type D4;
}
impl<const N: usize> Digits<N> for ()
where
    (): DigitOf<{ N & 255 }>,
    (): DigitOf<{ (N >> 8) & 255 }>,
    (): DigitOf<{ (N >> 16) & 255 }>,
    (): DigitOf<{ (N >> 24) & 255 }>,
{
    type D1 = <() as DigitOf<{ N & 255 }>>::Ty;
    type D2 = <() as DigitOf<{ (N >> 8) & 255 }>>::Ty;
    type D3 = <() as DigitOf<{ (N >> 16) & 255 }>>::Ty;
    type D4 = <() as DigitOf<{ (N >> 24) & 255 }>>::Ty;
}
/// The path to position `P` in the block over `LO..HI`, as types. The
/// constant is inside an impl of `(LO, HI, P)` only, so it is evaluated once
/// per (block, position).
#[doc(hidden)]
pub struct PathTo<const LO: usize, const HI: usize, const P: usize>;
#[doc(hidden)]
pub trait PathT {
    type D1;
    type D2;
    type D3;
    type D4;
}
impl<const LO: usize, const HI: usize, const P: usize> PathT for PathTo<LO, HI, P>
where
    (): Digits<{ path(LO, HI, P) }>,
{
    type D1 = <() as Digits<{ path(LO, HI, P) }>>::D1;
    type D2 = <() as Digits<{ path(LO, HI, P) }>>::D2;
    type D3 = <() as Digits<{ path(LO, HI, P) }>>::D3;
    type D4 = <() as Digits<{ path(LO, HI, P) }>>::D4;
}

type Out<T, K> = <T as Proj<K>>::Out;

impl<R, M, V, D1, D2, D3, D4, const LO: usize, const HI: usize, const P: usize> AtPos<V, P> for Blk<R, M, LO, HI>
where
    R: Fields<M, LO, HI>,
    M: Mapper,
    PathTo<LO, HI, P>: PathT<D1 = D1, D2 = D2, D3 = D3, D4 = D4>,
    Blk<R, M, LO, HI>: Proj<D1>,
    Out<Blk<R, M, LO, HI>, D1>: Proj<D2>,
    Out<Out<Blk<R, M, LO, HI>, D1>, D2>: Proj<D3>,
    Out<Out<Out<Blk<R, M, LO, HI>, D1>, D2>, D3>: Proj<D4, Out = V>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn at(&self) -> &V {
        let a = Proj::<D1>::proj(self);
        let b = Proj::<D2>::proj(a);
        let c = Proj::<D3>::proj(b);
        Proj::<D4>::proj(c)
    }
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn at_mut(&mut self) -> &mut V {
        let a = Proj::<D1>::proj_mut(self);
        let b = Proj::<D2>::proj_mut(a);
        let c = Proj::<D3>::proj_mut(b);
        Proj::<D4>::proj_mut(c)
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
impl<V> Empty for Option<V> {
    const EMPTY: Self = None;
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
impl<V> Finish for Option<V> {
    type Done = V;
    fn finish(self) -> V {
        self.expect("every component is constructed in execution order")
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

use crate::flat::{Blk, Fields, Ix};

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
