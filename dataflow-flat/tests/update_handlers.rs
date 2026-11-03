//! `UpdateHandler` on any entry of `#[subscribe(..)]`, named or not: `on_update(input, v)` with the
//! same `input` `update` gets, called when the entry fired, in the execution order of the parents,
//! before `update`. Covers named and unnamed entries, selected entries, `for` entries, `input`
//! being current for entries that did not fire, handle parents (owned and borrowed views), a handle
//! component with a handler, lifecycle counts, order, an always-running handler, and
//! `#[derive(UpdateHandler)]`. No `#![feature]` here.

use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::Mutex;

use dataflow_flat::{
    catalog, Component, ComponentSubscription, Configs, Demux, Pipeline, Root, Select, UpdateHandler,
};

static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SERIAL: Mutex<()> = Mutex::new(());
static NEWS: AtomicUsize = AtomicUsize::new(0);
static DROPS: AtomicUsize = AtomicUsize::new(0);
static ONS: AtomicUsize = AtomicUsize::new(0);
static UPDATES: AtomicUsize = AtomicUsize::new(0);

fn take() -> Vec<String> {
    std::mem::take(&mut *LOG.lock().unwrap())
}
fn log(s: impl Into<String>) {
    LOG.lock().unwrap().push(s.into());
}
fn only(prefix: &str, l: Vec<String>) -> Vec<String> {
    l.into_iter().filter(|s| s.starts_with(prefix)).collect()
}

/// A drop and construction counter.
struct Life;
impl Life {
    fn new() -> Life {
        NEWS.fetch_add(1, SeqCst);
        Life
    }
}
impl Drop for Life {
    fn drop(&mut self) {
        DROPS.fetch_add(1, SeqCst);
    }
}

// ---------------------------------------------------------------------------
// The market: a multiplexed feed, a quoter per symbol, selected and plain entries
// ---------------------------------------------------------------------------

pub mod syms {
    pub struct Btc;
    pub struct Eth;
    pub struct Sol;
    pub struct Other;
}
use syms::{Btc, Eth, Sol};

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
    Other(u64),
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct Ws;
impl Component for Ws {
    type Config = ();
    type Output = Option<Frame>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Option<Frame>) {
        (Ws, None)
    }
    fn update(&mut self, i: Self::Input<'_>) -> Option<Frame> {
        log("Ws");
        Some(match i.m.key {
            Some(0) => Frame::Btc(i.m.v),
            Some(1) => Frame::Eth(i.m.v),
            Some(2) => Frame::Sol(i.m.v),
            _ => Frame::Other(i.m.v),
        })
    }
}

/// A quoter per symbol; its initial output is 1000 + key.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(S(u)): Ws)]
pub struct Quoter<S: Symbol>(PhantomData<S>);
impl<S: Symbol> Component for Quoter<S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Quoter(PhantomData), 1000 + <S as Symbol>::KEY as u64)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("Quoter<{}>", S::NAME));
        *i.u * 2
    }
}

/// Runs on every push.
#[derive(ComponentSubscription, UpdateHandler)]
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

// ---------------------------------------------------------------------------
// `input` is current: entries that did not fire hold the values `update` would see
// ---------------------------------------------------------------------------

/// `Quoter<Btc>` is unnamed with a handler; `eth` is a named plain entry and `sol` a selected one
/// (stored: Mix reads it in steps where the selector does not match), both without a call.
#[derive(ComponentSubscription)]
#[subscribe(Quoter<Btc>, eth: Quoter<Eth>, Some(Sol(sol)): Ws)]
pub struct Mix;
impl Component for Mix {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Mix, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        log(format!("Mix.update eth={} sol={}", i.eth, i.sol));
    }
}
impl UpdateHandler<Quoter<Btc>> for Mix {
    fn on_update(&mut self, i: Self::Input<'_>, v: &u64) {
        log(format!("Mix.on_update btc={} eth={} sol={}", v, i.eth, i.sol));
    }
}
impl UpdateHandler<Quoter<Eth>> for Mix {
    const ACTIVE: bool = false;
}
impl UpdateHandler<mix_edges::Sol> for Mix {
    const ACTIVE: bool = false;
}

