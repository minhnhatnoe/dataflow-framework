//! Version 2 of "foo": a component `Foo` that counts its constructions,
//! updates and drops. The two versions have the same code and the same
//! `type_name`; only the counters tell them apart.

use std::sync::atomic::{AtomicUsize, Ordering};

use dataflow_flat::{Component, ComponentSubscription};

pub static NEW: AtomicUsize = AtomicUsize::new(0);
pub static UPDATE: AtomicUsize = AtomicUsize::new(0);
pub static DROP: AtomicUsize = AtomicUsize::new(0);

pub const VERSION: u32 = 2;

#[derive(ComponentSubscription)]
pub struct Foo;

impl Component for Foo {
    type Config = ();
    type Output = u32;
    fn new((): ()) -> Self {
        NEW.fetch_add(1, Ordering::SeqCst);
        Foo
    }
    fn update(&mut self, _: Self::Input<'_>) -> u32 {
        UPDATE.fetch_add(1, Ordering::SeqCst);
        VERSION
    }
}

impl Drop for Foo {
    fn drop(&mut self) {
        DROP.fetch_add(1, Ordering::SeqCst);
    }
}
