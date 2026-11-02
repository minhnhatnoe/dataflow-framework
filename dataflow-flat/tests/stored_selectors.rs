//! Stored selected values: a named selected input that is read in a step where
//! its selectors do not match gives the last value it matched, or the selector's
//! initial value (`Default::default()` for `#[derive(Demux)]`, `Some`, `Ok`,
//! `Err`) before the first match. `new` sees the initial value. A slot exists
//! only for an input that can be read unmatched, one per (source, chain) pair.
//! No `#![feature]` here.

use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, Configs, Demux, Initial, Pipeline, Root, Select, UpdateHandler};

static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SERIAL: Mutex<()> = Mutex::new(());
fn take() -> Vec<String> {
    std::mem::take(&mut *LOG.lock().unwrap())
}
fn log(s: impl Into<String>) {
    LOG.lock().unwrap().push(s.into());
}
fn lock() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

static DEFAULTS: AtomicUsize = AtomicUsize::new(0);
static CLONES: AtomicUsize = AtomicUsize::new(0);
static NEWS: AtomicUsize = AtomicUsize::new(0);
static DROPS: AtomicUsize = AtomicUsize::new(0);

/// A payload that counts its defaults and clones. Its default is -1, so it
/// can be told from a value.
#[derive(Debug, PartialEq)]
pub struct Book {
    bid: i64,
}
impl Default for Book {
    fn default() -> Self {
        DEFAULTS.fetch_add(1, SeqCst);
        Book { bid: -1 }
    }
}
impl Clone for Book {
    fn clone(&self) -> Self {
        CLONES.fetch_add(1, SeqCst);
        Book { bid: self.bid }
    }
}

#[derive(Root, Demux)]
pub enum Wire {
    Btc(Book),
    Eth(Book),
    Hb,
}
use wire::{Btc, Eth, Hb};

// ---------------------------------------------------------------------------
// The example: two variants of one source, read together
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription)]
#[subscribe(Btc(a): Wire, Eth(b): Wire)]
pub struct Spread;
impl Component for Spread {
    type Config = ();
    type Output = i64;
    fn new((): (), i: Self::Init<'_>) -> (Self, i64) {
        NEWS.fetch_add(1, SeqCst);
        log(format!("Spread.new {} {}", i.a.bid, i.b.bid));
        (Spread, i.a.bid - i.b.bid)
    }
    fn update(&mut self, i: Self::Input<'_>) -> i64 {
        log(format!("Spread {} {}", i.a.bid, i.b.bid));
        i.a.bid - i.b.bid
    }
}
impl Drop for Spread {
    fn drop(&mut self) {
        DROPS.fetch_add(1, SeqCst);
    }
}

/// Reads the output of the sink so the tests can see it.
#[derive(ComponentSubscription)]
#[subscribe(s: Spread)]
pub struct SpreadOut;
impl Component for SpreadOut {
    type Config = ();
    type Output = ();
    fn new((): (), i: Self::Init<'_>) -> (Self, ()) {
        log(format!("SpreadOut.new {}", i.s));
        (SpreadOut, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        log(format!("SpreadOut {}", i.s));
    }
}

#[test]
fn both_variants_read_default_first_then_the_last_value() {
    let _g = lock();
    DEFAULTS.store(0, SeqCst);
    NEWS.store(0, SeqCst);
    DROPS.store(0, SeqCst);
    take();
    {
        let mut p = Pipeline::<Wire, (SpreadOut,)>::build(Configs::new()).unwrap();
        // Two slots (Btc and Eth), each built once, before `new`; `new` sees the defaults.
        assert_eq!(DEFAULTS.load(SeqCst), 2);
        assert_eq!(take(), ["Spread.new -1 -1", "SpreadOut.new 0"]);
        // Eth first: Btc is still the default.
        p.push(Wire::Eth(Book { bid: 3 }));
        assert_eq!(take(), ["Spread -1 3", "SpreadOut -4"]);
        // Btc: Eth keeps its last value.
        p.push(Wire::Btc(Book { bid: 10 }));
        assert_eq!(take(), ["Spread 10 3", "SpreadOut 7"]);
        p.push(Wire::Btc(Book { bid: 12 }));
        assert_eq!(take(), ["Spread 12 3", "SpreadOut 9"]);
        // Neither edge fires: nothing runs, and the values stay.
        p.push(Wire::Hb);
        assert_eq!(take(), Vec::<String>::new());
        p.push(Wire::Eth(Book { bid: 5 }));
        assert_eq!(take(), ["Spread 12 5", "SpreadOut 7"]);
        assert_eq!(NEWS.load(SeqCst), 1);
        assert_eq!(DROPS.load(SeqCst), 0);
    }
    assert_eq!(DROPS.load(SeqCst), 1);
}

// ---------------------------------------------------------------------------
// Two readers share one slot per pair; a plain input does not change that
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription)]
#[subscribe(Btc(a): Wire, Hb: Wire)]
pub struct Other;
impl Component for Other {
    type Config = ();
    type Output = i64;
    fn new((): (), i: Self::Init<'_>) -> (Self, i64) {
        (Other, i.a.bid)
    }
    fn update(&mut self, i: Self::Input<'_>) -> i64 {
        log(format!("Other {}", i.a.bid));
        i.a.bid
    }
}

