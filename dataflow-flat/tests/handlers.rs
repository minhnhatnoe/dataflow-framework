//! Handler components (handled entries in `#[subscribe]`, `UpdateHandler`):
//! only the edges that fired reach `on_update`, in execution order; `update`
//! once per push, after them; nothing is called at build; named inputs;
//! lifecycle; order across a mix with regular components; selected handled
//! edges. No `#![feature]` here.

use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use dataflow_flat::{
    Component, ComponentSubscription, Configs, Demux, Pipeline, Root, Select, UpdateHandler,
};

static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SERIAL: Mutex<()> = Mutex::new(());

fn take() -> Vec<String> {
    std::mem::take(&mut *LOG.lock().unwrap())
}
fn log(s: impl Into<String>) {
    LOG.lock().unwrap().push(s.into());
}

/// The symbols, as marker types shared by `Frame`'s variants.
pub mod syms {
    pub struct Btc;
    pub struct Eth;
    pub struct Sol;
    pub struct Other;
}
use syms::{Btc, Eth, Sol};

/// A symbol: its marker selects its variant of `Frame`.
pub trait Symbol: Select<Frame, Out = u64> {
    const KEY: u32;
    const NAME: &'static str;
}
macro_rules! symbol {
    ($name:ident, $key:expr) => {
        impl Symbol for $name {
            const KEY: u32 = $key;
            const NAME: &'static str = stringify!($name);
        }
    };
}
symbol!(Btc, 0);
symbol!(Eth, 1);
symbol!(Sol, 2);

/// A message of a multiplexed feed.
#[derive(Root)]
pub struct Msg {
    pub key: Option<u32>,
    pub v: u64,
}

#[derive(Demux)]
#[demux(markers = syms)]
pub enum Frame {
    Btc(u64),
    Eth(u64),
    Sol(u64),
    /// Any other key, or none: nobody selects it.
    Other(u64),
}

#[derive(ComponentSubscription)]
#[subscribe(m: Msg)]
pub struct Ws;
impl Component for Ws {
    type Config = ();
    type Output = Option<Frame>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Option<Frame>) {
        log("Ws.new");
        (Ws, None)
    }
    fn update(&mut self, i: Self::Input<'_>) -> Option<Frame> {
        log("Ws.update");
        Some(match i.m.key {
            Some(0) => Frame::Btc(i.m.v),
            Some(1) => Frame::Eth(i.m.v),
            Some(2) => Frame::Sol(i.m.v),
            _ => Frame::Other(i.m.v),
        })
    }
}

/// A quoter per symbol; its initial output is 1000 + key.
#[derive(ComponentSubscription)]
#[subscribe(Some(S(u)): Ws)]
pub struct Quoter<S: Symbol>(PhantomData<S>);
impl<S: Symbol> Component for Quoter<S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        log(format!("Quoter<{}>.new", S::NAME));
        (Quoter(PhantomData), 1000 + <S as Symbol>::KEY as u64)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("Quoter<{}>", S::NAME));
        *i.u * 2
    }
}

/// Constant-ish plain input of the handlers below: runs on every push.
#[derive(ComponentSubscription)]
#[subscribe(m: Msg)]
pub struct Limits;
impl Component for Limits {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Limits, 7)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u64 {
        log("Limits");
        7
    }
}

static LIVE: AtomicUsize = AtomicUsize::new(0);
static NEWS: AtomicUsize = AtomicUsize::new(0);
static DROPS: AtomicUsize = AtomicUsize::new(0);

// ---------------------------------------------------------------------------
// An OMS over exclusive keys: every run is a switch leaf, edges known statically
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription)]
#[subscribe(Quoter<Btc>, Quoter<Eth>, Quoter<Sol>)]
pub struct Oms {
    pending: Vec<String>,
}
impl Component for Oms {
    type Config = ();
    type Output = Vec<String>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Vec<String>) {
        log("Oms.new");
        NEWS.fetch_add(1, Ordering::SeqCst);
        LIVE.fetch_add(1, Ordering::SeqCst);
        (Oms { pending: Vec::new() }, Vec::new())
    }
    fn update(&mut self, _: Self::Input<'_>) -> Vec<String> {
        log("Oms.update");
        std::mem::take(&mut self.pending)
    }
}
impl Drop for Oms {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}
impl<S: Symbol> UpdateHandler<Quoter<S>> for Oms {
    fn on_update(&mut self, v: &u64) {
        log(format!("Oms.on<{}>", S::NAME));
        self.pending.push(format!("{}={}", S::NAME, v));
    }
}

