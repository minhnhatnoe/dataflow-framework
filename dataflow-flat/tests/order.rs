//! Execution order is a guarantee: depth-first post-order in subscription
//! listing order. For C with deps [D1..Dn]: block i = Di plus everything
//! upstream of Di not yet run this run, blocks in order, then C.

use std::marker::PhantomData;
use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline, Root};

#[derive(Root)]
pub struct Poll;

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
            fn new((): (), _: Self::Init<'_>) -> (Self, u64) { ($name, 0) }
            fn update(&mut self, _: Self::Input<'_>) -> u64 {
                LOG.lock().unwrap().push(stringify!($name));
                0
            }
        }
    };
}

mod words {
    use super::*;
    comp!(Source, [p: Poll]);
    comp!(Tokenize, [line: Source]);
    comp!(WordCount, [tokens: Tokenize]);
    comp!(TopWords, [counts: WordCount]);
    comp!(Longest, [tokens: Tokenize]);
    comp!(Report, [top: TopWords, longest: Longest, line: Source]);
    #[derive(ComponentSubscription)]
    #[subscribe(items: U)]
    pub struct Len<U: Component + dataflow_flat::Node>(PhantomData<U>);
    impl<U: Component + dataflow_flat::Node> Component for Len<U> {
        type Config = ();
        type Output = u64;
        fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (Len(PhantomData), 0) }
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
    let mut p = Pipeline::<Poll, (words::App,)>::build(Configs::new()).unwrap();
    for _ in 0..3 {
        p.push(Poll);
        assert_eq!(take(), ["Source", "Tokenize", "WordCount", "TopWords", "Longest", "Report", "Len<Tokenize>", "App"]);
    }
    // Construction order is the same order.
    assert_eq!(p.order().len(), 8);
    assert_eq!(p.len(), 8);
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
    comp!(B, [p: Poll]);
    comp!(A, [b: B]);
    comp!(C, [a: A, b: B]);
    comp!(R, [p: Poll]);
    comp!(P, [r: R]);
    comp!(Q, [p: Poll]);
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
    let mut p = Pipeline::<Poll, (App1,)>::build(Configs::new()).unwrap();
    for _ in 0..2 {
        p.push(Poll);
        assert_eq!(take(), ["R", "P", "Q", "Top", "B", "A", "C", "Z", "App1"]);
    }
    let mut p = Pipeline::<Poll, (App2,)>::build(Configs::new()).unwrap();
    p.push(Poll);
    assert_eq!(take(), ["R", "P", "Q", "Top", "B", "A", "C", "Z", "App2"]);
    let mut p = Pipeline::<Poll, (App3,)>::build(Configs::new()).unwrap();
    p.push(Poll);
    assert_eq!(take(), ["Q", "R", "P", "Top", "B", "A", "C", "Z", "App3"]);
}

// The run order and the construction order (`Pipeline::order`) agree.
#[test]
fn construction_order_is_the_run_order() {
    let _g = SERIAL.lock().unwrap();
    take();
    let mut p = Pipeline::<Poll, (later_upstream::App3,)>::build(Configs::new()).unwrap();
    p.push(Poll);
    let ran = take();
    let built: Vec<&str> = p.order().iter().map(|n| n.rsplit("::").next().unwrap()).collect();
    assert_eq!(built, ran);
}

