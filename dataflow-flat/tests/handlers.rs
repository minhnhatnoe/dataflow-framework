//! Handler components (`#[derive(Handler)]`, `Collect`, `UpdateHandler`): only
//! the edges that fired reach `on_update`, in execution order; `finish` once per
//! step; the initial output is built through the handlers; plain inputs in
//! `finish`; lifecycle; order across a mix with regular components; selected
//! handled edges. No `#![feature]` here.

use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use dataflow_flat::{
    Collect, Component, ComponentSubscription, Configs, Demux, Handler, Keyed, On, Pipeline, Root, Symbol,
    UpdateHandler,
};

static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SERIAL: Mutex<()> = Mutex::new(());

fn take() -> Vec<String> {
    std::mem::take(&mut *LOG.lock().unwrap())
}
fn log(s: impl Into<String>) {
    LOG.lock().unwrap().push(s.into());
}

macro_rules! symbol {
    ($name:ident, $key:expr) => {
        pub struct $name;
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

pub struct Frame {
    key: Option<u32>,
    v: u64,
}
impl Keyed for Frame {
    type Item = u64;
    fn key(&self) -> Option<u32> {
        self.key
    }
    fn item(&self) -> &u64 {
        &self.v
    }
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
        Some(Frame { key: i.m.key, v: i.m.v })
    }
}

/// A quoter per symbol; its initial output is 1000 + key.
#[derive(ComponentSubscription)]
#[subscribe(Some(On::<S>(u)): Ws)]
pub struct Quoter<S: Symbol>(PhantomData<S>);
impl<S: Symbol> Component for Quoter<S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        log(format!("Quoter<{}>.new", S::NAME));
        (Quoter(PhantomData), 1000 + S::KEY as u64)
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

#[derive(Handler)]
#[handles(Quoter<Btc>, Quoter<Eth>, Quoter<Sol>)]
pub struct Oms {
    scratch: Vec<String>,
}
impl Collect for Oms {
    type Config = ();
    type Output = Vec<String>;
    type Buf = Vec<String>;
    fn new((): ()) -> Self {
        log("Oms.new");
        NEWS.fetch_add(1, Ordering::SeqCst);
        LIVE.fetch_add(1, Ordering::SeqCst);
        Oms { scratch: Vec::new() }
    }
    fn begin(&mut self) -> Vec<String> {
        log("Oms.begin");
        std::mem::take(&mut self.scratch)
    }
    fn finish(&mut self, buf: Vec<String>, _: Self::Input<'_>) -> Vec<String> {
        log("Oms.finish");
        buf
    }
}
impl Drop for Oms {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}
impl<S: Symbol> UpdateHandler<Quoter<S>> for Oms {
    fn on_update(&mut self, buf: &mut Vec<String>, v: &u64) {
        log(format!("Oms.on<{}>", S::NAME));
        buf.push(format!("{}={}", S::NAME, v));
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
        ["Ws.update", "Quoter<Eth>", "Oms.begin", "Oms.on<Eth>", "Oms.finish", r#"Report["Eth=10"]"#]
    );
    p.push(Msg { key: Some(0), v: 3 });
    assert_eq!(
        take(),
        ["Ws.update", "Quoter<Btc>", "Oms.begin", "Oms.on<Btc>", "Oms.finish", r#"Report["Btc=6"]"#]
    );
    // No quoter fires: the handler does not run, Report neither.
    p.push(Msg { key: Some(9), v: 1 });
    assert_eq!(take(), ["Ws.update"]);
    p.push(Msg { key: None, v: 1 });
    assert_eq!(take(), ["Ws.update"]);
}

#[test]
fn finish_runs_once_per_step() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = P::build(Configs::new()).unwrap();
    take();
    for k in [0, 1, 2, 0, 9, 2] {
        p.push(Msg { key: Some(k), v: 1 });
    }
    let l = take();
    let count = |s: &str| l.iter().filter(|x| *x == s).count();
    assert_eq!(count("Oms.begin"), 5);
    assert_eq!(count("Oms.finish"), 5);
    assert_eq!(count("Oms.on<Btc>"), 2);
    assert_eq!(count("Oms.on<Eth>"), 1);
    assert_eq!(count("Oms.on<Sol>"), 2);
}

#[test]
fn the_initial_output_is_built_through_the_handlers() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    take();
    let _p = P::build(Configs::new()).unwrap();
    let l = take();
    // Quoters hold initial outputs 1000 + key; the handler saw each, in listing order,
    // between its begin and finish; Report's `new` got the result.
    let at = |s: &str| l.iter().position(|x| x == s).unwrap_or_else(|| panic!("{s} in {l:?}"));
    assert!(at("Quoter<Sol>.new") < at("Oms.new"));
    assert!(at("Oms.new") < at("Oms.begin"));
    assert!(at("Oms.begin") < at("Oms.on<Btc>"));
    assert!(at("Oms.on<Btc>") < at("Oms.on<Eth>") && at("Oms.on<Eth>") < at("Oms.on<Sol>"));
    assert!(at("Oms.on<Sol>") < at("Oms.finish"));
    assert!(at("Oms.finish") < at(r#"Report.new["Btc=1000", "Eth=1001", "Sol=1002"]"#));
    // Nothing ran as a step.
    assert!(!l.iter().any(|x| x == "Ws.update"));
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
// Plain inputs in finish, and a handler whose run is not known statically
// ---------------------------------------------------------------------------

/// Joins the three quoters (known statically only partly) and reads `Limits`,
/// which runs on every push: the handler runs every step, its edges are
/// tested at run time.
#[derive(Handler)]
#[handles(Quoter<Btc>, Quoter<Eth>)]
#[subscribe(limits: Limits)]
pub struct OmsL;
impl Collect for OmsL {
    type Config = ();
    type Output = (u64, usize);
    type Buf = (u64, usize);
    fn new((): ()) -> Self {
        OmsL
    }
    fn begin(&mut self) -> (u64, usize) {
        log("OmsL.begin");
        (0, 0)
    }
    fn finish(&mut self, buf: (u64, usize), i: Self::Input<'_>) -> (u64, usize) {
        log("OmsL.finish");
        (buf.0 + *i.limits, buf.1)
    }
}
impl<S: Symbol> UpdateHandler<Quoter<S>> for OmsL {
    fn on_update(&mut self, buf: &mut (u64, usize), v: &u64) {
        log(format!("OmsL.on<{}>", S::NAME));
        buf.0 += *v;
        buf.1 += 1;
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
fn plain_inputs_reach_finish_and_unfired_edges_are_skipped() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    take();
    let mut p = Pipeline::<Msg, (ReportL,)>::build(Configs::new()).unwrap();
    // Initial: limits 7 + 1000 + 1001.
    assert!(take().contains(&"ReportL.new(2008, 2)".to_string()), "initial output");
    p.push(Msg { key: Some(1), v: 5 });
    assert_eq!(
        take(),
        ["Ws.update", "Quoter<Eth>", "Limits", "OmsL.begin", "OmsL.on<Eth>", "OmsL.finish", "ReportL(17, 1)"]
    );
    // Nothing handled fired, but `limits` did: the handler still runs.
    p.push(Msg { key: Some(2), v: 5 });
    assert_eq!(take(), ["Ws.update", "Limits", "OmsL.begin", "OmsL.finish", "ReportL(7, 0)"]);
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
#[derive(Handler)]
#[handles(Gamma, Beta)]
#[handles(Alpha)]
pub struct Mix;
impl Collect for Mix {
    type Config = ();
    type Output = u64;
    type Buf = u64;
    fn new((): ()) -> Self {
        Mix
    }
    fn begin(&mut self) -> u64 {
        log("Mix.begin");
        0
    }
    fn finish(&mut self, buf: u64, _: Self::Input<'_>) -> u64 {
        log("Mix.finish");
        buf
    }
}
macro_rules! handles {
    ($($t:ident),*) => { $(impl UpdateHandler<$t> for Mix {
        fn on_update(&mut self, buf: &mut u64, v: &u64) {
            log(concat!("Mix.on ", stringify!($t)));
            *buf += *v;
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
        ["Alpha", "Gamma", "Beta", "Mix.begin", "Mix.on Alpha", "Mix.on Gamma", "Mix.on Beta", "Mix.finish", "Post"]
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
#[derive(Handler)]
#[handles(OnA, OnB)]
#[subscribe(every: Every)]
pub struct Flags;
impl Collect for Flags {
    type Config = ();
    type Output = Vec<String>;
    type Buf = Vec<String>;
    fn new((): ()) -> Self {
        Flags
    }
    fn begin(&mut self) -> Vec<String> {
        Vec::new()
    }
    fn finish(&mut self, buf: Vec<String>, i: Self::Input<'_>) -> Vec<String> {
        let mut buf = buf;
        buf.push(format!("every={}", i.every));
        buf
    }
}
impl UpdateHandler<OnA> for Flags {
    fn on_update(&mut self, buf: &mut Vec<String>, v: &u32) {
        buf.push(format!("a={v}"));
    }
}
impl UpdateHandler<OnB> for Flags {
    fn on_update(&mut self, buf: &mut Vec<String>, v: &u32) {
        buf.push(format!("b={v}"));
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

#[derive(Handler)]
#[handles(Some(On::<Btc>(btc)): Ws, Some(On::<Eth>(eth)): Ws)]
#[handles(Quoter<Sol>)]
pub struct Fills;
impl Collect for Fills {
    type Config = ();
    type Output = Vec<String>;
    type Buf = Vec<String>;
    fn new((): ()) -> Self {
        Fills
    }
    fn begin(&mut self) -> Vec<String> {
        log("Fills.begin");
        Vec::new()
    }
    fn finish(&mut self, buf: Vec<String>, _: Self::Input<'_>) -> Vec<String> {
        log("Fills.finish");
        buf
    }
}
impl UpdateHandler<fills_edges::Btc> for Fills {
    fn on_update(&mut self, buf: &mut Vec<String>, v: &u64) {
        log("Fills.btc");
        buf.push(format!("btc:{v}"));
    }
}
impl UpdateHandler<fills_edges::Eth> for Fills {
    fn on_update(&mut self, buf: &mut Vec<String>, v: &u64) {
        log("Fills.eth");
        buf.push(format!("eth:{v}"));
    }
}
impl UpdateHandler<Quoter<Sol>> for Fills {
    fn on_update(&mut self, buf: &mut Vec<String>, v: &u64) {
        log("Fills.sol");
        buf.push(format!("sol:{v}"));
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
    // Selected edges have no initial value; the plain one (Quoter<Sol>, 1002) has.
    assert!(take().contains(&r#"FillsOut.new["sol:1002"]"#.to_string()));
    p.push(Msg { key: Some(0), v: 4 });
    assert_eq!(take(), ["Ws.update", "Fills.begin", "Fills.btc", "Fills.finish", r#"FillsOut["btc:4"]"#]);
    p.push(Msg { key: Some(1), v: 6 });
    assert_eq!(take(), ["Ws.update", "Fills.begin", "Fills.eth", "Fills.finish", r#"FillsOut["eth:6"]"#]);
    p.push(Msg { key: Some(2), v: 9 });
    assert_eq!(
        take(),
        ["Ws.update", "Quoter<Sol>", "Fills.begin", "Fills.sol", "Fills.finish", r#"FillsOut["sol:18"]"#]
    );
    p.push(Msg { key: Some(7), v: 1 });
    assert_eq!(take(), ["Ws.update"]);
}
