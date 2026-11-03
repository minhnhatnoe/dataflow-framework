//! Whether an edge fired is never stored: where the push program's position does not say, the
//! condition is asked again from the pushed root and the stored outputs. Covers a reader of two
//! independent guards (an always-running handler: `on_update` only for the edges that fired, in
//! parent order, before `update`), an always-running component between a conditional parent and
//! its conditional child, a stale parent (its stored output still matches, it did not run), and a
//! push where nothing matches. `scripts/gen_noflags.py` generates random graphs for the
//! differential check against an older checkout. No `#![feature]` here.

use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline, Root, Select, UpdateHandler, GUARD};

static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SERIAL: Mutex<()> = Mutex::new(());
fn log(s: impl Into<String>) {
    LOG.lock().unwrap().push(s.into());
}
fn take() -> Vec<String> {
    std::mem::take(&mut *LOG.lock().unwrap())
}

#[derive(Root)]
pub struct Pk {
    a: u32,
    b: u32,
}
/// Matches when `a` is a multiple of 3.
pub struct G1;
impl Select<Pk> for G1 {
    type Out = u32;
    const KEY: u32 = GUARD;
    fn select(x: &Pk) -> Option<&u32> {
        (x.a % 3 == 0).then_some(&x.a)
    }
    fn key_of(_: &Pk) -> u32 {
        GUARD
    }
}
/// Matches when `b` is over 50.
pub struct G2;
impl Select<Pk> for G2 {
    type Out = u32;
    const KEY: u32 = GUARD;
    fn select(x: &Pk) -> Option<&u32> {
        (x.b > 50).then_some(&x.b)
    }
    fn key_of(_: &Pk) -> u32 {
        GUARD
    }
}
/// Matches a stored output of 50 or more.
pub struct Big;
impl Select<u32> for Big {
    type Out = u32;
    const KEY: u32 = GUARD;
    fn select(x: &u32) -> Option<&u32> {
        (*x >= 50).then_some(x)
    }
    fn key_of(_: &u32) -> u32 {
        GUARD
    }
}

macro_rules! comp {
    ($name:ident, [$($sub:tt)*], |$i:ident| $body:expr) => {
        #[derive(ComponentSubscription, UpdateHandler)]
        #[subscribe($($sub)*)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = u32;
            fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
                ($name, 0)
            }
            fn update(&mut self, $i: Self::Input<'_>) -> u32 {
                log(stringify!($name));
                $body
            }
        }
    };
}
comp!(Ga, [G1(x): Pk], |i| *i.x * 10);
comp!(Gb, [G2(x): Pk], |i| *i.x);
comp!(Mid, [p: Pk], |i| i.p.a);
comp!(Child, [ga: Ga], |i| *i.ga + 1);
// Runs after `Mid` (an always-running component after `Ga`), on `Ga`'s stored output.
comp!(St, [Big(x): Ga], |i| *i.x);

