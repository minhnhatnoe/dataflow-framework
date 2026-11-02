use std::marker::PhantomData;
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

// `on_update` gets the whole `Input`: a plain entry under a selector upstream (`eth`) and a
// selected entry (`sol`) whose match did not fire in the step both hold valid values.

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

#[test]
fn on_update_sees_the_whole_input() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    take();
    let mut p = Pipeline::<Msg, (Mix,)>::build(Configs::new()).unwrap();
    take();
    let mix = |l: Vec<String>| l.into_iter().filter(|s| s.starts_with("Mix")).collect::<Vec<_>>();
    p.push(Msg { key: Some(2), v: 3 }); // Sol matches: sol = 3, eth is Quoter<Eth>'s initial output
    assert_eq!(mix(take()), ["Mix.update eth=1001 sol=3"]);
    p.push(Msg { key: Some(1), v: 5 }); // Eth: Quoter<Eth> = 10
    assert_eq!(mix(take()), ["Mix.update eth=10 sol=3"]);
    p.push(Msg { key: Some(0), v: 7 }); // Btc fires the handled edge; eth and sol are from earlier steps
    assert_eq!(mix(take()), ["Mix.on_update btc=14 eth=10 sol=3", "Mix.update eth=10 sol=3"]);
}