/// Something downstream that sees the handler's output (and its initial one).
#[derive(ComponentSubscription)]
#[subscribe(o: Oms)]
pub struct Report;
impl Component for Report {
    type Config = ();
    type Output = Vec<String>;
    fn new((): (), i: Self::Init<'_>) -> (Self, Vec<String>) {
        log(format!("Report.new{:?}", i.o));
        (Report, i.o.clone())
    }
    fn update(&mut self, i: Self::Input<'_>) -> Vec<String> {
        log(format!("Report{:?}", i.o));
        i.o.clone()
    }
}

type P = Pipeline<Msg, (Report,)>;

#[test]
fn only_the_fired_quoter_reaches_on_update() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = P::build(Configs::new()).unwrap();
    take();
    p.push(Msg { key: Some(1), v: 5 });
    assert_eq!(
        take(),
        ["Ws.update", "Quoter<Eth>", "Oms.on<Eth>", "Oms.update", r#"Report["Eth=10"]"#]
    );
    p.push(Msg { key: Some(0), v: 3 });
    assert_eq!(
        take(),
        ["Ws.update", "Quoter<Btc>", "Oms.on<Btc>", "Oms.update", r#"Report["Btc=6"]"#]
    );
    // No quoter fires: the handler does not run, Report neither.
    p.push(Msg { key: Some(9), v: 1 });
    assert_eq!(take(), ["Ws.update"]);
    p.push(Msg { key: None, v: 1 });
    assert_eq!(take(), ["Ws.update"]);
}

#[test]
fn update_runs_once_per_push() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = P::build(Configs::new()).unwrap();
    take();
    for k in [0, 1, 2, 0, 9, 2] {
        p.push(Msg { key: Some(k), v: 1 });
    }
    let l = take();
    let count = |s: &str| l.iter().filter(|x| *x == s).count();
    assert_eq!(count("Oms.update"), 5);
    assert_eq!(count("Oms.on<Btc>"), 2);
    assert_eq!(count("Oms.on<Eth>"), 1);
    assert_eq!(count("Oms.on<Sol>"), 2);
}

#[test]
fn nothing_is_called_at_build() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    take();
    let _p = P::build(Configs::new()).unwrap();
    let l = take();
    // Quoters hold initial outputs 1000 + key, but the handler is not fed them:
    // `new` returned the initial output (empty), which Report's `new` got.
    let at = |s: &str| l.iter().position(|x| x == s).unwrap_or_else(|| panic!("{s} in {l:?}"));
    assert!(at("Quoter<Sol>.new") < at("Oms.new"));
    assert!(at("Oms.new") < at("Report.new[]"));
    assert!(!l.iter().any(|x| x.starts_with("Oms.on") || x == "Oms.update" || x == "Ws.update"), "{l:?}");
}

#[test]
fn lifecycle_new_once_dropped_once() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let (n0, d0, l0) = (NEWS.load(Ordering::SeqCst), DROPS.load(Ordering::SeqCst), LIVE.load(Ordering::SeqCst));
    {
        let mut p = P::build(Configs::new()).unwrap();
        for k in 0..6 {
            p.push(Msg { key: Some(k % 3), v: 1 });
        }
        assert_eq!(NEWS.load(Ordering::SeqCst), n0 + 1);
        assert_eq!(LIVE.load(Ordering::SeqCst), l0 + 1);
        assert_eq!(DROPS.load(Ordering::SeqCst), d0);
    }
    assert_eq!(DROPS.load(Ordering::SeqCst), d0 + 1);
    assert_eq!(LIVE.load(Ordering::SeqCst), l0);
    take();
}

// ---------------------------------------------------------------------------
// Named inputs in update, and a handler whose run is not known statically
// ---------------------------------------------------------------------------

/// Joins the three quoters (known statically only partly) and reads `Limits`,
/// which runs on every push: the handler runs every step, its edges are
/// tested at run time.
#[derive(ComponentSubscription)]
#[subscribe(Quoter<Btc>, Quoter<Eth>, limits: Limits)]
pub struct OmsL {
    acc: (u64, usize),
}
impl Component for OmsL {
    type Config = ();
    type Output = (u64, usize);
    fn new((): (), i: Self::Init<'_>) -> (Self, (u64, usize)) {
        (OmsL { acc: (0, 0) }, (*i.limits, 0))
    }
    fn update(&mut self, i: Self::Input<'_>) -> (u64, usize) {
        log("OmsL.update");
        let acc = std::mem::take(&mut self.acc);
        (acc.0 + *i.limits, acc.1)
    }
}
impl<S: Symbol> UpdateHandler<Quoter<S>> for OmsL {
    fn on_update(&mut self, v: &u64) {
        log(format!("OmsL.on<{}>", S::NAME));
        self.acc.0 += *v;
        self.acc.1 += 1;
    }
}