#[test]
fn on_update_sees_the_whole_input() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    take();
    let mut p = Pipeline::<Msg, (Mix,)>::build(Configs::new()).unwrap();
    take();
    let mix = |l: Vec<String>| only("Mix", l);
    p.push(Msg { key: Some(2), v: 3 }); // Sol matches: sol = 3, eth is Quoter<Eth>'s initial output
    assert_eq!(mix(take()), ["Mix.update eth=1001 sol=3"]);
    p.push(Msg { key: Some(1), v: 5 }); // Eth: Quoter<Eth> = 10
    assert_eq!(mix(take()), ["Mix.update eth=10 sol=3"]);
    p.push(Msg { key: Some(0), v: 7 }); // Btc fires the handled edge; eth and sol are from earlier steps
    assert_eq!(mix(take()), ["Mix.on_update btc=14 eth=10 sol=3", "Mix.update eth=10 sol=3"]);
}

// ---------------------------------------------------------------------------
// Named entries with a call, selected entries with a call, an empty impl
// ---------------------------------------------------------------------------

/// Every entry has a handler: a named plain one (`ACTIVE` by default, with a body), a named one
/// whose impl is empty (`{}`: `ACTIVE`, does nothing), and a selected one keyed by its marker.
#[derive(ComponentSubscription)]
#[subscribe(btc: Quoter<Btc>, lim: Limits, Some(Eth(e)): Ws)]
pub struct Every {
    _life: Life,
}
impl Component for Every {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Every { _life: Life::new() }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        UPDATES.fetch_add(1, SeqCst);
        log(format!("Every.update btc={} lim={} e={}", i.btc, i.lim, i.e));
    }
}
impl UpdateHandler<Quoter<Btc>> for Every {
    fn on_update(&mut self, i: Self::Input<'_>, v: &u64) {
        ONS.fetch_add(1, SeqCst);
        log(format!("Every.btc v={} input.btc={}", v, i.btc));
    }
}
impl UpdateHandler<Limits> for Every {}
impl UpdateHandler<every_edges::E> for Every {
    fn on_update(&mut self, i: Self::Input<'_>, v: &u64) {
        ONS.fetch_add(1, SeqCst);
        log(format!("Every.eth v={} input.e={}", v, i.e));
    }
}

#[test]
fn named_selected_and_empty_impls() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    take();
    for c in [&NEWS, &DROPS, &ONS, &UPDATES] {
        c.store(0, SeqCst);
    }
    {
        let mut p = Pipeline::<Msg, (Every,)>::build(Configs::new()).unwrap();
        assert_eq!(NEWS.load(SeqCst), 1);
        take();
        let ev = |l: Vec<String>| only("Every", l);
        // Limits runs on every push, so `Every` does too; the empty impl for it calls nothing.
        p.push(Msg { key: Some(7), v: 1 });
        assert_eq!(ev(take()), ["Every.update btc=1000 lim=7 e=0"]);
        p.push(Msg { key: Some(0), v: 4 });
        // Btc fired: on_update with the new value, input.btc is the same value (a named entry is current).
        assert_eq!(ev(take()), ["Every.btc v=8 input.btc=8", "Every.update btc=8 lim=7 e=0"]);
        p.push(Msg { key: Some(1), v: 6 });
        // Eth matched: the selected part, then update.
        assert_eq!(ev(take()), ["Every.eth v=6 input.e=6", "Every.update btc=8 lim=7 e=6"]);
        p.push(Msg { key: Some(7), v: 6 });
        assert_eq!(ev(take()), ["Every.update btc=8 lim=7 e=6"]);
        assert_eq!(ONS.load(SeqCst), 2);
        assert_eq!(UPDATES.load(SeqCst), 4);
    }
    assert_eq!(DROPS.load(SeqCst), 1);
}

