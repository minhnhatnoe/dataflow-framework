//! Runtime guarantees of dataflow-flat.

use std::sync::atomic::{AtomicUsize, Ordering};

use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline};

// Diamond: Src -> Double, Src -> Square, (Double, Square) -> Sum, and the
// root App -> Sum, Double. Every component counts its own calls and reports
// the count in its output.

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

/// The root: what the program wants, by value. Nothing subscribes to it.
#[derive(ComponentSubscription)]
#[subscribe(sum: Sum, double: Double)]
pub struct App;
impl Component for App {
    type Config = ();
    type Output = AppOut;
    fn new((): ()) -> Self {
        App
    }
    fn update(&mut self, input: Self::Input<'_>) -> AppOut {
        AppOut { sum: *input.sum, double_value: input.double.0, double_calls: input.double.1 }
    }
}

#[derive(Debug, PartialEq)]
pub struct AppOut {
    sum: u64,
    double_value: u64,
    double_calls: u64,
}

#[test]
fn each_component_runs_once_per_run_in_topological_order() {
    let mut p = Pipeline::<App>::build(Configs::new().with::<Src>(10)).unwrap();
    assert_eq!(p.order(), ["Src", "Double", "Square", "Sum", "App"].map(|s| format!("runtime::{s}")));
    assert_eq!(p.len(), 5);
    assert!(!p.is_empty());

    for run in 1..=5u64 {
        let out = p.run(); // by value
        let n = 10 + run; // Src incremented exactly once per run
        assert_eq!(out.sum, 2 * n + n * n);
        assert_eq!(out.double_value, 2 * n);
        assert_eq!(out.double_calls, run, "Double called once per run despite 3 edges into it");
    }
}

