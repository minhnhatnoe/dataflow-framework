//! Generic user code over `Pipeline<A>`: one nameable bound, `Root`, and no
//! `generic_const_exprs` feature gate in the user crate (this file has none).
use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline, Root};

pub fn drive<A: Root>(p: &mut Pipeline<A>) -> A::Output {
    p.run()
}

pub fn count<A: Root>(p: &Pipeline<A>) -> usize {
    p.len()
}

#[derive(ComponentSubscription)]
pub struct Leaf;
impl Component for Leaf {
    type Config = ();
    type Output = u32;
    fn new((): ()) -> Self {
        Leaf
    }
    fn update(&mut self, _: Self::Input<'_>) -> u32 {
        4
    }
}

#[derive(ComponentSubscription)]
#[subscribe(a: Leaf)]
pub struct App;
impl Component for App {
    type Config = ();
    type Output = u32;
    fn new((): ()) -> Self {
        App
    }
    fn update(&mut self, input: Self::Input<'_>) -> u32 {
        *input.a * 10
    }
}

#[test]
fn generic_code_drives_a_concrete_pipeline() {
    let mut p = Pipeline::<App>::build(Configs::new()).unwrap();
    assert_eq!(drive(&mut p), 40);
    assert_eq!(count(&p), 2);
}
