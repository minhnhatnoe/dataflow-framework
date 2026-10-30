//! Smallest interesting graph, used to inspect compiler output (MIR, LLVM IR,
//! asm; see README "Inspecting compiler output").
//!
//! Diamond:  Src -> Double, Src -> Square, (Double, Square) -> Sum (the root)
use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline};

#[derive(ComponentSubscription)]
pub struct Src {
    n: u64,
}

impl Component for Src {
    type Config = u64;
    type Output = u64;
    fn new(start: u64) -> Self { Src { n: start } }
    fn update(&mut self, _: Self::Input<'_>) -> u64 { self.n += 1; self.n }
}

#[derive(ComponentSubscription)]
#[subscribe(x: Src)]
pub struct Double;

impl Component for Double {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self { Double }
    fn update(&mut self, input: Self::Input<'_>) -> u64 { input.x * 2 }
}

#[derive(ComponentSubscription)]
#[subscribe(x: Src)]
pub struct Square;

impl Component for Square {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self { Square }
    fn update(&mut self, input: Self::Input<'_>) -> u64 { input.x * input.x }
}

#[derive(ComponentSubscription)]
#[subscribe(
    doubled: Double,
    squared: Square,
)]
pub struct Sum {
    calls: u64,
}

impl Component for Sum {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self { Sum { calls: 0 } }
    fn update(&mut self, input: Self::Input<'_>) -> u64 {
        self.calls += 1;
        input.doubled + input.squared
    }
}

/// `Sum` is the root: nothing subscribes to it, and `run` returns its output.
#[inline(never)]
pub fn run_once(p: &mut Pipeline<Sum>) -> u64 { p.run() }

fn main() {
    let mut p = Pipeline::<Sum>::build(Configs::new().with::<Src>(10)).unwrap();
    let sum = run_once(&mut p);
    println!("order {:?}, sum = {sum}", p.order());   // n=11: 22 + 121 = 143
}
