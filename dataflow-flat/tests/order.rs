//! Execution order is a guarantee: depth-first post-order in subscription
//! listing order. For C with deps [D1..Dn]: block i = Di plus everything
//! upstream of Di not yet run this run, blocks in order, then C.

use std::marker::PhantomData;
use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline};

static LOG: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
// Tests in one binary run in parallel; each graph's test holds this lock.
static SERIAL: Mutex<()> = Mutex::new(());

fn take() -> Vec<&'static str> {
    std::mem::take(&mut *LOG.lock().unwrap())
}

macro_rules! comp {
    ($name:ident, [$($n:ident: $d:ty),*]) => {
        #[derive(ComponentSubscription)]
        #[subscribe($($n: $d),*)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): ()) -> Self { $name }
            fn update(&mut self, _: Self::Input<'_>) -> u64 {
                LOG.lock().unwrap().push(stringify!($name));
                0
            }
        }
    };
}

mod words {
    use super::*;
    comp!(Source, []);
    comp!(Tokenize, [line: Source]);
    comp!(WordCount, [tokens: Tokenize]);
    comp!(TopWords, [counts: WordCount]);
    comp!(Longest, [tokens: Tokenize]);
    comp!(Report, [top: TopWords, longest: Longest, line: Source]);
    #[derive(ComponentSubscription)]
    #[subscribe(items: U)]
    pub struct Len<U: Component>(PhantomData<U>);
    impl<U: Component> Component for Len<U> {
        type Config = ();
        type Output = u64;
        fn new((): ()) -> Self { Len(PhantomData) }
        fn update(&mut self, _: Self::Input<'_>) -> u64 {
            LOG.lock().unwrap().push("Len<Tokenize>");
            0
        }
    }
    comp!(App, [report: Report, count: Len<Tokenize>]);
}

#[test]
fn example_graph_exact_order() {
    let _g = SERIAL.lock().unwrap();
    take();
    let mut p = Pipeline::<words::App>::build(Configs::new()).unwrap();
    for _ in 0..3 {
        p.run();
        assert_eq!(take(), ["Source", "Tokenize", "WordCount", "TopWords", "Longest", "Report", "Len<Tokenize>", "App"]);
    }
    // Construction order is the same order.
    assert_eq!(p.order().len(), 8);
}

// Later-listed deps upstream of earlier ones.
//   B -> A            (A subscribes to B)
//   C: [A, B]         B is listed second but runs inside A's block
//   R -> P, Q (independent), Top: [P, Q, R]: R runs in P's block, before Q
//   Z: [Top, C]
//   App1: [Z]
//   App2: [Z, B, Q]   B and Q are already upstream of Z: they never run twice
//   App3: [Q, Z, B]   Q first: then Z's block skips Q
mod later_upstream {
    use super::*;
    comp!(B, []);
    comp!(A, [b: B]);
    comp!(C, [a: A, b: B]);
    comp!(R, []);
    comp!(P, [r: R]);
    comp!(Q, []);
    comp!(Top, [p: P, q: Q, r: R]);
    comp!(Z, [top: Top, c: C]);
    comp!(App1, [z: Z]);
    comp!(App2, [z: Z, b: B, q: Q]);
    comp!(App3, [q: Q, z: Z, b: B]);
}

#[test]
fn later_listed_dep_upstream_of_earlier_one() {
    use later_upstream::*;
    let _g = SERIAL.lock().unwrap();
    take();
    let mut p = Pipeline::<App1>::build(Configs::new()).unwrap();
    for _ in 0..2 {
        p.run();
        assert_eq!(take(), ["R", "P", "Q", "Top", "B", "A", "C", "Z", "App1"]);
    }
    let mut p = Pipeline::<App2>::build(Configs::new()).unwrap();
    p.run();
    assert_eq!(take(), ["R", "P", "Q", "Top", "B", "A", "C", "Z", "App2"]);
    let mut p = Pipeline::<App3>::build(Configs::new()).unwrap();
    p.run();
    assert_eq!(take(), ["Q", "R", "P", "Top", "B", "A", "C", "Z", "App3"]);
}

// The run order and the construction order (`Pipeline::order`) agree.
#[test]
fn construction_order_is_the_run_order() {
    let _g = SERIAL.lock().unwrap();
    take();
    let mut p = Pipeline::<later_upstream::App3>::build(Configs::new()).unwrap();
    p.run();
    let ran = take();
    let built: Vec<&str> = p.order().iter().map(|n| n.rsplit("::").next().unwrap()).collect();
    assert_eq!(built, ran);
}