#[derive(ComponentSubscription)]
#[subscribe(s: Spread, o: Other)]
pub struct Both;
impl Component for Both {
    type Config = ();
    type Output = (i64, i64);
    fn new((): (), i: Self::Init<'_>) -> (Self, (i64, i64)) {
        (Both, (*i.s, *i.o))
    }
    fn update(&mut self, i: Self::Input<'_>) -> (i64, i64) {
        log(format!("Both {} {}", i.s, i.o));
        (*i.s, *i.o)
    }
}

#[test]
fn two_readers_share_a_slot() {
    let _g = lock();
    take();
    let mut p = Pipeline::<Wire, (Both,)>::build(Configs::new()).unwrap();
    // Slots: Btc (read by Spread and Other), Eth (Spread), Hb (Other). One default each.
    take();
    CLONES.store(0, SeqCst);
    // A Btc push writes the shared Btc slot once, not once per reader.
    p.push(Wire::Btc(Book { bid: 4 }));
    assert_eq!(CLONES.load(SeqCst), 1);
    assert_eq!(take(), ["Spread 4 -1", "Other 4", "Both 5 4"]);
    // Hb makes Other run (its unit selector matches); it reads the stored Btc value.
    p.push(Wire::Hb);
    assert_eq!(take(), ["Other 4", "Both 5 4"]);
    p.push(Wire::Eth(Book { bid: 1 }));
    assert_eq!(take(), ["Spread 4 1", "Both 3 4"]);
}

// ---------------------------------------------------------------------------
// A mixed list: plain inputs, a stored input, and a handled entry
// ---------------------------------------------------------------------------

/// Runs on every push.
#[derive(ComponentSubscription)]
#[subscribe(w: Wire)]
pub struct Clock;
impl Component for Clock {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) {
        (Clock, 0)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u32 {
        log("Clock");
        1
    }
}

#[derive(ComponentSubscription)]
#[subscribe(Btc(a): Wire, Eth(_): Wire, c: Clock)]
pub struct Mixed {
    handled: u32,
}
impl Component for Mixed {
    type Config = ();
    type Output = (i64, u32);
    fn new((): (), i: Self::Init<'_>) -> (Self, (i64, u32)) {
        // The handled entry is not an input; the named, stored one has its default.
        log(format!("Mixed.new {} {}", i.a.bid, i.c));
        (Mixed { handled: 0 }, (i.a.bid, 0))
    }
    fn update(&mut self, i: Self::Input<'_>) -> (i64, u32) {
        log(format!("Mixed {} {}", i.a.bid, self.handled));
        (i.a.bid, self.handled)
    }
}
impl UpdateHandler<mixed_edges::Eth> for Mixed {
    fn on_update(&mut self, v: &Book) {
        log(format!("Mixed.eth {}", v.bid));
        self.handled += 1;
    }
}

#[test]
fn a_mixed_list_with_a_handled_entry() {
    let _g = lock();
    take();
    let mut p = Pipeline::<Wire, (Mixed,)>::build(Configs::new()).unwrap();
    assert_eq!(take(), ["Mixed.new -1 0"]);
    // Mixed runs on every push (Clock always runs); `a` is the default until a Btc.
    p.push(Wire::Eth(Book { bid: 2 }));
    assert_eq!(take(), ["Clock", "Mixed.eth 2", "Mixed -1 1"]);
    p.push(Wire::Btc(Book { bid: 8 }));
    assert_eq!(take(), ["Clock", "Mixed 8 1"]);
    p.push(Wire::Hb);
    assert_eq!(take(), ["Clock", "Mixed 8 1"]);
}

// ---------------------------------------------------------------------------
// The source is a component; nested `Some(Bid(a))`
// ---------------------------------------------------------------------------

#[derive(Clone, Demux)]
pub enum Tape {
    Bid(i64),
    Ask(i64),
}
use tape::{Ask, Bid};

