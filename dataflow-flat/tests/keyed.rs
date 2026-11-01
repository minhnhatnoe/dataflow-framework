//! Keyed selectors (`On<S>`) and listener requests: exclusivity of different
//! symbols, the keys a `#[listener]` is asked for (deduplicated, sorted, only
//! the symbols somebody reads), and that `subscribe` runs once, after
//! construction and before the first push. No `#![feature]` here.

use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, Configs, Keyed, Listener, On, Pipeline, Root, Symbol};

static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
// Tests in one binary run in parallel; each holds this and clears the log.
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
symbol!(Doge, 9); // nobody reads it

/// A message of a multiplexed feed.
#[derive(Root)]
pub struct Msg {
    pub key: Option<u32>,
    pub v: u64,
}

/// What the listeners publish: the message again, keyed.
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

/// A listener whose output is an `Option<Frame>`, read through `Some, On<_>`.
#[derive(ComponentSubscription)]
#[subscribe(m: Msg)]
#[listener]
pub struct Ws;
impl Listener for Ws {
    fn subscribe(&mut self, keys: &[u32]) {
        log(format!("Ws.subscribe{keys:?}"));
    }
}
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

/// A listener read with `On<_>` directly (output is the `Keyed` value).
#[derive(ComponentSubscription)]
#[subscribe(m: Msg)]
#[listener]
pub struct Ws2;
impl Listener for Ws2 {
    fn subscribe(&mut self, keys: &[u32]) {
        log(format!("Ws2.subscribe{keys:?}"));
    }
}
impl Component for Ws2 {
    type Config = ();
    type Output = Frame;
    fn new((): (), _: Self::Init<'_>) -> (Self, Frame) {
        log("Ws2.new");
        (Ws2, Frame { key: None, v: 0 })
    }
    fn update(&mut self, i: Self::Input<'_>) -> Frame {
        Frame { key: i.m.key, v: i.m.v }
    }
}

/// A listener nobody reads through `On`.
#[derive(ComponentSubscription)]
#[subscribe(m: Msg)]
#[listener]
pub struct Ws3;
impl Listener for Ws3 {
    fn subscribe(&mut self, keys: &[u32]) {
        log(format!("Ws3.subscribe{keys:?}"));
    }
}
impl Component for Ws3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Ws3, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        i.m.v
    }
}

#[derive(ComponentSubscription)]
#[subscribe(u: Ws => Some, On<S>)]
pub struct Book<S: Symbol>(std::marker::PhantomData<S>);
impl<S: Symbol> Component for Book<S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Book(std::marker::PhantomData), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("Book<{}>", S::NAME));
        *i.u
    }
}

/// Reads the same `On<S>` as `Book<S>`, and the book itself.
#[derive(ComponentSubscription)]
#[subscribe(u: Ws => Some, On<S>, b: Book<S>)]
pub struct Vwap<S: Symbol>(std::marker::PhantomData<S>);
impl<S: Symbol> Component for Vwap<S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Vwap(std::marker::PhantomData), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("Vwap<{}>", S::NAME));
        *i.u + *i.b
    }
}

/// An exclusive join of two symbols.
#[derive(ComponentSubscription)]
#[subscribe(b: Book<Btc>, e: Book<Eth>)]
pub struct Join;
impl Component for Join {
    type Config = ();
    type Output = (u64, u64);
    fn new((): (), _: Self::Init<'_>) -> (Self, (u64, u64)) {
        (Join, (0, 0))
    }
    fn update(&mut self, i: Self::Input<'_>) -> (u64, u64) {
        log("Join");
        (*i.b, *i.e)
    }
}

#[derive(ComponentSubscription)]
#[subscribe(f: Ws2 => On<Sol>)]
pub struct Direct;
impl Component for Direct {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Direct, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log("Direct");
        *i.f
    }
}

type P = Pipeline<Msg, (Join, Vwap<Btc>, Direct, Ws3)>;

#[test]
fn requests_are_exactly_the_keys_read_deduplicated() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    take();
    let _p = P::build(Configs::new()).unwrap();
    let l = take();
    // Book<Btc>, Vwap<Btc> and Book<Eth> read Ws: keys 0 and 1, once each, sorted;
    // Doge (9) is read by nobody. Ws2 is read by `On<Sol>` directly. Ws3 by no `On`.
    let subs: Vec<_> = l.iter().filter(|s| s.contains("subscribe")).cloned().collect();
    assert_eq!(subs.len(), 3, "once per listener: {l:?}");
    for want in ["Ws.subscribe[0, 1]", "Ws2.subscribe[2]", "Ws3.subscribe[]"] {
        assert_eq!(subs.iter().filter(|s| *s == want).count(), 1, "{want} in {l:?}");
    }
}

#[test]
fn subscribe_happens_after_construction_and_before_the_first_push() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    take();
    let mut p = P::build(Configs::new()).unwrap();
    let built = take();
    assert!(built.iter().any(|s| s == "Ws.subscribe[0, 1]"));
    // Nothing ran at build, and Ws was constructed before it was told its keys.
    assert!(built.iter().all(|s| s.ends_with(".new") || s.contains("subscribe")), "{built:?}");
    let new_at = built.iter().position(|s| s == "Ws.new").unwrap();
    let sub_at = built.iter().position(|s| s == "Ws.subscribe[0, 1]").unwrap();
    assert!(new_at < sub_at);
    p.push(Msg { key: Some(0), v: 5 });
    let pushed = take();
    assert!(pushed.iter().all(|s| !s.contains("subscribe")), "subscribe is not repeated: {pushed:?}");
    assert!(!pushed.is_empty());
}

#[test]
fn different_symbols_are_exclusive_and_only_the_matching_one_runs() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    take();
    let mut p = P::build(Configs::new()).unwrap();
    take();
    p.push(Msg { key: Some(0), v: 7 });
    assert_eq!(take(), ["Ws.update", "Book<Btc>", "Join", "Vwap<Btc>"]);
    p.push(Msg { key: Some(1), v: 8 });
    assert_eq!(take(), ["Ws.update", "Book<Eth>", "Join"]);
    // A symbol nobody reads, a message without a key, and Sol (read through Ws2 only).
    p.push(Msg { key: Some(9), v: 1 });
    assert_eq!(take(), ["Ws.update"]);
    p.push(Msg { key: None, v: 1 });
    assert_eq!(take(), ["Ws.update"]);
    p.push(Msg { key: Some(2), v: 3 });
    assert_eq!(take(), ["Ws.update", "Direct"]);
}
