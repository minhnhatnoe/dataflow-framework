//! The input side: the NIC component and one listener per exchange.

use std::cell::RefCell;
use std::collections::BTreeMap;

use dataflow_flat::{Component, ComponentSubscription, UpdateHandler, Listener, Request, Root};

use crate::market::*;

/// The root: one NIC card, polled once per push. It carries nothing; the
/// packet comes from the [`Nic`] component, which owns the (simulated) hardware.
#[derive(Root)]
pub struct Poll;

/// A deterministic packet generator (xorshift): the simulated wire. The
/// example's check replays it to know what every push must have caused.
pub struct Wire(u64);
impl Wire {
    pub fn new(seed: u64) -> Self {
        Wire(seed.max(1))
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    /// The next poll: nothing, or a packet (`seq` is the poll number).
    pub fn poll(&mut self, seq: u64) -> Option<Packet> {
        let r = self.next_u64();
        let symbol = ((r >> 8) % 4) as u32; // 0..=3, Doge included
        let px = 1_000 * (symbol as i64 + 1) + ((r >> 16) % 50) as i64;
        match r % 8 {
            0 => None,
            1..=4 => Some(Packet::Binance(BinanceMsg { symbol, bid: px, ask: px + 2 + (r >> 40) as i64 % 6, seq })),
            _ => {
                let half = 1 + (r >> 44) as i64 % 4;
                Some(Packet::Coinbase(CoinbaseMsg { product: symbol, mid: px + 1, half_spread: half, seq }))
            }
        }
    }
}

/// Reads the card: `Output = Option<Packet>`, `None` when the poll found nothing.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(_p: Poll)]
pub struct Nic {
    wire: Wire,
    polls: u64,
}
impl Component for Nic {
    type Config = u64; // the seed of the simulation
    type Output = Option<Packet>;
    fn new(seed: u64, _: Self::Init<'_>) -> (Self, Option<Packet>) {
        (Nic { wire: Wire::new(seed), polls: 0 }, None)
    }
    fn update(&mut self, _: Self::Input<'_>) -> Option<Packet> {
        self.polls += 1;
        self.wire.poll(self.polls)
    }
}

thread_local! {
    /// What each listener was asked for (exchange name -> requests, in `add` order).
    pub static REQUESTED: RefCell<BTreeMap<&'static str, Vec<&'static str>>> = RefCell::new(BTreeMap::new());
}

/// The Binance requests: one stream per market a subscriber reads.
#[derive(Default)]
pub struct Streams(pub Vec<&'static str>);
impl<S: BinanceStream> Request<S> for Streams {
    fn add(&mut self) {
        self.0.push(S::STREAM);
    }
}

/// Subscribes to `Some(Packet::Binance(m))` of `Nic` (`Some`, then `packet::Binance`): it runs only for a
/// Binance packet. `#[listener]` tells the plan that this component wants to be
/// told, at build and before `new`, the markets its subscribers read from it.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(Packet::Binance(m)): Nic)]
#[listener]
#[allow(dead_code)] // kept: a real listener owns its connection
pub struct BinanceWs {
    streams: Vec<&'static str>,
}
impl Listener for BinanceWs {
    type Requests = Streams;
}
impl Component for BinanceWs {
    type Config = ();
    type Output = Option<BinanceFrame>;
    fn new((): (), init: Self::Init<'_>) -> (Self, Option<BinanceFrame>) {
        // A real listener would open one websocket with a combined stream for
        // exactly these markets.
        let streams = init.requests.0;
        println!("BinanceWs: one connection, streams {streams:?}");
        REQUESTED.with(|r| r.borrow_mut().insert("Binance", streams.clone()));
        (BinanceWs { streams }, None)
    }
    fn update(&mut self, i: Self::Input<'_>) -> Option<BinanceFrame> {
        let m = i.m;
        let t = Tob { bid: m.bid, ask: m.ask, seq: m.seq };
        Some(match m.symbol {
            0 => BinanceFrame::Btc(t),
            1 => BinanceFrame::Eth(t),
            2 => BinanceFrame::Sol(t),
            _ => BinanceFrame::Doge(t),
        })
    }
}

/// The Coinbase requests: one product per market.
#[derive(Default)]
pub struct Products(pub Vec<&'static str>);
impl<S: CoinbaseProduct> Request<S> for Products {
    fn add(&mut self) {
        self.0.push(S::PRODUCT);
    }
}

/// The same for Coinbase, with its own framing.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(Packet::Coinbase(m)): Nic)]
#[listener]
#[allow(dead_code)]
pub struct CoinbaseWs {
    products: Vec<&'static str>,
}
impl Listener for CoinbaseWs {
    type Requests = Products;
}
impl Component for CoinbaseWs {
    type Config = ();
    type Output = Option<CoinbaseFrame>;
    fn new((): (), init: Self::Init<'_>) -> (Self, Option<CoinbaseFrame>) {
        let products = init.requests.0;
        println!("CoinbaseWs: one connection, channels {products:?}");
        REQUESTED.with(|r| r.borrow_mut().insert("Coinbase", products.clone()));
        (CoinbaseWs { products }, None)
    }
    fn update(&mut self, i: Self::Input<'_>) -> Option<CoinbaseFrame> {
        let m = i.m;
        let t = Tob { bid: m.mid - m.half_spread, ask: m.mid + m.half_spread, seq: m.seq };
        Some(match m.product {
            0 => CoinbaseFrame::Btc(t),
            1 => CoinbaseFrame::Eth(t),
            2 => CoinbaseFrame::Sol(t),
            _ => CoinbaseFrame::Doge(t),
        })
    }
}

pub struct Binance;
impl Exchange for Binance {
    const NAME: &'static str = "Binance";
    type Frame = BinanceFrame;
    type Ws = BinanceWs;
}
pub struct Coinbase;
impl Exchange for Coinbase {
    const NAME: &'static str = "Coinbase";
    type Frame = CoinbaseFrame;
    type Ws = CoinbaseWs;
}