#[derive(ComponentSubscription)]
#[subscribe(w: Wire)]
pub struct Feed;
impl Component for Feed {
    type Config = ();
    type Output = Option<Tape>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Option<Tape>) {
        (Feed, None)
    }
    fn update(&mut self, i: Self::Input<'_>) -> Option<Tape> {
        log("Feed");
        match i.w {
            Wire::Btc(b) => Some(Tape::Bid(b.bid)),
            Wire::Eth(b) => Some(Tape::Ask(b.bid)),
            Wire::Hb => None,
        }
    }
}

#[derive(ComponentSubscription)]
#[subscribe(Some(Bid(a)): Feed, Some(Ask(b)): Feed)]
pub struct Quote;
impl Component for Quote {
    type Config = ();
    type Output = i64;
    fn new((): (), i: Self::Init<'_>) -> (Self, i64) {
        log(format!("Quote.new {} {}", *i.a, *i.b));
        (Quote, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> i64 {
        log(format!("Quote {} {}", i.a, i.b));
        i.b - i.a
    }
}

#[test]
fn nested_some_with_a_component_source() {
    let _g = lock();
    take();
    let mut p = Pipeline::<Wire, (Quote,)>::build(Configs::new()).unwrap();
    assert_eq!(take(), ["Quote.new 0 0"]);
    p.push(Wire::Btc(Book { bid: 5 }));
    assert_eq!(take(), ["Feed", "Quote 5 0"]);
    p.push(Wire::Eth(Book { bid: 9 }));
    assert_eq!(take(), ["Feed", "Quote 5 9"]);
    // `None`: neither selector matches, Quote does not run.
    p.push(Wire::Hb);
    assert_eq!(take(), ["Feed"]);
    p.push(Wire::Btc(Book { bid: 6 }));
    assert_eq!(take(), ["Feed", "Quote 6 9"]);
}

// ---------------------------------------------------------------------------
// `Result`
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription)]
#[subscribe(w: Wire)]
pub struct Parse;
impl Component for Parse {
    type Config = ();
    type Output = Result<i64, i64>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Result<i64, i64>) {
        (Parse, Ok(0))
    }
    fn update(&mut self, i: Self::Input<'_>) -> Result<i64, i64> {
        match i.w {
            Wire::Btc(b) => Ok(b.bid),
            Wire::Eth(b) => Err(b.bid),
            Wire::Hb => Ok(0),
        }
    }
}

#[derive(ComponentSubscription)]
#[subscribe(Ok(v): Parse, Err(e): Parse)]
pub struct Fold;
impl Component for Fold {
    type Config = ();
    type Output = (i64, i64);
    fn new((): (), i: Self::Init<'_>) -> (Self, (i64, i64)) {
        log(format!("Fold.new {} {}", *i.v, *i.e));
        (Fold, (0, 0))
    }
    fn update(&mut self, i: Self::Input<'_>) -> (i64, i64) {
        (*i.v, *i.e)
    }
}
#[derive(ComponentSubscription)]
#[subscribe(f: Fold)]
pub struct FoldOut;
impl Component for FoldOut {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (FoldOut, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        log(format!("Fold {:?}", i.f));
    }
}

#[test]
fn result_selectors_compose() {
    let _g = lock();
    take();
    let mut p = Pipeline::<Wire, (FoldOut,)>::build(Configs::new()).unwrap();
    assert_eq!(take(), ["Fold.new 0 0"]);
    p.push(Wire::Eth(Book { bid: 7 }));
    assert_eq!(take(), ["Fold (0, 7)"]);
    p.push(Wire::Btc(Book { bid: 2 }));
    assert_eq!(take(), ["Fold (2, 7)"]);
}