#[derive(ComponentSubscription)]
#[subscribe(o: OmsL)]
pub struct ReportL;
impl Component for ReportL {
    type Config = ();
    type Output = ();
    fn new((): (), i: Self::Init<'_>) -> (Self, ()) {
        log(format!("ReportL.new{:?}", i.o));
        (ReportL, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        log(format!("ReportL{:?}", i.o));
    }
}

#[test]
fn named_inputs_reach_update_and_unfired_edges_are_skipped() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    take();
    let mut p = Pipeline::<Msg, (ReportL,)>::build(Configs::new()).unwrap();
    // Initial: `new` alone (limits 7), the quoters' initial outputs are not replayed.
    assert!(take().contains(&"ReportL.new(7, 0)".to_string()), "initial output");
    p.push(Msg { key: Some(1), v: 5 });
    assert_eq!(
        take(),
        ["Ws.update", "Quoter<Eth>", "Limits", "OmsL.on<Eth>", "OmsL.update", "ReportL(17, 1)"]
    );
    // Nothing handled fired, but `limits` did: the handler still runs.
    p.push(Msg { key: Some(2), v: 5 });
    assert_eq!(take(), ["Ws.update", "Limits", "OmsL.update", "ReportL(7, 0)"]);
}

// ---------------------------------------------------------------------------
// Order across a mix of regular components and handled edges
// ---------------------------------------------------------------------------

macro_rules! comp {
    ($name:ident, [$($sub:tt)*]) => {
        #[derive(ComponentSubscription)]
        #[subscribe($($sub)*)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
                ($name, 0)
            }
            fn update(&mut self, _: Self::Input<'_>) -> u64 {
                log(stringify!($name));
                1
            }
        }
    };
}
comp!(Alpha, [m: Msg]);
comp!(Beta, [m: Msg]);
comp!(Gamma, [a: Alpha]);
comp!(Post, [mix: Mix]);

/// Listing order is Gamma, Beta, Alpha, but Alpha runs first (Gamma needs it):
/// `on_update` follows execution order.
#[derive(ComponentSubscription)]
#[subscribe(Gamma, Beta)]
#[subscribe(Alpha)]
pub struct Mix {
    acc: u64,
}
impl Component for Mix {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Mix { acc: 0 }, 0)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u64 {
        log("Mix.update");
        std::mem::take(&mut self.acc)
    }
}
macro_rules! handles {
    ($($t:ident),*) => { $(impl UpdateHandler<$t> for Mix {
        fn on_update(&mut self, v: &u64) {
            log(concat!("Mix.on ", stringify!($t)));
            self.acc += *v;
        }
    })* };
}
handles!(Alpha, Beta, Gamma);

#[test]
fn execution_order_holds_across_regular_components_and_handled_edges() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Msg, (Post,)>::build(Configs::new()).unwrap();
    take();
    p.push(Msg { key: None, v: 0 });
    assert_eq!(
        take(),
        ["Alpha", "Gamma", "Beta", "Mix.on Alpha", "Mix.on Gamma", "Mix.on Beta", "Mix.update", "Post"]
    );
}

// ---------------------------------------------------------------------------
// Conditional parents with the handler always running: flags at run time
// ---------------------------------------------------------------------------

#[derive(Root, Demux)]
pub enum Pkt {
    A(u32),
    B(u32),
    Tick,
}

