//! Two distinct component types with the identical `type_name`
//! ("foo::Foo", from two versions of one crate) are two components: both are
//! in the plan, constructed once, run once per run and dropped once, and a
//! config for one is not taken by the other.

use std::any::{type_name, TypeId};
use std::sync::atomic::Ordering::SeqCst;

use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline};

#[derive(ComponentSubscription)]
#[subscribe(a: foo_a::Foo, b: foo_b::Foo)]
pub struct App;
impl Component for App {
    type Config = ();
    type Output = (u32, u32);
    fn new((): ()) -> Self {
        App
    }
    fn update(&mut self, input: Self::Input<'_>) -> (u32, u32) {
        (*input.a, *input.b)
    }
}

#[test]
fn same_type_name_different_components() {
    // The premise: same name, different types.
    assert_eq!(type_name::<foo_a::Foo>(), type_name::<foo_b::Foo>());
    assert_ne!(TypeId::of::<foo_a::Foo>(), TypeId::of::<foo_b::Foo>());

    {
        let mut p = Pipeline::<App>::build(Configs::new()).unwrap();
        // Distinct in the plan: three components, not two.
        assert_eq!(p.len(), 3);
        assert_eq!(foo_a::NEW.load(SeqCst), 1);
        assert_eq!(foo_b::NEW.load(SeqCst), 1);
        // Each ran and was seen by the root as its own output.
        assert_eq!(p.run(), (1, 2));
        assert_eq!(p.run(), (1, 2));
        assert_eq!(foo_a::UPDATE.load(SeqCst), 2);
        assert_eq!(foo_b::UPDATE.load(SeqCst), 2);
        assert_eq!(foo_a::DROP.load(SeqCst), 0);
        assert_eq!(foo_b::DROP.load(SeqCst), 0);
    }
    assert_eq!(foo_a::DROP.load(SeqCst), 1);
    assert_eq!(foo_b::DROP.load(SeqCst), 1);

    config_for_a_component_outside_the_graph_is_reported_by_identity();
}

// Not a test of its own: the counters are process-wide, so the two checks run
// one after the other from the single test above.
fn config_for_a_component_outside_the_graph_is_reported_by_identity() {
    // Only `foo_a::Foo` is in this graph; a config for `foo_b::Foo` has the
    // same type name but is not in it.
    #[derive(ComponentSubscription)]
    #[subscribe(a: foo_a::Foo)]
    pub struct OnlyA;
    impl Component for OnlyA {
        type Config = ();
        type Output = u32;
        fn new((): ()) -> Self {
            OnlyA
        }
        fn update(&mut self, input: Self::Input<'_>) -> u32 {
            *input.a
        }
    }
    let err = Pipeline::<OnlyA>::build(Configs::new().with::<foo_b::Foo>(())).err().unwrap();
    assert_eq!(err.unused, vec!["foo::Foo"]);
    assert!(Pipeline::<OnlyA>::build(Configs::new().with::<foo_a::Foo>(())).is_ok());
}
