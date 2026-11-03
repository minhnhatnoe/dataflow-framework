//! One `Component` trait for by-value components, handle components and listeners. No markers:
//! `#[derive(ComponentSubscription, UpdateHandler)]` alone, and the impl says what the component is.
//! The six spellings, plus a handle component that is also a listener with selectors on its view.
//! No `#![feature]` here.

use std::sync::Mutex;
use markets::Btc;

use dataflow_flat::{
    Component, ComponentSubscription, Configs, Demux, Node, Pipeline, Request, Root, Sinks, UpdateHandler,
};

static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SERIAL: Mutex<()> = Mutex::new(());
fn log(s: impl Into<String>) {
    LOG.lock().unwrap().push(s.into());
}
fn take() -> Vec<String> {
    std::mem::take(&mut *LOG.lock().unwrap())
}

pub mod markets {
    pub struct Btc;
    pub struct Eth;
}
#[derive(Clone, Copy, Debug, PartialEq, Demux)]
#[demux(markers = markets)]
pub enum Frame {
    Btc(u64),
    Eth(u64),
}
#[derive(Root)]
pub struct Msg {
    pub v: u64,
    pub frame: Option<Frame>,
}

// 1. By value: exactly the old spelling, nothing new.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct Sum {
    n: u64,
}
impl Component for Sum {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Sum { n: 0 }, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        self.n += i.m.v;
        self.n
    }
}

// 2. A handle with a reference view.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct Ring {
    arr: [u64; 4],
    n: usize,
}
impl Component for Ring {
    type Config = ();
    type Handle = usize; // the slot of the last write
    type View<'a> = &'a u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, usize) {
        (Ring { arr: [0; 4], n: 0 }, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> usize {
        self.n += 1;
        self.arr[self.n % 4] = i.m.v;
        self.n % 4
    }
    fn view<'a>(&'a self, h: &'a usize) -> &'a u64 {
        &self.arr[*h]
    }
}

// 3. A handle with an owned view computed from the state and a non-trivial handle.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct Tally {
    total: u64,
}
impl Component for Tally {
    type Config = ();
    type Handle = u64; // the last increment
    type View<'a> = (u64, u64);
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Tally { total: 0 }, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        self.total += i.m.v;
        i.m.v
    }
    fn view<'a>(&'a self, h: &'a u64) -> (u64, u64) {
        (self.total, *h)
    }
}

// 4. `Handle = ()`: the whole state is the view.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
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
        self.v += i.m.v;
    }
    fn view<'a>(&'a self, (): &'a ()) -> u64 {
        self.v
    }
}

// 5. A listener: `type Requests`, no `#[listener]`.
pub trait Name {
    const N: &'static str;
}
impl Name for markets::Btc {
    const N: &'static str = "Btc";
}
impl Name for markets::Eth {
    const N: &'static str = "Eth";
}
#[derive(Default)]
pub struct Names(Vec<&'static str>);
impl<S: Name> Request<S> for Names {
    fn add(&mut self) {
        self.0.push(S::N);
    }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct Ws;
impl Component for Ws {
    type Config = ();
    type Requests = Names;
    type Output = Option<Frame>;
    fn new((): (), init: Self::Init<'_>) -> (Self, Option<Frame>) {
        log(format!("Ws asked for {:?}", init.requests.0));
        (Ws, None)
    }
    fn update(&mut self, i: Self::Input<'_>) -> Option<Frame> {
        i.m.frame
    }
}

// Readers of all of them; one is generic code that only knows `X: Node<Output = u64>`.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(sum: Sum, ring: Ring, tally: Tally, level: Level, Some(Btc(b)): Ws)]
pub struct Reader;
impl Component for Reader {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Reader, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let ring: &u64 = i.ring; // a reference view, no `&` added
        let (total, last): (u64, u64) = i.tally; // an owned view
        let level: u64 = i.level;
        log(format!("sum={} ring={} tally={}/{} level={} btc={}", i.sum, ring, total, last, level, i.b));
        *i.sum + *ring + total + last + level + *i.b
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(s: X)]
pub struct Generic<X: Node<Output = u64>>(std::marker::PhantomData<X>);
impl<X: Node<Output = u64>> Component for Generic<X> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Generic(Default::default()), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let v: &u64 = i.s; // `&T` for `X: Node<Output = T>`
        *v + 1
    }
}