#[derive(ComponentSubscription)]
#[subscribe(pkt::A(p): Pkt)]
pub struct OnA;
impl Component for OnA {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (OnA, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u32 {
        log("OnA");
        *i.p
    }
}
#[derive(ComponentSubscription)]
#[subscribe(pkt::B(p): Pkt)]
pub struct OnB;
impl Component for OnB {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (OnB, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u32 {
        log("OnB");
        *i.p
    }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Pkt)]
pub struct Every;
impl Component for Every {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (Every, 0)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u32 {
        7
    }
}

/// Always runs (`every` fires on each push); `OnA` and `OnB` are tested by flag.
#[derive(ComponentSubscription)]
#[subscribe(every: Every, OnA, OnB)]
pub struct Flags {
    acc: Vec<String>,
}
impl Component for Flags {
    type Config = ();
    type Output = Vec<String>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Vec<String>) {
        (Flags { acc: Vec::new() }, Vec::new())
    }
    fn update(&mut self, i: Self::Input<'_>) -> Vec<String> {
        self.acc.push(format!("every={}", i.every));
        std::mem::take(&mut self.acc)
    }
}
impl UpdateHandler<OnA> for Flags {
    fn on_update(&mut self, v: &u32) {
        self.acc.push(format!("a={v}"));
    }
}
impl UpdateHandler<OnB> for Flags {
    fn on_update(&mut self, v: &u32) {
        self.acc.push(format!("b={v}"));
    }
}
#[derive(ComponentSubscription)]
#[subscribe(f: Flags)]
pub struct FlagsOut;
pub static SEEN: Mutex<Vec<Vec<String>>> = Mutex::new(Vec::new());
impl Component for FlagsOut {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (FlagsOut, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        SEEN.lock().unwrap().push(i.f.clone());
    }
}

#[test]
fn edges_with_conditional_parents_are_tested_by_flag() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Pkt, (FlagsOut,)>::build(Configs::new()).unwrap();
    SEEN.lock().unwrap().clear();
    p.push(Pkt::A(3));
    p.push(Pkt::B(4));
    p.push(Pkt::Tick);
    p.push(Pkt::A(5));
    let seen = SEEN.lock().unwrap().clone();
    assert_eq!(
        seen,
        vec![
            vec!["a=3", "every=7"],
            vec!["b=4", "every=7"],
            vec!["every=7"],
            vec!["a=5", "every=7"],
        ]
    );
    take();
}

// ---------------------------------------------------------------------------
// Selected handled edges
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription)]
#[subscribe(Some(Btc(_)): Ws, Some(Eth(_)): Ws, Quoter<Sol>)]
pub struct Fills {
    acc: Vec<String>,
}
impl Component for Fills {
    type Config = ();
    type Output = Vec<String>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Vec<String>) {
        (Fills { acc: Vec::new() }, Vec::new())
    }
    fn update(&mut self, _: Self::Input<'_>) -> Vec<String> {
        log("Fills.update");
        std::mem::take(&mut self.acc)
    }
}
impl UpdateHandler<fills_edges::Btc> for Fills {
    fn on_update(&mut self, v: &u64) {
        log("Fills.btc");
        self.acc.push(format!("btc:{v}"));
    }
}
impl UpdateHandler<fills_edges::Eth> for Fills {
    fn on_update(&mut self, v: &u64) {
        log("Fills.eth");
        self.acc.push(format!("eth:{v}"));
    }
}
impl UpdateHandler<Quoter<Sol>> for Fills {
    fn on_update(&mut self, v: &u64) {
        log("Fills.sol");
        self.acc.push(format!("sol:{v}"));
    }
}

#[derive(ComponentSubscription)]
#[subscribe(f: Fills)]
pub struct FillsOut;
impl Component for FillsOut {
    type Config = ();
    type Output = ();
    fn new((): (), i: Self::Init<'_>) -> (Self, ()) {
        log(format!("FillsOut.new{:?}", i.f));
        (FillsOut, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        log(format!("FillsOut{:?}", i.f));
    }
}

#[test]
fn a_selected_handled_edge() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    take();
    let mut p = Pipeline::<Msg, (FillsOut,)>::build(Configs::new()).unwrap();
    // No handler is called at build: the initial output is what `new` returned.
    assert!(take().contains(&"FillsOut.new[]".to_string()));
    p.push(Msg { key: Some(0), v: 4 });
    assert_eq!(take(), ["Ws.update", "Fills.btc", "Fills.update", r#"FillsOut["btc:4"]"#]);
    p.push(Msg { key: Some(1), v: 6 });
    assert_eq!(take(), ["Ws.update", "Fills.eth", "Fills.update", r#"FillsOut["eth:6"]"#]);
    p.push(Msg { key: Some(2), v: 9 });
    assert_eq!(
        take(),
        ["Ws.update", "Quoter<Sol>", "Fills.sol", "Fills.update", r#"FillsOut["sol:18"]"#]
    );
    p.push(Msg { key: Some(7), v: 1 });
    assert_eq!(take(), ["Ws.update"]);
}
