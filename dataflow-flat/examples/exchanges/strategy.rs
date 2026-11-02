//! Everything downstream of the books: the best bid across exchanges, a quoter
//! per symbol, and the sink.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use dataflow_flat::{Collect, Component, ComponentSubscription, Handler, UpdateHandler};

use crate::book::Book;
use crate::feed::{Binance, Coinbase};
use crate::market::markets::{Btc, Eth, Sol};
use crate::market::*;

/// Best bid/ask over both exchanges. It reads two books that can never update in
/// the same step (one packet, one exchange): an *exclusive join*, accepted by
/// the plan. In a step where Binance's book updated, the Coinbase book supplies
/// its stored (older) value.
#[derive(ComponentSubscription)]
#[subscribe(b: Book<Binance, S>, c: Book<Coinbase, S>)]
pub struct BestBid<S: Sym>(PhantomData<S>);
impl<S: Sym> Component for BestBid<S> {
    type Config = ();
    type Output = Tob;
    fn new((): (), _: Self::Init<'_>) -> (Self, Tob) {
        (BestBid(PhantomData), Tob::default())
    }
    fn update(&mut self, i: Self::Input<'_>) -> Tob {
        Tob {
            bid: i.b.bid.max(i.c.bid),
            ask: match (i.b.ask, i.c.ask) {
                (0, a) | (a, 0) => a,
                (a, b) => a.min(b),
            },
            seq: i.b.seq.max(i.c.seq),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Order {
    pub symbol: &'static str,
    pub buy: bool,
    pub px: i64,
    pub seq: u64,
}

thread_local! {
    pub static QUOTER_RUNS: RefCell<BTreeMap<&'static str, u64>> = RefCell::new(BTreeMap::new());
}

/// Quotes one tick inside the best bid when the market is not crossed.
#[derive(ComponentSubscription)]
#[subscribe(best: BestBid<S>)]
pub struct Quoter<S: Sym>(PhantomData<S>);
impl<S: Sym> Component for Quoter<S> {
    type Config = ();
    type Output = Order;
    fn new((): (), _: Self::Init<'_>) -> (Self, Order) {
        (Quoter(PhantomData), Order::default())
    }
    fn update(&mut self, i: Self::Input<'_>) -> Order {
        QUOTER_RUNS.with(|m| *m.borrow_mut().entry(<S as Market>::NAME).or_default() += 1);
        Order { symbol: <S as Market>::NAME, buy: i.best.ask - i.best.bid > 3, px: i.best.bid + 1, seq: i.best.seq }
    }
}

thread_local! {
    /// How many `on_update` calls the sink got per symbol (initial ones excluded).
    pub static SENDER_UPDATES: RefCell<BTreeMap<&'static str, u64>> = RefCell::new(BTreeMap::new());
}

/// The sink, a *handler component*: it handles the three quoters. In a step it
/// is called only for the quoter that updated (one symbol per packet), with no
/// look at the others: `begin`, one `on_update`, `finish`. Which quoter that is
/// is known statically inside each branch of the push, so this compiles to
/// straight-line code.
///
/// At build the framework also calls `on_update` with each quoter's initial
/// output (the default `Order`, `seq == 0`), which the handler ignores.
#[derive(Handler)]
#[handles(Quoter<Btc>, Quoter<Eth>, Quoter<Sol>)]
pub struct OrderSender {
    pub sent: Vec<Order>,
}
impl Collect for OrderSender {
    type Config = ();
    type Output = ();
    type Buf = Vec<Order>;
    fn new((): ()) -> Self {
        OrderSender { sent: Vec::new() }
    }
    fn begin(&mut self) -> Vec<Order> {
        Vec::new()
    }
    fn finish(&mut self, buf: Vec<Order>, _: Self::Input<'_>) {
        for o in buf {
            println!("  order: {} {} @ {} (packet #{})", if o.buy { "BUY " } else { "SELL" }, o.symbol, o.px, o.seq);
            self.sent.push(o);
        }
    }
}
impl<S: Sym> UpdateHandler<Quoter<S>> for OrderSender {
    fn on_update(&mut self, buf: &mut Vec<Order>, o: &Order) {
        if o.seq != 0 {
            SENDER_UPDATES.with(|m| *m.borrow_mut().entry(<S as Market>::NAME).or_default() += 1);
            buf.push(*o);
        }
    }
}
