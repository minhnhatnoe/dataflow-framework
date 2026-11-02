//! The split descent: `&mut` to the state at position `P` plus shared references to everything
//! to its left, so that a component can be updated while it reads the *state* of a handle
//! component that was constructed and run before it (a parent is always at a smaller position).
//!
//! `AtPos::at_mut` borrows the whole storage tree mutably, so the parent's state could not be
//! read at the same time. `SplitAt<P>` descends the same constant path and, at each level,
//! splits the block's tuple at the digit of `P`: the child becomes `&mut`, the siblings before it
//! become `&`. The result is the child and four *contexts* (one per level of the path, `()` for
//! the levels the path does not use). A parent at position `Q < P` lies in a left sibling of the
//! level where the two paths first differ (`diverge`); `StateAt<P, Q, V>` picks it from that
//! context and goes on down by shared `Proj` steps. All of it is constant offsets after inlining,
//! and safe: only `&mut` splitting of tuples by pattern.
//!
//! Only components with a handle parent use it (see `drive`).

use crate::flat::{Blk, Fields, Ix};
use crate::store::{Mapper, PathT, PathTo, Proj, Stop};

/// One step of the split: `&mut` to child `K`, and shared references to the children before it.
#[doc(hidden)]
pub trait Split<K> {
    /// The siblings before child `K`. (No `where Self: 'a`: every storage type is `'static`.)
    type Left<'a>
    where
        Self: 'a;
    type Child: 'static;
    fn split<'a>(&'a mut self) -> (Self::Left<'a>, &'a mut Self::Child);
}
impl<T: 'static> Split<Stop> for T {
    type Left<'a> = ()
    where
        Self: 'a;
    type Child = T;
    #[inline]
    fn split<'a>(&'a mut self) -> ((), &'a mut T) {
        ((), self)
    }
}
macro_rules! split_impl {
    (($($g:ident)+) [$($b:ident)*] $k:tt $s:ident) => {
        impl<$($g: 'static),+> Split<Ix<$k>> for ($($g,)+) {
            type Left<'a> = ($(&'a $b,)*) where Self: 'a;
            type Child = $s;
            #[inline]
            #[allow(non_snake_case, unused_variables)]
            fn split<'a>(&'a mut self) -> (Self::Left<'a>, &'a mut $s) {
                let ($($b,)* $s, ..) = self;
                (($(&*$b,)*), $s)
            }
        }
    };
}
dataflow_macros::split_tuples!(split_impl);
impl<R: Fields<M, LO, HI> + 'static, M: Mapper, const LO: usize, const HI: usize, const K: usize> Split<Ix<K>> for Blk<R, M, LO, HI>
where
    <R as Fields<M, LO, HI>>::F: Split<Ix<K>>,
{
    type Left<'a> = <<R as Fields<M, LO, HI>>::F as Split<Ix<K>>>::Left<'a> where Self: 'a;
    type Child = <<R as Fields<M, LO, HI>>::F as Split<Ix<K>>>::Child;
    #[inline]
    fn split<'a>(&'a mut self) -> (Self::Left<'a>, &'a mut Self::Child) {
        Split::<Ix<K>>::split(&mut self.0)
    }
}

/// Element `K` of a tuple of shared references (a left context).
#[doc(hidden)]
pub trait Take<K> {
    type Elem: 'static;
    fn take(&self) -> &Self::Elem;
}
macro_rules! take_impl {
    (($($g:ident)+) $k:tt $s:ident) => {
        impl<'a, $($g: 'static),+> Take<Ix<$k>> for ($(&'a $g,)+) {
            type Elem = $s;
            #[inline]
            fn take(&self) -> &$s {
                self.$k
            }
        }
    };
}
dataflow_macros::get_tuples!(take_impl);

type Ch<T, K> = <T as Split<K>>::Child;
type Lf<'a, T, K> = <T as Split<K>>::Left<'a>;

/// The split of a whole storage tree at position `P`: the four contexts and the child.
#[doc(hidden)]
pub trait SplitAt<const P: usize> {
    type Ctx<'a>
    where
        Self: 'a;
    type Child: 'static;
    fn split_at<'a>(&'a mut self) -> (Self::Ctx<'a>, &'a mut Self::Child);
}
impl<R, M, D1, D2, D3, D4, const LO: usize, const HI: usize, const P: usize> SplitAt<P> for Blk<R, M, LO, HI>
where
    R: Fields<M, LO, HI> + 'static,
    M: Mapper,
    PathTo<LO, HI, P>: PathT<D1 = D1, D2 = D2, D3 = D3, D4 = D4>,
    Blk<R, M, LO, HI>: Split<D1>,
    Ch<Blk<R, M, LO, HI>, D1>: Split<D2>,
    Ch<Ch<Blk<R, M, LO, HI>, D1>, D2>: Split<D3>,
    Ch<Ch<Ch<Blk<R, M, LO, HI>, D1>, D2>, D3>: Split<D4>,
{
    type Ctx<'a> = (
        Lf<'a, Blk<R, M, LO, HI>, D1>,
        Lf<'a, Ch<Blk<R, M, LO, HI>, D1>, D2>,
        Lf<'a, Ch<Ch<Blk<R, M, LO, HI>, D1>, D2>, D3>,
        Lf<'a, Ch<Ch<Ch<Blk<R, M, LO, HI>, D1>, D2>, D3>, D4>,
    ) where Self: 'a;
    type Child = Ch<Ch<Ch<Ch<Blk<R, M, LO, HI>, D1>, D2>, D3>, D4>;
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn split_at<'a>(&'a mut self) -> (Self::Ctx<'a>, &'a mut Self::Child) {
        let (l1, c) = Split::<D1>::split(self);
        let (l2, c) = Split::<D2>::split(c);
        let (l3, c) = Split::<D3>::split(c);
        let (l4, c) = Split::<D4>::split(c);
        ((l1, l2, l3, l4), c)
    }
}