// A passthrough whose output is a type parameter's output (the kind is a type, so generic code needs no const).
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(s: X)]
pub struct Pass<X: Node>(std::marker::PhantomData<X>)
where
    X::Output: Clone + Default;
impl<X: Node> Component for Pass<X>
where
    X::Output: Clone + Default,
{
    type Config = ();
    type Output = X::Output;
    fn new((): (), _: Self::Init<'_>) -> (Self, X::Output) {
        (Pass(Default::default()), Default::default())
    }
    fn update(&mut self, i: Self::Input<'_>) -> X::Output {
        i.s.clone()
    }
}

// 6. A by-value handler (handled entry), unchanged.
#[derive(ComponentSubscription)]
#[subscribe(Sum, m: Msg)]
pub struct Log;
impl Component for Log {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Log, ())
    }
    fn update(&mut self, _: Self::Input<'_>) {}
}
impl UpdateHandler<Msg> for Log {
    const ACTIVE: bool = false;
}
impl UpdateHandler<Sum> for Log {
    fn on_update(&mut self, _: Self::Input<'_>, v: &u64) {
        log(format!("on_update Sum {v}"));
    }
}

// 7. A handle component that is also a listener: `Requests` is independent of `Handle`. A
// subscriber selects from its view (`Btc(b): HWs`), and the selector is requested.
#[derive(Demux)]
#[demux(markers = markets)]
pub enum HView<'a> {
    Btc(&'a u64),
    Eth(&'a u64),
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct HWs {
    last: Frame,
}
impl Component for HWs {
    type Config = ();
    type Handle = ();
    type View<'a> = HView<'a>;
    type Requests = Names;
    fn new((): (), init: Self::Init<'_>) -> (Self, ()) {
        log(format!("HWs asked for {:?}", init.requests.0));
        (HWs { last: Frame::Eth(0) }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        if let Some(f) = i.m.frame {
            self.last = f;
        }
    }
    fn view<'a>(&'a self, (): &'a ()) -> HView<'a> {
        match &self.last {
            Frame::Btc(x) => HView::Btc(x),
            Frame::Eth(x) => HView::Eth(x),
        }
    }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Btc(b): HWs)]
pub struct HReader;
impl Component for HReader {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (HReader, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let b: &u64 = i.b;
        log(format!("HReader btc={b}"));
        *b
    }
}

fn run<S: Sinks<Msg>>() -> Vec<String> {
    let mut p = Pipeline::<Msg, S>::build(Configs::new()).unwrap();
    p.push(Msg { v: 2, frame: Some(Frame::Btc(10)) });
    p.push(Msg { v: 3, frame: Some(Frame::Eth(20)) });
    take()
}

#[test]
fn all_six_spellings() {
    let _g = SERIAL.lock().unwrap();
    take();
    let out = run::<(Reader, Generic<Sum>, Pass<Sum>, Log)>();
    assert_eq!(out[0], "Ws asked for [\"Btc\"]"); // only the selector somebody reads
    assert_eq!(out[1], "sum=2 ring=2 tally=2/2 level=9 btc=10");
    assert_eq!(out[2], "on_update Sum 2"); // the handler `Log` runs after `Reader` (listing order)
    // the second push: an Eth frame, so the last matched Btc value (10) is read
    assert_eq!(out[3], "sum=5 ring=3 tally=5/3 level=12 btc=10");
    assert_eq!(out[4], "on_update Sum 5");
}

#[test]
fn a_handle_listener_is_asked_through_selectors_on_its_view() {
    let _g = SERIAL.lock().unwrap();
    take();
    let out = run::<(HReader,)>();
    assert_eq!(out[0], "HWs asked for [\"Btc\"]");
    // runs on the first push only (a Btc frame); the Eth frame does not match
    assert_eq!(out[1], "HReader btc=10");
    assert_eq!(out.len(), 2);
}
