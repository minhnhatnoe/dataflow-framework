//! Smallest interesting graph, used to inspect compiler output (MIR, LLVM IR,
//! asm; see README "Inspecting compiler output").
//!
//! Diamond:  Src -> Double, Src -> Square, (Double, Square) -> Sum
use dataflow_flat::{deps, Component, ComponentSubscription, Configs, Deps, Pipeline};

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

pub type Roots = Deps![Sum];

#[inline(never)]
pub fn run_once(p: &mut Pipeline<Roots>) { p.run() }

fn main() {
    let mut p = Pipeline::<Roots>::build(Configs::new().with::<Src>(10)).unwrap();
    run_once(&mut p);
    let deps!(sum) = p.outputs().unwrap();
    println!("order {:?}, sum = {sum}", p.order());   // n=11: 22 + 121 = 143
}