// ---------------------------------------------------------------------------
// Wide components. Subscriptions of a component are stored as a tree of
// tuples (4 entries and up to 4 children per node), so these cross node
// boundaries: 5 (one inner node), 16, 17, and 22 subscriptions. Listing order
// must hold across node boundaries.
// ---------------------------------------------------------------------------
mod wide {
    use super::*;
    comp!(X0, [p: Poll]);
    comp!(X1, [p: Poll]);
    comp!(X2, [p: Poll]);
    comp!(X3, [p: Poll]);
    comp!(X4, [p: Poll]);
    comp!(X5, [p: Poll]);
    comp!(X6, [p: Poll]);
    comp!(X7, [p: Poll]);
    comp!(X8, [p: Poll]);
    comp!(X9, [p: Poll]);
    comp!(X10, [p: Poll]);
    comp!(X11, [p: Poll]);
    comp!(X12, [p: Poll]);
    comp!(X13, [p: Poll]);
    comp!(X14, [p: Poll]);
    comp!(X15, [p: Poll]);
    comp!(X16, [p: Poll]);
    comp!(X17, [p: Poll]);
    comp!(X18, [p: Poll]);
    comp!(X19, [p: Poll]);
    comp!(X20, [p: Poll]);
    // Mid is listed early but subscribes to X20, which a wide root lists last.
    comp!(Mid, [x: X20]);
    comp!(W5, [f0: X0, f1: X1, f2: X2, f3: X3, f4: X4]);
    comp!(W16, [f0: X0, f1: X1, f2: X2, f3: X3, f4: X4, f5: X5, f6: X6, f7: X7, f8: X8, f9: X9, f10: X10, f11: X11, f12: X12, f13: X13, f14: X14, f15: X15]);
    comp!(W17, [f0: X0, f1: X1, f2: X2, f3: X3, f4: X4, f5: X5, f6: X6, f7: X7, f8: X8, f9: X9, f10: X10, f11: X11, f12: X12, f13: X13, f14: X14, f15: X15, f16: X16]);
    comp!(W21, [f0: X0, f1: X1, f2: Mid, f3: X2, f4: X3, f5: X4, f6: X5, f7: X6, f8: X7, f9: X8, f10: X9, f11: X10, f12: X11, f13: X12, f14: X13, f15: X14, f16: X15, f17: X16, f18: X17, f19: X18, f20: X19]);
    comp!(W22, [f0: X0, f1: Mid, f2: X1, f3: X2, f4: X3, f5: X4, f6: X5, f7: X6, f8: X7, f9: X8, f10: X9, f11: X10, f12: X11, f13: X12, f14: X13, f15: X14, f16: X15, f17: X16, f18: X17, f19: X18, f20: X19, f21: X20]);
}

#[test]
fn wide_components_keep_listing_order_across_nodes() {
    use wide::*;
    let _g = SERIAL.lock().unwrap();
    take();
    fn xs(r: std::ops::Range<usize>) -> Vec<String> {
        r.map(|i| format!("X{i}")).collect()
    }
    macro_rules! check {
        ($root:ty, $name:literal, $expected:expr) => {{
            let mut p = Pipeline::<Poll, ($root,)>::build(Configs::new()).unwrap();
            for _ in 0..2 {
                p.push(Poll);
                let ran: Vec<String> = take().into_iter().map(String::from).collect();
                let mut want: Vec<String> = $expected;
                want.push($name.to_string());
                assert_eq!(ran, want, $name);
            }
            let built: Vec<String> = p.order().iter().map(|n| n.rsplit("::").next().unwrap().to_string()).collect();
            let mut want: Vec<String> = $expected;
            want.push($name.to_string());
            assert_eq!(built, want, "construction order of {}", $name);
        }};
    }
    check!(W5, "W5", xs(0..5));
    check!(W16, "W16", xs(0..16));
    check!(W17, "W17", xs(0..17));
    // W21 = [X0, X1, Mid, X2..X19]: Mid's block is X20, Mid.
    let mut w21 = xs(0..2);
    w21.extend(["X20".to_string(), "Mid".to_string()]);
    w21.extend(xs(2..20));
    check!(W21, "W21", w21.clone());
    // W22 = [X0, Mid, X1..X20]: X20 is listed last but runs inside Mid's block.
    let mut w22 = vec!["X0".to_string(), "X20".to_string(), "Mid".to_string()];
    w22.extend(xs(1..20));
    check!(W22, "W22", w22.clone());
}
