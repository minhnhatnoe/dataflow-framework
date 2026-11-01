//! Generic user code over `Pipeline<R, S>`: one nameable bound, `S: Sinks<R>`,
//! and no `generic_const_exprs` feature gate in the user crate (this file has
//! none).
use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline, Root, Sinks};
use std::sync::atomic::{AtomicU32, Ordering::SeqCst};

pub fn drive<R, S: Sinks<R>>(p: &mut Pipeline<R, S>, steps: impl IntoIterator<Item = R>) {
    for r in steps {
        p.push(r);
    }
}

pub fn count<R, S: Sinks<R>>(p: &Pipeline<R, S>) -> usize {
    p.len()
}

/// Build in generic code too.
pub fn make<R, S: Sinks<R>>() -> Pipeline<R, S> {
    Pipeline::build(Configs::new()).unwrap()
}

#[derive(Root)]
pub struct Num(u32);

static TOTAL: AtomicU32 = AtomicU32::new(0);

#[derive(ComponentSubscription)]
#[subscribe(n: Num)]
pub struct Leaf;
impl Component for Leaf {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (Leaf, 0)
    }
    fn update(&mut self, input: Self::Input<'_>) -> u32 {
        input.n.0 * 4
    }
}

#[derive(ComponentSubscription)]
#[subscribe(a: Leaf)]
pub struct App;
impl Component for App {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (App, ())
    }
    fn update(&mut self, input: Self::Input<'_>) {
        TOTAL.fetch_add(*input.a * 10, SeqCst);
    }
}

#[test]
fn generic_code_drives_a_concrete_pipeline() {
    let mut p: Pipeline<Num, (App,)> = make();
    drive(&mut p, [Num(1), Num(2)]);
    assert_eq!(TOTAL.load(SeqCst), 40 + 80);
    assert_eq!(count(&p), 2);
}