/// Always runs; handles two independent guards.
#[derive(ComponentSubscription)]
#[subscribe(p: Pk, ga: Ga, gb: Gb)]
pub struct Rd;
impl Component for Rd {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (Rd, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u32 {
        log(format!("Rd.update ga={} gb={}", i.ga, i.gb));
        0
    }
}
impl UpdateHandler<Pk> for Rd {
    const ACTIVE: bool = false;
}
impl UpdateHandler<Ga> for Rd {
    fn on_update(&mut self, _: Self::Input<'_>, v: &u32) {
        log(format!("Rd.on ga={v}"));
    }
}
impl UpdateHandler<Gb> for Rd {
    fn on_update(&mut self, _: Self::Input<'_>, v: &u32) {
        log(format!("Rd.on gb={v}"));
    }
}

#[test]
fn guards_are_asked_again() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Pk, (Ga, Mid, Child, St, Rd)>::build(Configs::new()).unwrap();
    take();
    // a = 6: Ga runs (60); b = 10: Gb does not. `Child` and `St` follow `Ga`, `Mid` is between.
    p.push(Pk { a: 6, b: 10 });
    assert_eq!(take(), ["Ga", "Mid", "Child", "St", "Rd.on ga=60", "Rd.update ga=60 gb=0"]);
    // a = 4: Ga does not run but its stored output (60) still matches `Big`: St must not run.
    // b = 60: Gb runs. The reader handles only the edge that fired.
    p.push(Pk { a: 4, b: 60 });
    assert_eq!(take(), ["Mid", "Gb", "Rd.on gb=60", "Rd.update ga=60 gb=60"]);
    // Both guards at once: the handlers run in the parents' order, before update.
    p.push(Pk { a: 9, b: 99 });
    assert_eq!(take(), ["Ga", "Mid", "Child", "St", "Gb", "Rd.on ga=90", "Rd.on gb=99", "Rd.update ga=90 gb=99"]);
    // Nothing matches: only the always-running components.
    p.push(Pk { a: 5, b: 5 });
    assert_eq!(take(), ["Mid", "Rd.update ga=90 gb=99"]);
    // A stale `Ga` again, with its stored 90.
    p.push(Pk { a: 5, b: 70 });
    assert_eq!(take(), ["Mid", "Gb", "Rd.on gb=70", "Rd.update ga=90 gb=70"]);
}

// ---------------------------------------------------------------------------
// A handle source behind a guard: its view is stale in pushes where it did not run
// ---------------------------------------------------------------------------

use dataflow_flat::Demux;

