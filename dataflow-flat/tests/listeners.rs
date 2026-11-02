//! Listener requests as types: a `#[listener]` is told, before its `new`, the
//! selectors its subscribers read (`Request<S>::add`, once per distinct
//! selector, in the order of the first reader: components in execution order,
//! subscriptions in listing order, chains left to right). Exact lists, no
//! duplicates, nothing for unread selectors, readers through `Some`, generic
//! readers, a marker shared by two enums, two listeners on different enums,
//! generated (non-marker) selectors. No `#![feature]` here.

use std::marker::PhantomData;
use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, Configs, Demux, Listener, Pipeline, Request, Root, Select};

static SERIAL: Mutex<()> = Mutex::new(());
/// (listener, what its `new` got in `init.requests`).
static GOT: Mutex<Vec<(&'static str, Vec<&'static str>)>> = Mutex::new(Vec::new());

fn got() -> Vec<(&'static str, Vec<&'static str>)> {
    std::mem::take(&mut *GOT.lock().unwrap())
}

pub mod markets {
    pub struct Btc;
    pub struct Eth;
    pub struct Sol;
    pub struct Doge;
}
use markets::{Btc, Doge, Eth, Sol};

/// What a selector is called in the request lists.
pub trait Name {
    const N: &'static str;
}
macro_rules! names {
    ($($t:ident),*) => {$(impl Name for markets::$t { const N: &'static str = stringify!($t); })*};
}
names!(Btc, Eth, Sol, Doge);

/// The requests: the names, in `add` order.
#[derive(Default)]
pub struct Names(Vec<&'static str>);
impl<S: Name> Request<S> for Names {
    fn add(&mut self) {
        self.0.push(S::N);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Demux)]
#[demux(markers = markets)]
pub enum BinFrame {
    Btc(u32),
    Eth(u32),
    Sol(u32),
    Doge(u32),
}
#[derive(Clone, Copy, Debug, PartialEq, Demux)]
#[demux(markers = markets)]
pub enum CbFrame {
    Btc(u32),
    Eth(u32),
    Sol(u32),
    Doge(u32),
}

#[derive(Root)]
pub struct Wire {
    b: Option<BinFrame>,
    c: Option<CbFrame>,
    d: BinFrame,
}

macro_rules! listener {
    ($name:ident, $out:ty, $field:ident) => {
        #[derive(ComponentSubscription)]
        #[subscribe(w: Wire)]
        #[listener]
        pub struct $name;
        impl Listener for $name {
            type Requests = Names;
        }
        impl Component for $name {
            type Config = ();
            type Output = $out;
            fn new((): (), init: Self::Init<'_>) -> (Self, $out) {
                GOT.lock().unwrap().push((stringify!($name), init.requests.0));
                ($name, Default::default())
            }
            fn update(&mut self, i: Self::Input<'_>) -> $out {
                i.w.$field
            }
        }
    };
}
// `Option<BinFrame>` etc. need `Default`: a frame is never constructed by `new`.
listener!(LB, Option<BinFrame>, b);
listener!(LC, Option<CbFrame>, c);

macro_rules! reader {
    ($name:ident, [$($sub:tt)*]) => {
        #[derive(ComponentSubscription)]
        #[subscribe($($sub)*)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = ();
            fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
                ($name, ())
            }
            fn update(&mut self, _: Self::Input<'_>) {}
        }
    };
}

// ---------------------------------------------------------------------------
// Exact list, order, no duplicates, nothing unread, readers through `Some`
// ---------------------------------------------------------------------------

mod basic {
    use super::*;
    reader!(REth, [Some(Eth(t)): LB]);
    reader!(RBtc, [Some(Btc(t)): LB]);
    reader!(RBtc2, [Some(BinFrame::Btc(t)): LB]); // the same selector again, spelled via the enum
    reader!(RSol, [Some(Sol(t)): LB]);
}

#[test]
fn exact_list_in_first_reader_order_without_duplicates_or_unread() {
    let _g = SERIAL.lock().unwrap();
    got();
    // Execution order: LB, REth, RBtc, RBtc2, RSol (sinks in listing order).
    let _p = Pipeline::<Wire, (basic::REth, basic::RBtc, basic::RBtc2, basic::RSol)>::build(Configs::new()).unwrap();
    // Eth (REth), Btc (RBtc), Sol (RSol); Btc again in RBtc2 is skipped; Doge is never read.
    assert_eq!(got(), [("LB", vec!["Eth", "Btc", "Sol"])]);
}

#[test]
fn the_order_follows_the_execution_order_of_the_readers() {
    let _g = SERIAL.lock().unwrap();
    got();
    let _p = Pipeline::<Wire, (basic::RSol, basic::RBtc, basic::REth)>::build(Configs::new()).unwrap();
    assert_eq!(got(), [("LB", vec!["Sol", "Btc", "Eth"])]);
}

#[test]
fn a_listener_nobody_reads_gets_an_empty_default() {
    let _g = SERIAL.lock().unwrap();
    got();
    reader!(JustAsks, [w: LB]);
    let _p = Pipeline::<Wire, (JustAsks,)>::build(Configs::new()).unwrap();
    assert_eq!(got(), [("LB", vec![])]);
}

// ---------------------------------------------------------------------------
// A listener whose output is the enum itself (no `Some`)
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription)]
#[subscribe(w: Wire)]
#[listener]
pub struct LD;
impl Listener for LD {
    type Requests = Names;
}
impl Component for LD {
    type Config = ();
    type Output = BinFrame;
    fn new((): (), init: Self::Init<'_>) -> (Self, BinFrame) {
        GOT.lock().unwrap().push(("LD", init.requests.0));
        (LD, BinFrame::Doge(0))
    }
    fn update(&mut self, i: Self::Input<'_>) -> BinFrame {
        i.w.d
    }
}
reader!(DirectEth, [Eth(t): LD]);
reader!(DirectSol, [BinFrame::Sol(t): LD]);

#[test]
fn a_direct_enum_output_is_read_without_some() {
    let _g = SERIAL.lock().unwrap();
    got();
    let _p = Pipeline::<Wire, (DirectSol, DirectEth)>::build(Configs::new()).unwrap();
    assert_eq!(got(), [("LD", vec!["Sol", "Eth"])]);
}

// ---------------------------------------------------------------------------
// Generic readers
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription)]
#[subscribe(Some(S(t)): LB)]
pub struct Gen<S: Select<BinFrame, Out = u32>>(PhantomData<S>);
impl<S: Select<BinFrame, Out = u32>> Component for Gen<S> {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Gen(PhantomData), ())
    }
    fn update(&mut self, _: Self::Input<'_>) {}
}

