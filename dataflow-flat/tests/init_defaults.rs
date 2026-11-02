//! `new` is given the selector's default on every selected edge whose payload
//! has one, whether or not the edge has a slot. A payload without a default
//! compiles and builds unless `new` reads it. No `#![feature]` here.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, Configs, Demux, Initial, Pipeline, Root, Select};

static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SERIAL: Mutex<()> = Mutex::new(());
fn take() -> Vec<String> {
    std::mem::take(&mut *LOG.lock().unwrap_or_else(|e| e.into_inner()))
}
fn log(s: impl Into<String>) {
    LOG.lock().unwrap_or_else(|e| e.into_inner()).push(s.into());
}
fn lock() -> std::sync::MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(|e| e.into_inner())
}

#[derive(Debug, PartialEq)]
pub struct Book {
    bid: i64,
}
impl Default for Book {
    fn default() -> Self {
        Book { bid: -1 }
    }
}
/// No `Default`, no `Clone`.
pub struct Opaque(u8);

#[derive(Root, Demux)]
pub enum Wire {
    Btc(Book),
    Eth(Book),
    Blob(Opaque),
}
use wire::{Blob, Btc};

macro_rules! reader {
    ($name:ident, [$($sub:tt)*], |$i:ident| $new:expr, |$u:ident| $upd:expr) => {
        #[derive(ComponentSubscription)]
        #[subscribe($($sub)*)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = ();
            fn new((): (), $i: Self::Init<'_>) -> (Self, ()) {
                log($new);
                ($name, ())
            }
            fn update(&mut self, $u: Self::Input<'_>) {
                log($upd);
            }
        }
    };
}

// A plain selected edge: always covered, so no slot.
reader!(Plain, [Btc(a): Wire], |i| format!("Plain.new {}", i.a.bid), |u| format!("Plain {}", u.a.bid));

#[test]
fn plain_selected_edge_reads_default_in_new() {
    let _g = lock();
    take();
    let mut p = Pipeline::<Wire, (Plain,)>::build(Configs::new()).unwrap();
    assert_eq!(take(), ["Plain.new -1"]);
    p.push(Wire::Btc(Book { bid: 3 }));
    assert_eq!(take(), ["Plain 3"]);
}

// Nested chain: Some(Btc(a)) over an Option; the last selector's value.
#[derive(ComponentSubscription)]
#[subscribe(w: Wire)]
pub struct Wrap;
impl Component for Wrap {
    type Config = ();
    type Output = Option<Wire>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Option<Wire>) {
        (Wrap, None)
    }
    fn update(&mut self, _: Self::Input<'_>) -> Option<Wire> {
        None
    }
}
reader!(Nested, [Some(Btc(a)): Wrap], |i| format!("Nested.new {}", i.a.bid), |u| format!("Nested {}", u.a.bid));
// `Wire` is not `Default`: `Some(w)` has no default; unread in `new`, it builds.
reader!(JustSome, [Some(w): Wrap], |i| format!("JustSome.new {}", i.w.get().is_none()), |_u| "JustSome");

#[test]
fn nested_chain_reads_the_last_selectors_default() {
    let _g = lock();
    take();
    let _p = Pipeline::<Wire, (Nested,)>::build(Configs::new()).unwrap();
    assert_eq!(take(), ["Nested.new -1"]);
    let _q = Pipeline::<Wire, (JustSome,)>::build(Configs::new()).unwrap();
    assert_eq!(take(), ["JustSome.new true"]);
}

// Result
#[derive(ComponentSubscription)]
#[subscribe(w: Wire)]
pub struct Parse;
impl Component for Parse {
    type Config = ();
    type Output = Result<i64, String>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Self::Output) {
        (Parse, Ok(0))
    }
    fn update(&mut self, _: Self::Input<'_>) -> Self::Output {
        Ok(1)
    }
}
reader!(OnOk, [Ok(v): Parse], |i| format!("OnOk.new {}", *i.v), |u| format!("OnOk {}", u.v));
reader!(OnErr, [Err(e): Parse], |i| format!("OnErr.new {:?}", *i.e), |_u| "OnErr");

#[test]
fn result_reads_default_in_new() {
    let _g = lock();
    take();
    let _p = Pipeline::<Wire, (OnOk,)>::build(Configs::new()).unwrap();
    assert_eq!(take(), ["OnOk.new 0"]);
    let _q = Pipeline::<Wire, (OnErr,)>::build(Configs::new()).unwrap();
    assert_eq!(take(), ["OnErr.new \"\""]);
}

// Hand-written selector with an `Initial` impl on a plain edge (payload not `Default`).
pub struct Positive;
impl Select<Wire> for Positive {
    type Out = Opaque;
    const KEY: u32 = dataflow_flat::GUARD;
    fn select(x: &Wire) -> Option<&Opaque> {
        match x {
            Wire::Blob(b) if b.0 > 0 => Some(b),
            _ => None,
        }
    }
    fn key_of(_: &Wire) -> u32 {
        dataflow_flat::GUARD
    }
}
impl Initial<Wire> for Positive {
    fn initial() -> Opaque {
        Opaque(42)
    }
}
reader!(Hand, [Positive(p): Wire], |i| format!("Hand.new {}", i.p.0), |u| format!("Hand {}", u.p.0));

#[test]
fn hand_written_selector_initial_on_a_plain_edge() {
    let _g = lock();
    take();
    let mut p = Pipeline::<Wire, (Hand,)>::build(Configs::new()).unwrap();
    assert_eq!(take(), ["Hand.new 42"]);
    p.push(Wire::Blob(Opaque(5)));
    assert_eq!(take(), ["Hand 5"]);
}

// A non-Default payload: unread in `new` it builds; read, it panics clearly.
reader!(Unread, [Blob(b): Wire], |i| format!("Unread.new {}", i.b.get().is_none()), |u| format!("Unread {}", u.b.0));
reader!(Read, [Blob(b): Wire], |i| format!("Read.new {}", i.b.0), |u| format!("Read {}", u.b.0));

#[test]
fn non_default_payload_unread_builds() {
    let _g = lock();
    take();
    let mut p = Pipeline::<Wire, (Unread,)>::build(Configs::new()).unwrap();
    assert_eq!(take(), ["Unread.new true"]);
    p.push(Wire::Blob(Opaque(7)));
    assert_eq!(take(), ["Unread 7"]);
}

#[test]
fn non_default_payload_read_panics_with_a_clear_message() {
    let _g = lock();
    take();
    let r = catch_unwind(AssertUnwindSafe(|| {
        let _ = Pipeline::<Wire, (Read,)>::build(Configs::new());
    }));
    let e = r.expect_err("reading a payload without a default in new must panic");
    let msg = e.downcast_ref::<String>().cloned().unwrap_or_default();
    assert!(msg.contains("`Read::new` read the input `b`"), "{msg}");
    assert!(msg.contains("Opaque"), "{msg}");
    assert!(msg.contains("implement `Default`"), "{msg}");
}
