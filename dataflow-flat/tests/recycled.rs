//! Both ordinary and active-handler drivers must recycle outputs.
use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline, Root, UpdateHandler};
use std::{cell::RefCell, rc::Rc};
#[derive(Root)]
struct Tick(u64);
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(t: Tick)]
struct ViewSource(u64);
impl Component for ViewSource {
    type Config = ();
    type Handle = ();
    type View<'a> = &'a u64;
    fn new(_: (), _: Self::Init<'_>) -> (Self, ()) {
        (Self(0), ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        self.0 = i.t.0;
    }
    fn view<'a>(&'a self, _: &'a ()) -> &'a u64 {
        &self.0
    }
}
macro_rules! accumulator {
    ($name:ident, $source:ty) => {
        #[derive(ComponentSubscription)]
        #[subscribe(value: $source)]
        struct $name;
        impl UpdateHandler<$source> for $name {}
        impl Component for $name {
            type Config = ();
            type Output = Vec<u64>;
            fn new(_: (), _: Self::Init<'_>) -> (Self, Vec<u64>) {
                (Self, vec![])
            }
            fn update(&mut self, _: Self::Input<'_>) -> Vec<u64> {
                panic!("recycled path required")
            }
            fn take_output(v: &mut Vec<u64>) -> Option<Vec<u64>> {
                Some(std::mem::take(v))
            }
            fn update_recycled(
                &mut self,
                _: Self::Input<'_>,
                previous: Option<Vec<u64>>,
            ) -> Vec<u64> {
                let mut v = previous.unwrap();
                v.push(v.len() as u64);
                v
            }
        }
    };
}
accumulator!(FromRoot, Tick);
accumulator!(FromView, ViewSource);
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(a: FromRoot, b: FromView)]
struct Check(Rc<RefCell<Vec<(usize, usize)>>>);
impl Component for Check {
    type Config = Rc<RefCell<Vec<(usize, usize)>>>;
    type Output = ();
    fn new(log: Self::Config, _: Self::Init<'_>) -> (Self, ()) {
        (Self(log), ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        self.0.borrow_mut().push((i.a.len(), i.b.len()));
    }
}
#[test]
fn active_handlers_recycle_with_value_and_handle_parents() {
    let log = Rc::new(RefCell::new(vec![]));
    let mut p =
        Pipeline::<Tick, (Check,)>::build(Configs::new().with::<Check>(log.clone())).unwrap();
    p.push(Tick(1));
    p.push(Tick(2));
    assert_eq!(*log.borrow(), vec![(1, 1), (2, 2)]);
}
