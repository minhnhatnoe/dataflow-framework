//! Version 2 of "foo": a component `Foo<R>` (subscribed to the root `R`)
//! that counts its constructions, updates and drops. The two versions have the
//! same code and the same `type_name` (for the same `R`); only the counters
//! tell them apart.

use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

use dataflow_flat::{Component, ComponentSubscription, Root};

pub static NEW: AtomicUsize = AtomicUsize::new(0);
pub static UPDATE: AtomicUsize = AtomicUsize::new(0);
pub static DROP: AtomicUsize = AtomicUsize::new(0);

pub const VERSION: u32 = 2;

#[derive(ComponentSubscription)]
#[subscribe(root: R)]
pub struct Foo<R: Root>(PhantomData<R>);

impl<R: Root> Component for Foo<R> {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        NEW.fetch_add(1, Ordering::SeqCst);
        (Foo(PhantomData), VERSION)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u32 {
        UPDATE.fetch_add(1, Ordering::SeqCst);
        VERSION
    }
}

impl<R: Root> Drop for Foo<R> {
    fn drop(&mut self) {
        DROP.fetch_add(1, Ordering::SeqCst);
    }
}
