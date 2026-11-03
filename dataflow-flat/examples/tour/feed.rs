//! The input side: vocabulary, the root, and everything between the wire and the books.

use std::marker::PhantomData;

use dataflow_flat::{
    catalog, Component, ComponentSubscription, Concat, Demux, Request, Root, Select, UpdateHandler,
};

use crate::say;

// ---- Vocabulary -----------------------------------------------------------------------------

/// One marker type per market. Several enums share them (`Frame`, `FrameView`), so `Btc` selects
/// the `Btc` variant of either.
pub mod mkt {
    pub struct Btc;
    pub struct Eth;
    pub struct Sol;
    pub struct Doge;
}
pub use mkt::{Btc, Eth, Sol};

pub trait Symbol: 'static {
    const NAME: &'static str;
}
impl Symbol for mkt::Btc {
    const NAME: &'static str = "BTC";
}
impl Symbol for mkt::Eth {
    const NAME: &'static str = "ETH";
}
impl Symbol for mkt::Sol {
    const NAME: &'static str = "SOL";
}
impl Symbol for mkt::Doge {
    const NAME: &'static str = "DOGE";
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Quote {
    pub bid: i64,
    pub ask: i64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Trade {
    pub qty: u64,
}

/// A quote per market. `#[derive(Demux)]` writes one selector per variant; with `markers` it
/// implements `Select<Frame>` for the existing `mkt::*` types instead of making new ones.
#[derive(Clone, Copy, Debug, PartialEq, Demux)]
#[demux(markers = mkt)]
pub enum Frame {
    Btc(Quote),
    Eth(Quote),
    Sol(Quote),
    Doge(Quote),
}

/// One message from the wire. The derive makes the selectors `msg::Book`, `msg::Trade` and the
/// unit selector `msg::Heartbeat`; a message is one variant, so they exclude each other.
#[derive(Clone, Copy, Debug, PartialEq, Demux)]
pub enum Msg {
    Book(Frame),
    Trade(Trade),
    Heartbeat,
}

/// A market whose frames carry a `Quote`: what the generic components below are written over.
/// This is the one place that repeats the bounds; the rest of the code says `S: Sym`.
pub trait Sym: Symbol + Select<Frame, Out = Quote> {}
impl<S: Symbol + Select<Frame, Out = Quote>> Sym for S {}

/// Catalogs: a type-level list of markers, for `for` entries. `Concat` joins two catalogs.
pub type AllSymbols = Concat<catalog![Btc, Eth], catalog![Sol]>;

// ---- 1. Root --------------------------------------------------------------------------------

/// The root is the value given to `push`. It has no state and fires on every push.
#[derive(Root)]
pub struct Wire {
    pub seq: u64,
    pub msg: Option<Msg>,
}

// ---- 2. By-value component ------------------------------------------------------------------

/// A by-value component: `new` returns the state and a first output, `update` returns the next
/// output, and subscribers read `&Output` (stored between pushes). Use it by default.
/// `w: Wire` is a *named* subscription: `i.w` is a field of `Input`.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(w: Wire)]
pub struct Nic {
    last: u64,
}
impl Component for Nic {
    type Config = ();
    type Output = Option<Msg>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Option<Msg>) {
        (Nic { last: 0 }, None)
    }
    fn update(&mut self, i: Self::Input<'_>) -> Option<Msg> {
        if i.w.seq <= self.last {
            say(format!("Nic: seq {} is a replay, dropped", i.w.seq));
            return None;
        }
        self.last = i.w.seq;
        i.w.msg
    }
}

// ---- 10. Listener ---------------------------------------------------------------------------

/// What a listener is told: `Request<S>::add` is called once for each market `S` that some
/// subscriber reads through it, before anything is constructed. `Default` is the empty list.
#[derive(Default)]
pub struct Names(pub Vec<&'static str>);
impl<S: Symbol> Request<S> for Names {
    fn add(&mut self) {
        self.0.push(S::NAME);
    }
}

/// A listener: `type Requests` makes it one, and `new` reads `init.requests`. Use it when a
/// source (a websocket) should carry only what the graph reads. Doge is a variant of `Frame`
/// nobody reads, so it is never requested.
///
/// `Some(Msg::Book(f)): Nic` is a *selector chain*: `Some`, then the `Book` variant. The edge
/// fires only when both match, and `f` is `&Frame`.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(Msg::Book(f)): Nic)]
pub struct Ws;
impl Component for Ws {
    type Config = ();
    type Requests = Names;
    type Output = Option<Frame>;
    fn new((): (), init: Self::Init<'_>) -> (Self, Option<Frame>) {
        say(format!("Ws: subscribe to {:?}", init.requests.0));
        (Ws, None)
    }
    fn update(&mut self, i: Self::Input<'_>) -> Option<Frame> {
        Some(*i.f)
    }
}

// ---- 13. Generic user component -------------------------------------------------------------

/// A component generic over the market: `Book<Btc>` and `Book<Eth>` are two components.
/// `Some(S(q)): Ws` selects market `S` out of the listener's frame, so `Book<S>` runs only for
/// frames of its own market. One instance per type, however many components name it.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(S(q)): Ws)]
pub struct Book<S: Sym>(PhantomData<S>);
impl<S: Sym> Component for Book<S> {
    type Config = ();
    type Output = Quote;
    fn new((): (), _: Self::Init<'_>) -> (Self, Quote) {
        (Book(PhantomData), Quote::default())
    }
    fn update(&mut self, i: Self::Input<'_>) -> Quote {
        say(format!("Book<{}> {}/{}", S::NAME, i.q.bid, i.q.ask));
        *i.q
    }
}

/// An alias is not a new component: `BtcBook` and `Book<Btc>` are the same node.
pub type BtcBook = Book<Btc>;

// ---- 8, 9, 10. Handle components ------------------------------------------------------------

/// The view of a frame: `#[derive(Demux)]` on an enum *with a lifetime* writes selectors that
/// take a view (`SelectView`) instead of a stored value, so `Btc(q): Journal` hands `q` out as
/// `&Quote` without copying or storing anything.
#[derive(Demux)]
#[demux(markers = mkt)]
pub enum FrameView<'a> {
    Btc(&'a Quote),
    Eth(&'a Quote),
    Sol(&'a Quote),
    Doge(&'a Quote),
}

/// A handle component keeps its state in place and stores only a small *handle* (here the slot
/// of the last write). Subscribers get `View<'a>`, computed by `view` from the state and the
/// handle each time they run: no output is built or copied. Use it for big or in-place state.
/// This one borrows from its ring, and it is also a listener (`Requests`).
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(f): Ws)]
pub struct Journal {
    ring: [Frame; 4],
    n: usize,
}
impl Component for Journal {
    type Config = ();
    type Handle = usize;
    type View<'a> = FrameView<'a>;
    type Requests = Names;
    fn new((): (), init: Self::Init<'_>) -> (Self, usize) {
        say(format!("Journal: asked for {:?}", init.requests.0));
        (Journal { ring: [Frame::Doge(Quote::default()); 4], n: 0 }, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> usize {
        self.n += 1;
        self.ring[self.n % 4] = *i.f;
        self.n % 4
    }
    fn view<'a>(&'a self, h: &'a usize) -> FrameView<'a> {
        match &self.ring[*h] {
            Frame::Btc(q) => FrameView::Btc(q),
            Frame::Eth(q) => FrameView::Eth(q),
            Frame::Sol(q) => FrameView::Sol(q),
            Frame::Doge(q) => FrameView::Doge(q),
        }
    }
}

/// A handle component with an *owned* view: the handle is `()` and the view is a copy of the
/// running total. A subscriber's field is exactly `View<'a>`, here a plain `u64`.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(Msg::Trade(t)): Nic)]
pub struct Volume {
    total: u64,
}
impl Component for Volume {
    type Config = ();
    type Handle = ();
    type View<'a> = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Volume { total: 0 }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        self.total += i.t.qty;
    }
    fn view<'a>(&'a self, (): &'a ()) -> u64 {
        self.total
    }
}

// ---- 5. Stored selected input, exclusive join -----------------------------------------------

/// Two selected edges of one source: the component runs on a trade or on a heartbeat, which
/// can never be the same message (an exclusive join). `t` is read in steps where `Trade` did
/// not match, so the plan stores the *last matched trade*; before the first one, `new` and
/// `update` see `Trade::default()`. `Msg::Heartbeat: Nic` is a unit selector: no field.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(Msg::Trade(t)): Nic, Some(Msg::Heartbeat): Nic)]
pub struct LastQty;
impl Component for LastQty {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) {
        (LastQty, i.t.qty) // the initial value of a selected input: 0
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        i.t.qty
    }
}
