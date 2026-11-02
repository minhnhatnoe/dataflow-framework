// Q2 probe: one `Component` trait with associated-type defaults. Does an existing by-value impl
// (`type Output = u64; fn update -> u64`, nothing else) keep compiling, and can the trait give a
// default body for `output`?  rustc +nightly --crate-type lib default_ty.rs
#![feature(associated_type_defaults)]
pub trait Component: Sized {
    type Output: 'static;
    type Handle: 'static = Self::Output;
    type View<'a>: 'a = &'a Self::Output where Self: 'a;
    fn update(&mut self) -> Self::Handle;
    // default body: by-value components have nothing to write
    fn output<'a>(&'a self, h: &'a Self::Handle) -> Self::View<'a> { h }
}
pub struct Old;
impl Component for Old { type Output = u64; fn update(&mut self) -> u64 { 1 } }
