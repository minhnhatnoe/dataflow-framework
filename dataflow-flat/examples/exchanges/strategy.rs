//! Everything downstream of the books: the best bid across exchanges, a quoter
//! per symbol, and the sink.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use dataflow_flat::{Component, ComponentSubscription, Symbol};

use crate::book::Book;
use crate::feed::{Binance, Coinbase};
use crate::market::*;

/// Best bid/ask over both exchanges. It reads two books that can never update in
/// the same step (one packet, one exchange): an *exclusive join*, accepted by
/// the plan. In a step where Binance's book updated, the Coinbase book supplies
/// its stored (older) value.
#[derive(ComponentSubscription)]
#[subscribe(b: Book<Binance, S>, c: Book<Coinbase, S>)]
pub struct BestBid<S: Symbol>(PhantomData<S>);
impl<S: Symbol> Component for BestBid<S> {
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
pub struct Quoter<S: Symbol>(PhantomData<S>);
impl<S: Symbol> Component for Quoter<S> {
    type Config = ();
    type Output = Order;
    fn new((): (), _: Self::Init<'_>) -> (Self, Order) {
        (Quoter(PhantomData), Order::default())
    }
    fn update(&mut self, i: Self::Input<'_>) -> Order {
        QUOTER_RUNS.with(|m| *m.borrow_mut().entry(S::NAME).or_default() += 1);
        Order { symbol: S::NAME, buy: i.best.ask - i.best.bid > 3, px: i.best.bid + 1, seq: i.best.seq }
    }
}

/// The sink. It joins the three quoters (exclusive: one symbol per packet) and
/// sends the order of whichever quoter just ran, the one with the newest `seq`.
#[derive(ComponentSubscription)]
#[subscribe(btc: Quoter<Btc>, eth: Quoter<Eth>, sol: Quoter<Sol>)]
pub struct OrderSender {
    pub sent: Vec<Order>,
}
impl Component for OrderSender {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (OrderSender { sent: Vec::new() }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        let o = [i.btc, i.eth, i.sol].into_iter().max_by_key(|o| o.seq).unwrap();
        println!("  order: {} {} @ {} (packet #{})", if o.buy { "BUY " } else { "SELL" }, o.symbol, o.px, o.seq);
        self.sent.push(*o);
    }
}