/// The level (0 to 3) where two packed paths first differ.
#[doc(hidden)]
pub const fn diverge(a: usize, b: usize) -> usize {
    let mut i = 0;
    while i < 3 {
        if (a >> (8 * i)) & 255 != (b >> (8 * i)) & 255 {
            return i;
        }
        i += 1;
    }
    3
}

#[doc(hidden)]
pub struct Lv0;
#[doc(hidden)]
pub struct Lv1;
#[doc(hidden)]
pub struct Lv2;
#[doc(hidden)]
pub struct Lv3;
#[doc(hidden)]
pub trait LevelOf<const L: usize> {
    type Ty;
}
impl LevelOf<0> for () {
    type Ty = Lv0;
}
impl LevelOf<1> for () {
    type Ty = Lv1;
}
impl LevelOf<2> for () {
    type Ty = Lv2;
}
impl LevelOf<3> for () {
    type Ty = Lv3;
}

type Pj<T, K> = <T as Proj<K>>::Out;

/// The level where the paths to `P` and `Q` in the block over `LO..HI` diverge, as a type.
#[doc(hidden)]
pub struct DivAt<const LO: usize, const HI: usize, const P: usize, const Q: usize>;
#[doc(hidden)]
pub trait DivTy {
    type Ty;
}
impl<const LO: usize, const HI: usize, const P: usize, const Q: usize> DivTy for DivAt<LO, HI, P, Q>
where
    (): LevelOf<{ diverge(crate::store::path(LO, HI, P), crate::store::path(LO, HI, Q)) }>,
{
    type Ty = <() as LevelOf<{ diverge(crate::store::path(LO, HI, P), crate::store::path(LO, HI, Q)) }>>::Ty;
}

/// Pick the value of the path `D1..D4` out of the context of level `L`, then down.
#[doc(hidden)]
pub trait CtxPick<L, D1, D2, D3, D4, V> {
    fn pick(&self) -> &V;
}
impl<C1, C2, C3, C4, D1, D2, D3, D4, V: 'static> CtxPick<Lv0, D1, D2, D3, D4, V> for (C1, C2, C3, C4)
where
    C1: Take<D1>,
    <C1 as Take<D1>>::Elem: Proj<D2>,
    Pj<<C1 as Take<D1>>::Elem, D2>: Proj<D3>,
    Pj<Pj<<C1 as Take<D1>>::Elem, D2>, D3>: Proj<D4, Out = V>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn pick(&self) -> &V {
        let a = Take::<D1>::take(&self.0);
        let b = Proj::<D2>::proj(a);
        let c = Proj::<D3>::proj(b);
        Proj::<D4>::proj(c)
    }
}
impl<C1, C2, C3, C4, D1, D2, D3, D4, V: 'static> CtxPick<Lv1, D1, D2, D3, D4, V> for (C1, C2, C3, C4)
where
    C2: Take<D2>,
    <C2 as Take<D2>>::Elem: Proj<D3>,
    Pj<<C2 as Take<D2>>::Elem, D3>: Proj<D4, Out = V>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn pick(&self) -> &V {
        let b = Take::<D2>::take(&self.1);
        let c = Proj::<D3>::proj(b);
        Proj::<D4>::proj(c)
    }
}
impl<C1, C2, C3, C4, D1, D2, D3, D4, V: 'static> CtxPick<Lv2, D1, D2, D3, D4, V> for (C1, C2, C3, C4)
where
    C3: Take<D3>,
    <C3 as Take<D3>>::Elem: Proj<D4, Out = V>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn pick(&self) -> &V {
        let c = Take::<D3>::take(&self.2);
        Proj::<D4>::proj(c)
    }
}
impl<C1, C2, C3, C4, D1, D2, D3, D4, V: 'static> CtxPick<Lv3, D1, D2, D3, D4, V> for (C1, C2, C3, C4)
where
    C4: Take<D4, Elem = V>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn pick(&self) -> &V {
        Take::<D4>::take(&self.3)
    }
}

/// The state at position `Q` (a parent), read from the context of the split at position `P`.
#[doc(hidden)]
pub trait StateAt<const P: usize, const Q: usize, V>: SplitAt<P> {
    fn state<'c, 'a>(cx: &'c Self::Ctx<'a>) -> &'c V;
}
impl<R, M, V, D1, D2, D3, D4, const LO: usize, const HI: usize, const P: usize, const Q: usize> StateAt<P, Q, V> for Blk<R, M, LO, HI>
where
    R: Fields<M, LO, HI>,
    M: Mapper,
    Blk<R, M, LO, HI>: SplitAt<P>,
    PathTo<LO, HI, Q>: PathT<D1 = D1, D2 = D2, D3 = D3, D4 = D4>,
    DivAt<LO, HI, P, Q>: DivTy,
    for<'a> <Blk<R, M, LO, HI> as SplitAt<P>>::Ctx<'a>: CtxPick<<DivAt<LO, HI, P, Q> as DivTy>::Ty, D1, D2, D3, D4, V>,
{
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    fn state<'c, 'a>(cx: &'c <Blk<R, M, LO, HI> as SplitAt<P>>::Ctx<'a>) -> &'c V {
        CtxPick::<<DivAt<LO, HI, P, Q> as DivTy>::Ty, D1, D2, D3, D4, V>::pick(cx)
    }
}