#[test]
fn run_returns_the_root_output_by_value() {
    let mut p = Pipeline::<App>::build(Configs::new().with::<Src>(0)).unwrap();
    // Owned results: both outlive the next `run`, which needs `&mut p`.
    let first = p.run();
    let second = p.run();
    assert_eq!(first, AppOut { sum: 2 + 1, double_value: 2, double_calls: 1 });
    assert_eq!(second, AppOut { sum: 4 + 4, double_value: 4, double_calls: 2 });
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

    let err = Pipeline::<App>::build(Configs::new().with::<Src>(10).with::<NotInGraph>(()))
        .err()
        .expect("NotInGraph is not in the graph");
    assert_eq!(err.unused.len(), 1);
    assert!(err.unused[0].ends_with("NotInGraph"));

    // No config for Src: `u64::default()` is used.
    let mut p = Pipeline::<App>::build(Configs::new()).unwrap();
    assert_eq!(p.run().double_value, 2, "Src started at 0 and was incremented once");
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

#[derive(ComponentSubscription)]
#[subscribe(a: UsesTracked1, b: UsesTracked2)]
pub struct UsesBoth;
impl Component for UsesBoth {
    type Config = ();
    type Output = usize;
    fn new((): ()) -> Self {
        UsesBoth
    }
    fn update(&mut self, input: Self::Input<'_>) -> usize {
        input.a + input.b
    }
}

#[test]
fn shared_component_is_constructed_and_dropped_once() {
    {
        let mut p = Pipeline::<UsesBoth>::build(Configs::new()).unwrap();
        assert_eq!(p.len(), 4);
        assert_eq!(p.run(), 6);
        assert_eq!(p.run(), 6);
        assert_eq!(DROPS.load(Ordering::SeqCst), 0);
    }
    assert_eq!(DROPS.load(Ordering::SeqCst), 1);
}

// ---------------------------------------------------------------------------
// Outputs of non-root components do not outlive `run`.
// ---------------------------------------------------------------------------

static OUT_CREATED: AtomicUsize = AtomicUsize::new(0);
static OUT_DROPPED: AtomicUsize = AtomicUsize::new(0);

/// An output that counts its creation and its drop.
pub struct Counted(u32);
impl Counted {
    fn new(v: u32) -> Counted {
        OUT_CREATED.fetch_add(1, Ordering::SeqCst);
        Counted(v)
    }
}
impl Drop for Counted {
    fn drop(&mut self) {
        OUT_DROPPED.fetch_add(1, Ordering::SeqCst);
    }
}

#[derive(ComponentSubscription)]
pub struct Leaf;
impl Component for Leaf {
    type Config = ();
    type Output = Counted;
    fn new((): ()) -> Self {
        Leaf
    }
    fn update(&mut self, _: Self::Input<'_>) -> Counted {
        Counted::new(1)
    }
}

#[derive(ComponentSubscription)]
#[subscribe(leaf: Leaf)]
pub struct Mid;
impl Component for Mid {
    type Config = ();
    type Output = Counted;
    fn new((): ()) -> Self {
        Mid
    }
    fn update(&mut self, input: Self::Input<'_>) -> Counted {
        Counted::new(input.leaf.0 + 10)
    }
}

#[derive(ComponentSubscription)]
#[subscribe(mid: Mid, leaf: Leaf)]
pub struct Top;
impl Component for Top {
    type Config = ();
    type Output = Counted;
    fn new((): ()) -> Self {
        Top
    }
    fn update(&mut self, input: Self::Input<'_>) -> Counted {
        Counted::new(input.mid.0 + input.leaf.0 + 100)
    }
}

#[test]
fn non_root_outputs_are_dropped_by_the_end_of_each_run() {
    let mut p = Pipeline::<Top>::build(Configs::new()).unwrap();
    assert_eq!(OUT_CREATED.load(Ordering::SeqCst), 0, "no outputs exist before a run");
    for run in 1..=4usize {
        let root = p.run();
        assert_eq!(root.0, 112);
        // Leaf, Mid and Top each produced one output this run. Leaf's and
        // Mid's are already dropped; the root's is in our hands.
        assert_eq!(OUT_CREATED.load(Ordering::SeqCst), 3 * run);
        assert_eq!(OUT_DROPPED.load(Ordering::SeqCst), 2 * run + (run - 1), "run {run}");
        drop(root);
        assert_eq!(OUT_DROPPED.load(Ordering::SeqCst), 3 * run);
    }
}

// An unused config is reported before any constructor runs.
static BUILT: AtomicUsize = AtomicUsize::new(0);

#[derive(ComponentSubscription)]
pub struct Counted1;
impl Component for Counted1 {
    type Config = ();
    type Output = ();
    fn new((): ()) -> Self {
        BUILT.fetch_add(1, Ordering::SeqCst);
        Counted1
    }
    fn update(&mut self, _: Self::Input<'_>) {}
}

#[derive(ComponentSubscription)]
#[subscribe(a: Counted1)]
pub struct CountedApp;
impl Component for CountedApp {
    type Config = ();
    type Output = ();
    fn new((): ()) -> Self {
        BUILT.fetch_add(1, Ordering::SeqCst);
        CountedApp
    }
    fn update(&mut self, _: Self::Input<'_>) {}
}

#[derive(ComponentSubscription)]
pub struct Stranger;
impl Component for Stranger {
    type Config = ();
    type Output = ();
    fn new((): ()) -> Self {
        Stranger
    }
    fn update(&mut self, _: Self::Input<'_>) {}
}

#[test]
fn unused_config_is_reported_before_any_constructor_runs() {
    BUILT.store(0, Ordering::SeqCst);
    let err = Pipeline::<CountedApp>::build(Configs::new().with::<Counted1>(()).with::<Stranger>(()))
        .err()
        .expect("Stranger is not in the graph");
    assert_eq!(err.unused.len(), 1);
    assert_eq!(BUILT.load(Ordering::SeqCst), 0, "no constructor ran");
    Pipeline::<CountedApp>::build(Configs::new().with::<Counted1>(())).unwrap();
    assert_eq!(BUILT.load(Ordering::SeqCst), 2);
}
