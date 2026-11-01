use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline, Root};
use std::sync::Mutex;

#[derive(Root)]
pub struct Poll;

#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (A, 7)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u32 {
        8
    }
}

static SEEN: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());

#[derive(ComponentSubscription)]
#[subscribe(a: A, p: Poll)]
pub struct B;
impl Component for B {
    type Config = ();
    type Output = ();
    fn new((): (), init: Self::Init<'_>) -> (Self, ()) {
        SEEN.lock().unwrap().push((*init.a, 0));
        (B, ())
    }
    fn update(&mut self, input: Self::Input<'_>) {
        SEEN.lock().unwrap().push((*input.a, 1));
    }
}

#[test]
fn smoke() {
    let mut p = Pipeline::<Poll, (B,)>::build(Configs::new()).unwrap();
    p.push(Poll);
    assert_eq!(*SEEN.lock().unwrap(), [(7, 0), (8, 1)]);
}
