//! The output side: derived values, a set reader, and the sinks.

use std::marker::PhantomData;

use dataflow_flat::{Component, ComponentSubscription, UpdateHandler};

use crate::feed::*;
use crate::say;

/// Exclusive join of two books. One message is one market, so `BtcBook` and `Book<Eth>` never
/// run in the same step; the one that did not run supplies its stored (older) output.
/// `BtcBook` is an alias spelling of `Book<Btc>`; `Risk` below reaches the same component.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(btc: BtcBook, eth: Book<Eth>)]
pub struct Gap;
impl Component for Gap {
    type Config = ();
    type Output = i64;
    fn new((): (), _: Self::Init<'_>) -> (Self, i64) {
        (Gap, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> i64 {
        i.btc.bid - i.eth.bid
    }
}

/// One quoter per market, generic like `Book<S>`.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(b: Book<S>)]
pub struct Quoter<S: Sym>(PhantomData<S>);
impl<S: Sym> Component for Quoter<S> {
    type Config = ();
    type Output = i64;
    fn new((): (), _: Self::Init<'_>) -> (Self, i64) {
        (Quoter(PhantomData), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> i64 {
        i.b.bid + 1
    }
}

// ---- 3, 12. Config and a set subscription ----------------------------------------------------

/// `quotes: for S: Sym in AllSymbols { Book<S> }` is a *set*: one input, `quotes`, that reads
/// every element of the catalog. It runs when any element runs; elements that did not run
/// give their last output. `get::<S>()` reads one element, `fold` and `for_each` visit all in
/// catalog order, `len()` counts them, and all four work in `new` too. Use it for "all
/// markets" readers instead of listing each one.
///
/// `type Config` is the owned value `Configs::new().with::<Risk>(limit)` passes to `new`.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(quotes: for S: Sym in AllSymbols { Book<S> })]
pub struct Risk {
    limit: i64,
}
impl Component for Risk {
    type Config = i64; // the exposure limit
    type Output = bool; // limit breached
    fn new(limit: i64, i: Self::Init<'_>) -> (Self, bool) {
        say(format!("Risk: {} books, limit {limit}", i.quotes.len()));
        assert_eq!(i.quotes.fold(0, |a, q: &Quote| a + q.bid), 0);
        (Risk { limit }, false)
    }
    fn update(&mut self, i: Self::Input<'_>) -> bool {
        let total = i.quotes.fold(0, |a, q| a + q.bid);
        let mut bids = Vec::new();
        i.quotes.for_each(|q| bids.push(q.bid));
        say(format!("Risk: btc {} total {total} bids {bids:?}", i.quotes.get::<Btc>().bid));
        total > self.limit
    }
}

// ---- Plain sinks -----------------------------------------------------------------------------

/// A sink is an ordinary component nobody subscribes to; it is how the graph gets observed.
/// `Gap`, `LastQty` and `Volume` join exclusive branches (books against trades and heartbeats),
/// and `vol` is the owned view of a handle component: a plain `u64` field.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(gap: Gap, qty: LastQty, vol: Volume)]
pub struct Dash;
impl Component for Dash {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Dash, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        say(format!("Dash: gap {} last_qty {} volume {}", i.gap, i.qty, i.vol));
    }
}

/// A selector on a handle source: `Btc(q): Journal` runs only when the journal's current frame
/// is a Btc one, and `q` is `&Quote` borrowed from the journal's ring.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Btc(q): Journal)]
pub struct BtcSpread;
impl Component for BtcSpread {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (BtcSpread, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        say(format!("BtcSpread: {}", i.q.ask - i.q.bid));
    }
}

// ---- 7, 11. A handler component --------------------------------------------------------------

/// A component with handlers: no `UpdateHandler` derive, one hand-written impl per entry.
/// `on_update` is called, before `update`, once for each handled edge that fired in the step.
///
/// * `risk: Risk` is a named edge that needs no call.
/// * `for S: Sym in AllSymbols { Quoter<S> }` is an *unnamed* `for` entry: one handled call per
///   element, in catalog order. Nothing reads these in `update`, they only trigger and call.
/// * `Some(Msg::Trade(_)): Nic` is a *selected* unnamed entry. Its key is the marker the
///   derive generates in `desk_edges`, named after the innermost selector.
#[derive(ComponentSubscription)]
#[subscribe(risk: Risk, for S: Sym in AllSymbols { Quoter<S> }, Some(Msg::Trade(_)): Nic)]
pub struct Desk {
    orders: Vec<(&'static str, i64)>,
    prints: u32,
}
impl Component for Desk {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Desk { orders: Vec::new(), prints: 0 }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        say(format!("Desk: breached {} orders {:?} trade prints {}", i.risk, self.orders, self.prints));
        self.orders.clear();
    }
}
/// Skip an edge: `ACTIVE = false` says there is never a call (a bare `{}` also compiles and does
/// nothing, but keeps the call in the push). `Risk` is still read as `i.risk` in `update`.
impl UpdateHandler<Risk> for Desk {
    const ACTIVE: bool = false;
}
/// An active handler for a whole family: called for whichever quoter updated.
impl<S: Sym> UpdateHandler<Quoter<S>> for Desk {
    fn on_update(&mut self, _input: Self::Input<'_>, px: &i64) {
        self.orders.push((S::NAME, *px));
    }
}
/// A selected edge through the generated marker; `t` is the selected part, `&Trade`.
impl UpdateHandler<desk_edges::Trade> for Desk {
    fn on_update(&mut self, _input: Self::Input<'_>, _t: &Trade) {
        self.prints += 1;
    }
}