// ---------------------------------------------------------------------------
// `for` entries
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription)]
#[subscribe(lim: Limits, for S: Symbol in catalog![Btc, Eth, Sol] { Quoter<S> })]
pub struct Oms {
    got: Vec<String>,
}
impl Component for Oms {
    type Config = ();
    type Output = Vec<String>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Vec<String>) {
        (Oms { got: Vec::new() }, Vec::new())
    }
    fn update(&mut self, i: Self::Input<'_>) -> Vec<String> {
        log(format!("Oms.update lim={} got={:?}", i.lim, self.got));
        std::mem::take(&mut self.got)
    }
}
impl UpdateHandler<Limits> for Oms {
    const ACTIVE: bool = false;
}
impl<S: Symbol> UpdateHandler<Quoter<S>> for Oms {
    fn on_update(&mut self, i: Self::Input<'_>, v: &u64) {
        self.got.push(format!("{}={}/{}", S::NAME, v, i.lim));
    }
}

#[test]
fn for_entries_get_input_too() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Msg, (Oms,)>::build(Configs::new()).unwrap();
    take();
    p.push(Msg { key: Some(1), v: 5 });
    assert_eq!(only("Oms", take()), ["Oms.update lim=7 got=[\"Eth=10/7\"]"]);
    p.push(Msg { key: Some(9), v: 5 });
    assert_eq!(only("Oms", take()), ["Oms.update lim=7 got=[]"]);
}

// ---------------------------------------------------------------------------
// Order, always-running handlers, lifecycle
// ---------------------------------------------------------------------------

/// Listing order is Quoter<Eth>, Quoter<Btc>; both run in that order, then the handler. `lim` is
/// an always-running named entry whose impl is `ACTIVE = false`.
#[derive(ComponentSubscription)]
#[subscribe(Quoter<Eth>, lim: Limits, Quoter<Btc>)]
pub struct Late {
    _life: Life,
}
impl Component for Late {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Late { _life: Life::new() }, ())
    }
    fn update(&mut self, _: Self::Input<'_>) {
        UPDATES.fetch_add(1, SeqCst);
        log("Late.update");
    }
}
impl UpdateHandler<Limits> for Late {
    const ACTIVE: bool = false;
}
impl UpdateHandler<Quoter<Eth>> for Late {
    fn on_update(&mut self, _: Self::Input<'_>, v: &u64) {
        log(format!("Late.eth {v}"));
    }
}
impl UpdateHandler<Quoter<Btc>> for Late {
    fn on_update(&mut self, _: Self::Input<'_>, v: &u64) {
        log(format!("Late.btc {v}"));
    }
}

#[test]
fn order_and_always_running() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    for c in [&NEWS, &DROPS, &UPDATES] {
        c.store(0, SeqCst);
    }
    {
        let mut p = Pipeline::<Msg, (Late,)>::build(Configs::new()).unwrap();
        take();
        p.push(Msg { key: Some(0), v: 1 });
        // Listing order: Quoter<Eth> (does not fire), Limits, Quoter<Btc>; `on_update`s in that order of the parents.
        assert_eq!(take(), ["Ws", "Limits", "Quoter<Btc>", "Late.btc 2", "Late.update"]);
        p.push(Msg { key: Some(1), v: 1 });
        assert_eq!(take(), ["Ws", "Quoter<Eth>", "Limits", "Late.eth 2", "Late.update"]);
        // Neither fired: the handler still runs (Limits did), with no call.
        p.push(Msg { key: Some(5), v: 1 });
        assert_eq!(take(), ["Ws", "Limits", "Late.update"]);
        assert_eq!(UPDATES.load(SeqCst), 3);
    }
    assert_eq!((NEWS.load(SeqCst), DROPS.load(SeqCst)), (1, 1));
}

// ---------------------------------------------------------------------------
// Handle parents: the value is exactly the parent's `View<'a>`, borrowed or owned
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Root, Demux)]
pub enum Tick {
    A(u64),
    B(u64),
}
fn tv(m: &Tick) -> u64 {
    match m {
        Tick::A(v) | Tick::B(v) => *v,
    }
}