#[derive(Demux)]
pub enum HV<'a> {
    Big(&'a u64),
    Small(u64),
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(G1(v): Pk)]
pub struct Hf {
    ring: [u64; 2],
    n: usize,
}
impl Component for Hf {
    type Config = ();
    type Handle = usize;
    type View<'a> = HV<'a>;
    fn new((): (), _: Self::Init<'_>) -> (Self, usize) {
        (Hf { ring: [0; 2], n: 0 }, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> usize {
        log(format!("Hf {}", i.v));
        self.n += 1;
        let s = self.n % 2;
        self.ring[s] = *i.v as u64;
        s
    }
    fn view<'a>(&'a self, h: &'a usize) -> HV<'a> {
        if self.ring[*h] >= 15 {
            HV::Big(&self.ring[*h])
        } else {
            HV::Small(self.ring[*h])
        }
    }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(HV::Big(x): Hf)]
pub struct Hb;
impl Component for Hb {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (Hb, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u32 {
        log(format!("Hb {}", i.x));
        *i.x as u32
    }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(HV::Small(n): Hf)]
pub struct Hs;
impl Component for Hs {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (Hs, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u32 {
        log(format!("Hs {}", i.n));
        i.n as u32
    }
}
/// By-value parents whose conditions are on a handle source.
#[derive(ComponentSubscription)]
#[subscribe(p: Pk, b: Hb, s: Hs)]
pub struct Hr;
impl Component for Hr {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (Hr, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u32 {
        log(format!("Hr.update b={} s={}", i.b, i.s));
        0
    }
}
impl UpdateHandler<Pk> for Hr {
    const ACTIVE: bool = false;
}
impl UpdateHandler<Hb> for Hr {
    fn on_update(&mut self, _: Self::Input<'_>, v: &u32) {
        log(format!("Hr.on Hb {v}"));
    }
}
impl UpdateHandler<Hs> for Hr {
    fn on_update(&mut self, _: Self::Input<'_>, v: &u32) {
        log(format!("Hr.on Hs {v}"));
    }
}
/// A handler with the handle component itself as a parent (run through the split of the states).
#[derive(ComponentSubscription)]
#[subscribe(p: Pk, h: Hf, b: Hb)]
pub struct Hh;
impl Component for Hh {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (Hh, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u32 {
        let h = match i.h {
            HV::Big(x) => *x,
            HV::Small(n) => n,
        };
        log(format!("Hh.update h={h} b={}", i.b));
        0
    }
}
impl UpdateHandler<Pk> for Hh {
    const ACTIVE: bool = false;
}
impl UpdateHandler<Hf> for Hh {
    fn on_update(&mut self, _: Self::Input<'_>, v: HV<'_>) {
        log(format!("Hh.on Hf {}", match v { HV::Big(x) => *x, HV::Small(n) => n }));
    }
}
impl UpdateHandler<Hb> for Hh {
    fn on_update(&mut self, _: Self::Input<'_>, v: &u32) {
        log(format!("Hh.on Hb {v}"));
    }
}

#[test]
fn a_handle_source_is_asked_again() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Pk, (Hr, Hh)>::build(Configs::new()).unwrap();
    take();
    p.push(Pk { a: 3, b: 0 });
    assert_eq!(take(), ["Hf 3", "Hs 3", "Hr.on Hs 3", "Hr.update b=0 s=3", "Hh.on Hf 3", "Hh.update h=3 b=0"]);
    p.push(Pk { a: 15, b: 0 });
    assert_eq!(take(), ["Hf 15", "Hb 15", "Hr.on Hb 15", "Hr.update b=15 s=3", "Hh.on Hf 15", "Hh.on Hb 15", "Hh.update h=15 b=15"]);
    // Hf did not run: its view is still `Big(15)`, which must not run Hb again.
    p.push(Pk { a: 4, b: 0 });
    assert_eq!(take(), ["Hr.update b=15 s=3", "Hh.update h=15 b=15"]);
    p.push(Pk { a: 6, b: 0 });
    assert_eq!(take(), ["Hf 6", "Hs 6", "Hr.on Hs 6", "Hr.update b=15 s=6", "Hh.on Hf 6", "Hh.update h=6 b=15"]);
}

// ---------------------------------------------------------------------------
// A wide handler (26 entries, 24 or more) and a handler over a `for` catalog
// ---------------------------------------------------------------------------

/// Matches when `a` is a multiple of `M`.
pub struct Mult<const M: u32>;
impl<const M: u32> Select<Pk> for Mult<M> {
    type Out = u32;
    const KEY: u32 = GUARD;
    fn select(x: &Pk) -> Option<&u32> {
        (x.a % M == 0).then_some(&x.a)
    }
    fn key_of(_: &Pk) -> u32 {
        GUARD
    }
}
macro_rules! wide {
    ($($w:ident $m:literal),*) => {
        $(comp!($w, [Mult::<$m>(x): Pk], |i| *i.x);
          impl UpdateHandler<$w> for Wide {
              fn on_update(&mut self, _: Self::Input<'_>, _: &u32) {
                  log(format!("Wide.on {}", $m));
              }
          })*
    };
}
#[derive(ComponentSubscription)]
#[subscribe(p: Pk, w1: W1, w2: W2, w3: W3, w4: W4, w5: W5, w6: W6, w7: W7, w8: W8, w9: W9, w10: W10, w11: W11, w12: W12, w13: W13, w14: W14, w15: W15, w16: W16, w17: W17, w18: W18, w19: W19, w20: W20, w21: W21, w22: W22, w23: W23, w24: W24, w25: W25)]
pub struct Wide;
impl Component for Wide {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (Wide, 0)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u32 {
        log("Wide.update");
        0
    }
}
impl UpdateHandler<Pk> for Wide {
    const ACTIVE: bool = false;
}
wide!(W1 1, W2 2, W3 3, W4 4, W5 5, W6 6, W7 7, W8 8, W9 9, W10 10, W11 11, W12 12, W13 13, W14 14, W15 15, W16 16, W17 17, W18 18, W19 19, W20 20, W21 21, W22 22, W23 23, W24 24, W25 25);

#[test]
fn a_wide_handler_calls_only_the_edges_that_fired() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Pk, (Wide,)>::build(Configs::new()).unwrap();
    take();
    let ons = |l: Vec<String>| l.into_iter().filter(|s| s.starts_with("Wide")).collect::<Vec<_>>();
    // 6: multiples 1, 2, 3, 6, in the parents' order, then update.
    p.push(Pk { a: 6, b: 0 });
    assert_eq!(ons(take()), ["Wide.on 1", "Wide.on 2", "Wide.on 3", "Wide.on 6", "Wide.update"]);
    p.push(Pk { a: 7, b: 0 });
    assert_eq!(ons(take()), ["Wide.on 1", "Wide.on 7", "Wide.update"]);
    // 0 is a multiple of everything.
    p.push(Pk { a: 0, b: 0 });
    let l = ons(take());
    assert_eq!(l.len(), 26);
    assert_eq!((l[0].as_str(), l[24].as_str(), l[25].as_str()), ("Wide.on 1", "Wide.on 25", "Wide.update"));
}

use dataflow_flat::catalog;
use std::marker::PhantomData;

pub trait Symbol: 'static {
    const KEY: u32;
}
pub struct Sy<const K: u32>;
impl<const K: u32> Symbol for Sy<K> {
    const KEY: u32 = K;
}
/// Matches when `t` is the symbol's key (a keyed selector: different keys exclude one another).
pub struct Pick<S>(PhantomData<fn() -> S>);
impl<S: Symbol> Select<Pk> for Pick<S> {
    type Out = u32;
    const KEY: u32 = S::KEY;
    fn select(x: &Pk) -> Option<&u32> {
        (x.a % 4 == S::KEY).then_some(&x.b)
    }
    fn key_of(x: &Pk) -> u32 {
        x.a % 4
    }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Pick::<S>(u): Pk)]
pub struct Qt<S: Symbol>(PhantomData<S>);
impl<S: Symbol> Component for Qt<S> {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (Qt(PhantomData), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u32 {
        *i.u * 2
    }
}
/// Always runs (a plain root entry) and handles a `for` entry per symbol.
#[derive(ComponentSubscription)]
#[subscribe(p: Pk, for S: Symbol in catalog![Sy<0>, Sy<1>, Sy<2>] { Qt<S> })]
pub struct ForH {
    got: Vec<(u32, u32)>,
}
impl Component for ForH {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (ForH { got: Vec::new() }, 0)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u32 {
        log(format!("ForH{:?}", self.got));
        self.got.clear();
        0
    }
}
impl UpdateHandler<Pk> for ForH {
    const ACTIVE: bool = false;
}
impl<S: Symbol> UpdateHandler<Qt<S>> for ForH {
    fn on_update(&mut self, _: Self::Input<'_>, v: &u32) {
        self.got.push((S::KEY, *v));
    }
}

#[test]
fn a_for_entry_handler_calls_only_the_edge_that_fired() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Pk, (ForH,)>::build(Configs::new()).unwrap();
    take();
    p.push(Pk { a: 1, b: 5 });
    assert_eq!(take(), ["ForH[(1, 10)]"]);
    p.push(Pk { a: 6, b: 7 });
    assert_eq!(take(), ["ForH[(2, 14)]"]);
    // 3 is outside the catalog: nothing matches, the handler only updates.
    p.push(Pk { a: 3, b: 1 });
    assert_eq!(take(), ["ForH[]"]);
}

// ---------------------------------------------------------------------------
// A chain of guards longer than the nesting cap of guard blocks (`MAX_IF_DEPTH`), with an
// always-running component in the middle of it (order splits the chain: the part after it asks
// again), and a handler at the end. The log is compared with a model.
// ---------------------------------------------------------------------------

/// Matches an output that is not 6 modulo 7.
pub struct Not6;
impl Select<u32> for Not6 {
    type Out = u32;
    const KEY: u32 = GUARD;
    fn select(x: &u32) -> Option<&u32> {
        (*x % 7 != 6).then_some(x)
    }
    fn key_of(_: &u32) -> u32 {
        GUARD
    }
}
/// Matches when `a` is even.
pub struct Even;
impl Select<Pk> for Even {
    type Out = u32;
    const KEY: u32 = GUARD;
    fn select(x: &Pk) -> Option<&u32> {
        (x.a % 2 == 0).then_some(&x.a)
    }
    fn key_of(_: &Pk) -> u32 {
        GUARD
    }
}
comp!(K0, [Even(x): Pk], |i| *i.x);
comp!(K1, [Not6(x): K0], |i| *i.x + 1);
comp!(K2, [Not6(x): K1], |i| *i.x + 2);
comp!(K3, [Not6(x): K2], |i| *i.x + 3);
comp!(K4, [Not6(x): K3], |i| *i.x + 4);
comp!(K5, [Not6(x): K4], |i| *i.x + 5);
comp!(K6, [Not6(x): K5], |i| *i.x + 6);
comp!(K7, [Not6(x): K6], |i| *i.x + 7);
comp!(K8, [Not6(x): K7], |i| *i.x + 8);
comp!(K9, [Not6(x): K8], |i| *i.x + 9);
comp!(K10, [Not6(x): K9], |i| *i.x + 10);
comp!(K11, [Not6(x): K10], |i| *i.x + 11);
comp!(K12, [Not6(x): K11], |i| *i.x + 12);
comp!(K13, [Not6(x): K12], |i| *i.x + 13);

/// Always runs, after `K3` and before `K4` (the sink lists it before `K13`): the chain is split by
/// it. Handles `K3`.
#[derive(ComponentSubscription)]
#[subscribe(p: Pk, k3: K3)]
pub struct Rs;
impl Component for Rs {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (Rs, 0)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u32 {
        log("Rs.update");
        0
    }
}
impl UpdateHandler<Pk> for Rs {
    const ACTIVE: bool = false;
}
impl UpdateHandler<K3> for Rs {
    fn on_update(&mut self, _: Self::Input<'_>, v: &u32) {
        log(format!("Rs.on {v}"));
    }
}

#[derive(ComponentSubscription)]
#[subscribe(p: Pk, r: Rs, e: K13)]
pub struct Hc;
impl Component for Hc {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (Hc, 0)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u32 {
        log("Hc.update");
        0
    }
}
impl UpdateHandler<Pk> for Hc {
    const ACTIVE: bool = false;
}
impl UpdateHandler<Rs> for Hc {
    const ACTIVE: bool = false;
}
impl UpdateHandler<K13> for Hc {
    fn on_update(&mut self, _: Self::Input<'_>, v: &u32) {
        log(format!("Hc.on {v}"));
    }
}

#[test]
fn a_long_chain_with_an_order_split() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Pk, (Hc,)>::build(Configs::new()).unwrap();
    take();
    let mut out = [0u32; 14];
    for a in (0..300u32).map(|k| k.wrapping_mul(2654435761) >> 7) {
        let mut want: Vec<String> = Vec::new();
        let mut prev_ran = false;
        let mut out_ran3 = false;
        for k in 0..14usize {
            if k == 4 {
                if out_ran3 {
                    want.push(format!("Rs.on {}", out[3]));
                }
                want.push("Rs.update".into());
            }
            let ran = if k == 0 { a % 2 == 0 } else { prev_ran && out[k - 1] % 7 != 6 };
            if ran {
                out[k] = if k == 0 { a } else { out[k - 1] + k as u32 };
                want.push(format!("K{k}"));
            }
            prev_ran = ran;
            if k == 3 {
                out_ran3 = ran;
            }
        }
        if prev_ran {
            want.push(format!("Hc.on {}", out[13]));
        }
        want.push("Hc.update".into());
        p.push(Pk { a, b: 0 });
        assert_eq!(take(), want, "a = {a}");
    }
}
