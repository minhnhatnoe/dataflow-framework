//! Demuxing on the subscription edge: selector chains, pruning, conditions
//! (OR / partial updates reading stored outputs), nested chains, `Some`/`None`,
//! unit variants, guards, absorption, exclusive joins, order within a step.
//! No `#![feature]` here.

use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, Configs, Demux, Pipeline, Root, Select, GUARD};

static LOG: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
// Tests in one binary run in parallel; each test holds this lock and clears the log.
static SERIAL: Mutex<()> = Mutex::new(());

fn take() -> Vec<&'static str> {
    std::mem::take(&mut *LOG.lock().unwrap())
}
fn log(s: &'static str) {
    LOG.lock().unwrap().push(s);
}

/// A component with the given subscriptions whose output is `$out(input)`;
/// logs its name when it runs.
macro_rules! comp {
    ($(#[$m:meta])* $name:ident, [$($sub:tt)*], $out:ty, |$i:ident| $body:expr) => {
        #[derive(ComponentSubscription)]
        #[subscribe($($sub)*)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = $out;
            fn new((): (), _: Self::Init<'_>) -> (Self, $out) {
                ($name, Default::default())
            }
            fn update(&mut self, $i: Self::Input<'_>) -> $out {
                log(stringify!($name));
                $body
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Pruning and joins
// ---------------------------------------------------------------------------

#[derive(Root, Demux)]
pub enum Pkt {
    A(u32),
    B(u32),
    Tick,
}

mod basic {
    use super::*;
    comp!(OnA, [p: Pkt => pkt::A], u32, |i| *i.p * 10);
    comp!(OnB, [p: Pkt => pkt::B], u32, |i| *i.p * 100);
    comp!(AfterA, [a: OnA], u32, |i| *i.a + 1);
    // An exclusive join: reads both; the one that did not run supplies its stored output.
    comp!(Join, [a: AfterA, b: OnB], (u32, u32), |i| (*i.a, *i.b));
    comp!(OnTick, [t: Pkt => pkt::Tick], (), |i| {
        let _: &() = i.t;
    });
    // Absorption: the condition {A} is implied by both edges.
    comp!(Absorb, [a: OnA, p: Pkt => pkt::A], u32, |i| *i.a + *i.p);
    // A plain root edge makes the component run on every push.
    comp!(Always, [p: Pkt], u32, |i| match i.p {
        Pkt::A(x) | Pkt::B(x) => *x,
        Pkt::Tick => 0,
    });
    // A condition that is "always" absorbs the conditional edge.
    comp!(Sink, [j: Join, t: OnTick, x: Absorb, y: Always], (), |i| {
        let _ = (i.j, i.t, i.x, i.y);
    });
}

#[test]
fn pruning_runs_only_the_matching_arm() {
    let _g = SERIAL.lock().unwrap();
    take();
    let mut p = Pipeline::<Pkt, (basic::Join,)>::build(Configs::new()).unwrap();
    take();
    p.push(Pkt::A(1));
    assert_eq!(take(), ["OnA", "AfterA", "Join"]);
    p.push(Pkt::B(2));
    assert_eq!(take(), ["OnB", "Join"]);
    p.push(Pkt::Tick);
    assert_eq!(take(), Vec::<&str>::new());
    p.push(Pkt::A(3));
    assert_eq!(take(), ["OnA", "AfterA", "Join"]);
}

#[test]
fn join_reads_stored_outputs_of_components_that_did_not_run() {
    let _g = SERIAL.lock().unwrap();
    // Observe Join's output through a sink that stores it.
    static SEEN: Mutex<Vec<(u32, u32)>> = Mutex::new(Vec::new());
    comp!(Watch, [j: basic::Join], (), |i| SEEN.lock().unwrap().push(*i.j));
    take();
    let mut p = Pipeline::<Pkt, (Watch,)>::build(Configs::new()).unwrap();
    take();
    p.push(Pkt::A(1)); // OnA = 10, AfterA = 11, OnB stored (0)
    p.push(Pkt::B(2)); // OnB = 200, AfterA stored (11)
    p.push(Pkt::Tick); // nothing
    p.push(Pkt::A(5)); // OnA = 50, AfterA = 51, OnB stored (200)
    assert_eq!(*SEEN.lock().unwrap(), [(11, 0), (11, 200), (51, 200)]);
}

#[test]
fn unit_variant_and_absorption_and_always() {
    let _g = SERIAL.lock().unwrap();
    take();
    let mut p = Pipeline::<Pkt, (basic::Sink,)>::build(Configs::new()).unwrap();
    take();
    // Sink has an "always" edge (Always), so it and everything unconditional run every time.
    p.push(Pkt::Tick);
    assert_eq!(take(), ["OnTick", "Always", "Sink"]);
    p.push(Pkt::A(1));
    assert_eq!(take(), ["OnA", "AfterA", "Join", "Absorb", "Always", "Sink"]);
    p.push(Pkt::B(1));
    assert_eq!(take(), ["OnB", "Join", "Always", "Sink"]);
}

// ---------------------------------------------------------------------------
// Option, nested chains, a computed value
// ---------------------------------------------------------------------------

#[derive(Root)]
pub struct Msg {
    v: Option<u32>,
}

mod opt {
    use super::*;
    comp!(Parse, [m: Msg], Option<u32>, |i| i.m.v);
    comp!(Got, [x: Parse => Some], u32, |i| *i.x + 1);
    comp!(Missing, [x: Parse => None], u32, |_i| 999);
    comp!(Either, [g: Got, m: Missing], (u32, u32), |i| (*i.g, *i.m));
}

#[test]
fn some_and_none() {
    let _g = SERIAL.lock().unwrap();
    take();
    let mut p = Pipeline::<Msg, (opt::Either,)>::build(Configs::new()).unwrap();
    take();
    p.push(Msg { v: Some(4) });
    assert_eq!(take(), ["Parse", "Got", "Either"]);
    p.push(Msg { v: None });
    assert_eq!(take(), ["Parse", "Missing", "Either"]);
    p.push(Msg { v: Some(7) });
    assert_eq!(take(), ["Parse", "Got", "Either"]);
}

#[derive(Demux)]
pub enum Inner {
    Depth(u32),
    Trade(u64),
}
#[derive(Root, Demux)]
pub enum Outer {
    Binance(Inner),
    Coinbase(Inner),
    Ping,
}

mod nested {
    use super::*;
    // Two selectors in one chain, then a different leaf of the same prefix.
    comp!(BinDepth, [p: Outer => outer::Binance, inner::Depth], u32, |i| *i.p);
    comp!(BinTrade, [p: Outer => outer::Binance, inner::Trade], u64, |i| *i.p);
    comp!(CoinDepth, [p: Outer => outer::Coinbase, inner::Depth], u32, |i| *i.p);
    // Joins of exclusive leaves at different levels.
    comp!(AnyDepth, [b: BinDepth, c: CoinDepth], u32, |i| *i.b + *i.c);
    comp!(AnyBin, [d: BinDepth, t: BinTrade], (u32, u64), |i| (*i.d, *i.t));
    comp!(Top, [d: AnyDepth, b: AnyBin], (), |i| {
        let _ = (i.d, i.b);
    });
}

#[test]
fn nested_chains_share_prefixes_and_prune() {
    let _g = SERIAL.lock().unwrap();
    take();
    let mut p = Pipeline::<Outer, (nested::Top,)>::build(Configs::new()).unwrap();
    take();
    p.push(Outer::Binance(Inner::Depth(1)));
    assert_eq!(take(), ["BinDepth", "AnyDepth", "AnyBin", "Top"]);
    p.push(Outer::Binance(Inner::Trade(2)));
    assert_eq!(take(), ["BinTrade", "AnyBin", "Top"]);
    p.push(Outer::Coinbase(Inner::Depth(3)));
    assert_eq!(take(), ["CoinDepth", "AnyDepth", "Top"]);
    p.push(Outer::Coinbase(Inner::Trade(3))); // no selector matches
    p.push(Outer::Ping);
    assert_eq!(take(), Vec::<&str>::new());
}

// ---------------------------------------------------------------------------
// A guard
// ---------------------------------------------------------------------------

#[derive(Root)]
pub struct Num(u32);

pub struct Big;
impl Select<Num> for Big {
    type Out = u32;
    const KEY: u32 = GUARD;
    fn select(x: &Num) -> Option<&u32> {
        (x.0 > 10).then_some(&x.0)
    }
    fn key_of(_: &Num) -> u32 {
        GUARD
    }
}

mod guard {
    use super::*;
    comp!(Loud, [n: Num => Big], u32, |i| *i.n * 2);
    comp!(Plain, [n: Num], u32, |i| i.n.0);
    comp!(After, [l: Loud], u32, |i| *i.l + 1);
}

#[test]
fn guard_selector() {
    let _g = SERIAL.lock().unwrap();
    take();
    let mut p = Pipeline::<Num, (guard::After, guard::Plain)>::build(Configs::new()).unwrap();
    take();
    p.push(Num(3));
    assert_eq!(take(), ["Plain"]);
    p.push(Num(30));
    assert_eq!(take(), ["Loud", "After", "Plain"]);
}
