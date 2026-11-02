//! Prototype of the safe split descent: `&mut` to one position of a 16-ary tuple tree plus shared
//! references to everything to its left, with no `unsafe`. Stable Rust only.
#![forbid(unsafe_code)]

pub struct Ix<const K: usize>;

/// Shared element `K` of a tuple (the existing `TupAt` / `Proj`).
pub trait At<K> {
    type Out;
    fn at(&self) -> &Self::Out;
    fn at_mut(&mut self) -> &mut Self::Out;
}

/// One step of the split: `&mut` to child `K`, and shared references to the children before it.
pub trait Split<K> {
    type Left<'a>
    where
        Self: 'a;
    type Child;
    fn split<'a>(&'a mut self) -> (Self::Left<'a>, &'a mut Self::Child);
}

/// Element `K` of a tuple of shared references (the left context): the reference itself.
pub trait Take<K> {
    type Out;
    fn take(&self) -> Self::Out;
}

include!("tuples.rs");

/// A component whose output is a view computed from its state and a stored handle.
pub trait HandleComponent: Sized {
    type Handle: Copy;
    type Output<'a>
    where
        Self: 'a;
    fn output<'a>(&'a self, h: &'a Self::Handle) -> Self::Output<'a>;
}
pub mod variants;