// ---------------------------------------------------------------------------
// A generic selector, and a hand-written one with its own initial value
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription)]
#[subscribe(S(a): Wire, Eth(b): Wire)]
pub struct GenSpread<S: Select<Wire, Out = Book> + Initial<Wire>>(PhantomData<S>);
impl<S: Select<Wire, Out = Book> + Initial<Wire>> Component for GenSpread<S> {
    type Config = ();
    type Output = i64;
    fn new((): (), _: Self::Init<'_>) -> (Self, i64) {
        (GenSpread(PhantomData), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> i64 {
        i.a.bid * 100 + i.b.bid
    }
}

#[derive(ComponentSubscription)]
#[subscribe(s: GenSpread<Btc>)]
pub struct GenOut;
impl Component for GenOut {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (GenOut, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        log(format!("GenOut {}", i.s));
    }
}

#[test]
fn generic_selector() {
    let _g = lock();
    take();
    let mut p = Pipeline::<Wire, (GenOut,)>::build(Configs::new()).unwrap();
    p.push(Wire::Eth(Book { bid: 3 }));
    assert_eq!(take(), ["GenOut -97"]);
    p.push(Wire::Btc(Book { bid: 1 }));
    assert_eq!(take(), ["GenOut 103"]);
}

/// A hand-written selector: matches Btc whose bid is positive. Its initial value is 42.
pub struct Positive;
impl Select<Wire> for Positive {
    type Out = Book;
    const KEY: u32 = dataflow_flat::GUARD;
    fn select(x: &Wire) -> Option<&Book> {
        match x {
            Wire::Btc(b) if b.bid > 0 => Some(b),
            _ => None,
        }
    }
    fn key_of(_: &Wire) -> u32 {
        dataflow_flat::GUARD
    }
}
impl Initial<Wire> for Positive {
    fn initial() -> Book {
        Book { bid: 42 }
    }
}

#[derive(ComponentSubscription)]
#[subscribe(Positive(p): Wire, c: Clock)]
pub struct Guarded;
impl Component for Guarded {
    type Config = ();
    type Output = i64;
    fn new((): (), i: Self::Init<'_>) -> (Self, i64) {
        log(format!("Guarded.new {}", i.p.bid));
        (Guarded, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> i64 {
        log(format!("Guarded {}", i.p.bid));
        i.p.bid
    }
}

#[test]
fn hand_written_selector_with_its_own_initial_value() {
    let _g = lock();
    take();
    let mut p = Pipeline::<Wire, (Guarded,)>::build(Configs::new()).unwrap();
    assert_eq!(take(), ["Guarded.new 42"]);
    p.push(Wire::Hb);
    assert_eq!(take(), ["Clock", "Guarded 42"]);
    p.push(Wire::Btc(Book { bid: -5 })); // does not match `Positive`: the slot keeps its value
    assert_eq!(take(), ["Clock", "Guarded 42"]);
    p.push(Wire::Btc(Book { bid: 6 }));
    assert_eq!(take(), ["Clock", "Guarded 6"]);
    p.push(Wire::Hb);
    assert_eq!(take(), ["Clock", "Guarded 6"]);
}

// ---------------------------------------------------------------------------
// A payload without `Default` or `Clone` in a variant nobody reads unmatched
// ---------------------------------------------------------------------------

pub struct Opaque(u8);

#[derive(Root, Demux)]
pub enum Raw {
    Num(Book),
    Blob(Opaque),
}

/// Reads only `Num`, always covered: no slot, and `Opaque` needs neither `Default` nor `Clone`.
#[derive(ComponentSubscription)]
#[subscribe(raw::Num(n): Raw)]
pub struct Covered;
impl Component for Covered {
    type Config = ();
    type Output = i64;
    fn new((): (), i: Self::Init<'_>) -> (Self, i64) {
        // No slot, but Book is Default, so new gets the default.
        log(format!("Covered.new {}", i.n.get().is_none())); // Book is Default
        (Covered, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> i64 {
        i.n.bid
    }
}

/// Reads `Blob` as a handled entry: it needs no storage either.
#[derive(ComponentSubscription)]
#[subscribe(raw::Num(n): Raw, raw::Blob(_): Raw)]
pub struct Watch {
    blobs: u32,
}
impl Component for Watch {
    type Config = ();
    type Output = (i64, u32);
    fn new((): (), i: Self::Init<'_>) -> (Self, (i64, u32)) {
        (Watch { blobs: 0 }, (i.n.bid, 0))
    }
    fn update(&mut self, i: Self::Input<'_>) -> (i64, u32) {
        (i.n.bid, self.blobs)
    }
}
impl UpdateHandler<watch_edges::Blob> for Watch {
    fn on_update(&mut self, v: &Opaque) {
        self.blobs += v.0 as u32;
    }
}

#[test]
fn a_payload_without_default_compiles_when_not_read_unmatched() {
    let _g = lock();
    take();
    let mut p = Pipeline::<Raw, (Covered,)>::build(Configs::new()).unwrap();
    assert_eq!(take(), ["Covered.new false"]);
    p.push(Raw::Num(Book { bid: 1 }));
    p.push(Raw::Blob(Opaque(1)));
    // Watch reads Num (stored: it also runs on Blob) and handles Blob.
    let mut q = Pipeline::<Raw, (Watch,)>::build(Configs::new()).unwrap();
    q.push(Raw::Blob(Opaque(2)));
    q.push(Raw::Num(Book { bid: 4 }));
}
