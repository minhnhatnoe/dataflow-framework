// Library side, variant B: only `type_info` (no internal `rustc_attrs`); the reflection search
// lives in the associated const's body. Direct fields first; nested structs (one level) only
// when no direct field has type `Output`. The ONLY unsafe block is in `view`.
#![feature(type_info)]
#![feature(const_trait_impl, const_cmp)]
#![allow(incomplete_features)]
use core::mem::type_info::{self, Type, TypeKind};

pub trait Component {
    type Input<'a> where Self: 'a;
    type Output;
    fn update<'s>(&'s mut self, input: Self::Input<'_>) -> &'s Self::Output;
}
pub struct Cell<C> { c: C }
impl<C: Component + 'static> Cell<C> {
    /// Byte offset of the unique field of type `C::Output` inside `C`, found at compile time.
    pub const OFF: usize = {
        let want = type_info::of::<C::Output>();
        let ty = type_info::of::<C>();
        if ty == want { 0 } else {                                   // `Output = Self`
            assert!(matches!(Type::of::<C>().kind, TypeKind::Struct), "a component must be a struct (or set `Output = Self`)");
            let (mut n, mut off) = (0usize, 0usize);
            let fields = ty.fields(0);
            let mut i = 0;
            while i < fields { let f = ty.field(0, i); if f.type_id() == want { n += 1; off = f.offset(); } i += 1; }
            match n { 1 => { assert!(off % core::mem::align_of::<C::Output>() == 0 && core::mem::align_of::<C>() >= core::mem::align_of::<C::Output>(), "the `Output` field is not aligned (packed struct)"); off }
                0 => panic!("the component has no field of type `Output`: update cannot return a reference into it"),
                _ => panic!("the component has more than one field of type `Output`: the output location is ambiguous") }
        }
    };
    pub fn new(c: C) -> Self { Cell { c } }
    #[inline(always)]
    pub fn run(&mut self, i: C::Input<'_>) {
        let returned = self.c.update(i) as *const C::Output;
        debug_assert!(core::ptr::eq(returned, self.view()), "update must return a reference to the component's own `Output` field");
    }
    /// The last output, whether or not `update` ran this step: a plain field projection.
    #[inline(always)]
    pub fn view(&self) -> &C::Output {
        // SAFETY: `OFF` is the compile-time offset of a field of type `C::Output` inside `C`
        // (reflection checked the type, uniqueness and that `C` is a struct); `&self.c` is a live
        // shared borrow, so the projected place is in bounds, aligned, initialised and borrowed
        // for the returned lifetime: this is `&self.c.<field>` spelled without the name.
        unsafe { &*(&self.c as *const C as *const u8).add(Self::OFF).cast::<C::Output>() }
    }
}