#[test]
fn generic_readers_request_their_concrete_selector() {
    let _g = SERIAL.lock().unwrap();
    got();
    let _p = Pipeline::<Wire, (Gen<Sol>, Gen<Btc>, basic::RBtc)>::build(Configs::new()).unwrap();
    assert_eq!(got(), [("LB", vec!["Sol", "Btc"])]);
}

// ---------------------------------------------------------------------------
// One marker on two enums; two listeners on different enums
// ---------------------------------------------------------------------------

mod two {
    use super::*;
    reader!(BBtc, [Some(Btc(t)): LB]);
    reader!(CBtc, [Some(Btc(t)): LC]); // the same marker, the other enum
    reader!(CEth, [Some(CbFrame::Eth(t)): LC]);
}

#[test]
fn two_listeners_on_different_enums_share_markers() {
    let _g = SERIAL.lock().unwrap();
    got();
    // Order: LB, BBtc, LC, CBtc, CEth.
    let _p = Pipeline::<Wire, (two::BBtc, two::CBtc, two::CEth)>::build(Configs::new()).unwrap();
    let mut g = got();
    g.sort();
    // Each listener gets its own list; `Btc` is requested of both (one marker, two enums).
    assert_eq!(g, [("LB", vec!["Btc"]), ("LC", vec!["Btc", "Eth"])]);
}

// ---------------------------------------------------------------------------
// Generated selectors (no `markers`): `Request<frame_kind::A>`
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Demux)]
pub enum FrameKind {
    A(u32),
    B(u32),
}
#[derive(Default)]
pub struct Kinds(Vec<&'static str>);
impl Request<frame_kind::A> for Kinds {
    fn add(&mut self) {
        self.0.push("A");
    }
}
impl Request<frame_kind::B> for Kinds {
    fn add(&mut self) {
        self.0.push("B");
    }
}
#[derive(ComponentSubscription)]
#[subscribe(w: Wire)]
#[listener]
pub struct LK;
impl Listener for LK {
    type Requests = Kinds;
}
impl Component for LK {
    type Config = ();
    type Output = FrameKind;
    fn new((): (), init: Self::Init<'_>) -> (Self, FrameKind) {
        GOT.lock().unwrap().push(("LK", init.requests.0));
        (LK, FrameKind::A(0))
    }
    fn update(&mut self, _: Self::Input<'_>) -> FrameKind {
        FrameKind::B(0)
    }
}
reader!(KB, [FrameKind::B(t): LK]);

#[test]
fn generated_selectors_are_requests_too() {
    let _g = SERIAL.lock().unwrap();
    got();
    let _p = Pipeline::<Wire, (KB,)>::build(Configs::new()).unwrap();
    assert_eq!(got(), [("LK", vec!["B"])]);
}

// ---------------------------------------------------------------------------
// The first pass constructs nothing downstream: `add` calls happen before any `new`
// ---------------------------------------------------------------------------

static EVENTS: Mutex<Vec<String>> = Mutex::new(Vec::new());

#[derive(Default)]
pub struct Logged;
impl<S: Name> Request<S> for Logged {
    fn add(&mut self) {
        EVENTS.lock().unwrap().push(format!("add {}", S::N));
    }
}
#[derive(ComponentSubscription)]
#[subscribe(w: Wire)]
#[listener]
pub struct LL;
impl Listener for LL {
    type Requests = Logged;
}
impl Component for LL {
    type Config = ();
    type Output = Option<BinFrame>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Option<BinFrame>) {
        EVENTS.lock().unwrap().push("new LL".into());
        (LL, None)
    }
    fn update(&mut self, _: Self::Input<'_>) -> Option<BinFrame> {
        None
    }
}
#[derive(ComponentSubscription)]
#[subscribe(Some(Btc(t)): LL)]
pub struct LateReader;
impl Component for LateReader {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        EVENTS.lock().unwrap().push("new LateReader".into());
        (LateReader, ())
    }
    fn update(&mut self, _: Self::Input<'_>) {}
}

#[test]
fn requests_are_delivered_before_the_listener_and_before_anything_is_constructed() {
    let _g = SERIAL.lock().unwrap();
    EVENTS.lock().unwrap().clear();
    let _p = Pipeline::<Wire, (LateReader,)>::build(Configs::new()).unwrap();
    assert_eq!(*EVENTS.lock().unwrap(), ["add Btc", "new LL", "new LateReader"]);
}
