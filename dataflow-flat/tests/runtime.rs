//! Runtime guarantees of dataflow-flat.

use std::sync::atomic::{AtomicUsize, Ordering};

use dataflow_flat::{deps, Component, ComponentSubscription, Configs, Deps, Pipeline};

// Diamond: Src -> Double, Src -> Square, (Double, Square) -> Sum.
// Every component counts its own calls and reports the count in its output.

#[derive(ComponentSubscription)]
pub struct Src {
    n: u64,
}
impl Component for Src {
    type Config = u64;
    type Output = u64;
    fn new(start: u64) -> Self {
        Src { n: start }
    }
    fn update(&mut self, _: Self::Input<'_>) -> u64 {
        self.n += 1;
        self.n
    }
}

#[derive(ComponentSubscription)]
#[subscribe(x: Src)]
pub struct Double {
    calls: u64,
}
impl Component for Double {
    type Config = ();
    type Output = (u64, u64); // (value, calls)
    fn new((): ()) -> Self {
        Double { calls: 0 }
    }
    fn update(&mut self, input: Self::Input<'_>) -> (u64, u64) {
        self.calls += 1;
        (input.x * 2, self.calls)
    }
}

#[derive(ComponentSubscription)]
#[subscribe(x: Src)]
pub struct Square;
impl Component for Square {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self {
        Square
    }
    fn update(&mut self, input: Self::Input<'_>) -> u64 {
        input.x * input.x
    }
}

#[derive(ComponentSubscription)]
#[subscribe(doubled: Double, squared: Square, again: Double)]
pub struct Sum;
impl Component for Sum {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self {
        Sum
    }
    fn update(&mut self, input: Self::Input<'_>) -> u64 {
        // Same component under two names: one dependency, same reference.
        assert!(std::ptr::eq(input.doubled, input.again));
        input.doubled.0 + input.squared
    }
}

type Roots = Deps![Sum, Double];

#[test]
fn each_component_runs_once_per_run_in_topological_order() {
    let mut p = Pipeline::<Roots>::build(Configs::new().with::<Src>(10)).unwrap();
    assert!(p.outputs().is_none(), "no outputs before the first run");
    assert_eq!(p.order(), ["Src", "Double", "Square", "Sum"].map(|s| format!("runtime::{s}")));

    for run in 1..=5u64 {
        p.run();
        let deps!(sum, double) = p.outputs().unwrap();
        let n = 10 + run; // Src incremented exactly once per run
        assert_eq!(*sum, 2 * n + n * n);
        assert_eq!(double.1, run, "Double called once per run despite 3 edges into it");
    }
}

#[test]
fn unused_configs_are_reported_and_missing_ones_default() {
    #[derive(ComponentSubscription)]
    pub struct NotInGraph;
    impl Component for NotInGraph {
        type Config = ();
        type Output = ();
        fn new((): ()) -> Self {
            NotInGraph
        }
        fn update(&mut self, _: Self::Input<'_>) {}
    }

    let err = Pipeline::<Roots>::build(Configs::new().with::<Src>(10).with::<NotInGraph>(()))
        .err()
        .expect("NotInGraph is not in the graph");
    assert_eq!(err.unused.len(), 1);
    assert!(err.unused[0].ends_with("NotInGraph"));

    // No config for Src: `u64::default()` is used.
    let mut p = Pipeline::<Roots>::build(Configs::new()).unwrap();
    p.run();
    let deps!(_, double) = p.outputs().unwrap();
    assert_eq!(double.0, 2, "Src started at 0 and was incremented once");
}

static DROPS: AtomicUsize = AtomicUsize::new(0);

#[derive(ComponentSubscription)]
pub struct Tracked;
impl Drop for Tracked {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}
impl Component for Tracked {
    type Config = ();
    type Output = Vec<u8>; // heap output, also dropped
    fn new((): ()) -> Self {
        Tracked
    }
    fn update(&mut self, _: Self::Input<'_>) -> Vec<u8> {
        vec![1, 2, 3]
    }
}

#[derive(ComponentSubscription)]
#[subscribe(a: Tracked)]
pub struct UsesTracked1;
impl Component for UsesTracked1 {
    type Config = ();
    type Output = usize;
    fn new((): ()) -> Self {
        UsesTracked1
    }
    fn update(&mut self, input: Self::Input<'_>) -> usize {
        input.a.len()
    }
}

#[derive(ComponentSubscription)]
#[subscribe(a: Tracked)]
pub struct UsesTracked2;
impl Component for UsesTracked2 {
    type Config = ();
    type Output = usize;
    fn new((): ()) -> Self {
        UsesTracked2
    }
    fn update(&mut self, input: Self::Input<'_>) -> usize {
        input.a.len()
    }
}

#[test]
fn shared_component_is_constructed_and_dropped_once() {
    {
        let mut p = Pipeline::<Deps![UsesTracked1, UsesTracked2]>::build(Configs::new()).unwrap();
        assert_eq!(p.len(), 3);
        p.run();
        p.run();
        assert_eq!(DROPS.load(Ordering::SeqCst), 0);
    }
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
}