/// A ring; its view is a reference into the state. Runs on `A` only.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Tick::A(v): Tick)]
pub struct Ring {
    arr: [u64; 4],
    n: usize,
}
impl Component for Ring {
    type Config = ();
    type Handle = usize;
    type View<'a> = &'a u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, usize) {
        (Ring { arr: [0; 4], n: 0 }, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> usize {
        log("update Ring");
        self.n += 1;
        let slot = self.n % 4;
        self.arr[slot] = *i.v;
        slot
    }
    fn view<'a>(&'a self, h: &'a usize) -> &'a u64 {
        &self.arr[*h]
    }
}

/// Owned view.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(t: Tick)]
pub struct Level {
    v: u64,
}
impl Component for Level {
    type Config = ();
    type Handle = ();
    type View<'a> = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Level { v: 7 }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        log("update Level");
        self.v += tv(i.t);
    }
    fn view<'a>(&'a self, (): &'a ()) -> u64 {
        self.v
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(t: Tick)]
pub struct Twice;
impl Component for Twice {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Twice, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log("update Twice");
        tv(i.t) * 2
    }
}

/// A by-value handler over both kinds of handle parent and a by-value one (named, with a call):
/// `input` holds the views as they are now.
#[derive(ComponentSubscription)]
#[subscribe(Ring, Level, tw: Twice)]
pub struct Watch;
impl Component for Watch {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Watch, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        log(format!("Watch.update tw={}", i.tw));
    }
}
impl UpdateHandler<Ring> for Watch {
    fn on_update(&mut self, i: Self::Input<'_>, v: &u64) {
        log(format!("Watch.Ring {v} tw={}", i.tw));
    }
}
impl UpdateHandler<Level> for Watch {
    fn on_update(&mut self, i: Self::Input<'_>, v: u64) {
        log(format!("Watch.Level {v} tw={}", i.tw));
    }
}
impl UpdateHandler<Twice> for Watch {
    fn on_update(&mut self, i: Self::Input<'_>, v: &u64) {
        log(format!("Watch.Twice {v} tw={}", i.tw));
    }
}

#[test]
fn handle_parents_borrowed_and_owned() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Tick, (Watch,)>::build(Configs::new()).unwrap();
    take();
    p.push(Tick::A(5));
    assert_eq!(
        take(),
        [
            "update Ring",
            "update Level",
            "update Twice",
            "Watch.Ring 5 tw=10",
            "Watch.Level 12 tw=10",
            "Watch.Twice 10 tw=10",
            "Watch.update tw=10"
        ]
    );
    // Ring did not run: its edge did not fire, no call; the others fired.
    p.push(Tick::B(3));
    assert_eq!(
        take(),
        ["update Level", "update Twice", "Watch.Level 15 tw=6", "Watch.Twice 6 tw=6", "Watch.update tw=6"]
    );
}

/// A handle component with a handler of its own (a by-value parent, `tw`, with a call).
#[derive(ComponentSubscription)]
#[subscribe(tw: Twice, Tick::A(a): Tick)]
pub struct Acc {
    sum: u64,
}
impl Component for Acc {
    type Config = ();
    type Handle = ();
    type View<'a> = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Acc { sum: 0 }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        log(format!("Acc.update a={}", i.a));
    }
    fn view<'a>(&'a self, (): &'a ()) -> u64 {
        self.sum
    }
}
impl UpdateHandler<Twice> for Acc {
    fn on_update(&mut self, i: Self::Input<'_>, v: &u64) {
        self.sum += *v;
        log(format!("Acc.Twice {v} a={}", i.a));
    }
}
impl UpdateHandler<acc_edges::A> for Acc {
    const ACTIVE: bool = false;
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(acc: Acc)]
pub struct ReadAcc;
impl Component for ReadAcc {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (ReadAcc, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("ReadAcc {}", i.acc));
        i.acc
    }
}

#[test]
fn handle_component_with_a_handler() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Tick, (ReadAcc,)>::build(Configs::new()).unwrap();
    take();
    p.push(Tick::A(2));
    assert_eq!(take(), ["update Twice", "Acc.Twice 4 a=2", "Acc.update a=2", "ReadAcc 4"]);
    p.push(Tick::B(3));
    assert_eq!(take(), ["update Twice", "Acc.Twice 6 a=2", "Acc.update a=2", "ReadAcc 10"]);
}
