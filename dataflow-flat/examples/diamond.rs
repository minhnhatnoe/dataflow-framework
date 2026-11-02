//! Smallest interesting graph, used to inspect compiler output (MIR, LLVM IR,
//! asm; see README "Inspecting compiler output").
//!
//! Run with `cargo run -p dataflow-flat --example diamond`.
//!
//! Diamond:  Tick -> Src -> Double, Src -> Square, (Double, Square) -> Sum (the sink)
use dataflow_flat::{Component, ComponentSubscription, UpdateHandler, Configs, Pipeline, Root};

/// The event: one tick per push.
#[derive(Root)]
pub struct Tick;

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(tick: Tick)]
pub struct Src {
    n: u64,
}

impl Component for Src {
    type Config = u64;
    type Output = u64;
    fn new(start: u64, _: Self::Init<'_>) -> (Self, u64) { (Src { n: start }, start) }
    fn update(&mut self, _: Self::Input<'_>) -> u64 { self.n += 1; self.n }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(x: Src)]
pub struct Double;

impl Component for Double {
    type Config = ();
    type Output = u64;
    fn new((): (), init: Self::Init<'_>) -> (Self, u64) { (Double, *init.x * 2) }
    fn update(&mut self, input: Self::Input<'_>) -> u64 { input.x * 2 }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(x: Src)]
pub struct Square;

impl Component for Square {
    type Config = ();
    type Output = u64;
    fn new((): (), init: Self::Init<'_>) -> (Self, u64) { (Square, *init.x * *init.x) }
    fn update(&mut self, input: Self::Input<'_>) -> u64 { input.x * input.x }
}

#[derive(ComponentSubscription, UpdateHandler)]
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
    fn new((): (), init: Self::Init<'_>) -> (Self, u64) { (Sum { calls: 0 }, *init.doubled + *init.squared) }
    fn update(&mut self, input: Self::Input<'_>) -> u64 {
        self.calls += 1;
        input.doubled + input.squared
    }
}

/// One step. `Sum` is the sink; its output is stored in the pipeline.
#[inline(never)]
pub fn push_once(p: &mut Pipeline<Tick, (Sum,)>) { p.push(Tick) }

fn main() {
    let mut p = Pipeline::<Tick, (Sum,)>::build(Configs::new().with::<Src>(10)).unwrap();
    push_once(&mut p);
    // n=11: 22 + 121 = 143 is now Sum's stored output.
    println!("order {:?}", p.order());
}
