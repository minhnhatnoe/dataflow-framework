//! The input side: the NIC component and one listener per exchange.

use std::collections::BTreeSet;

use dataflow_flat::{Component, ComponentSubscription, Listener, Root};

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
#[derive(ComponentSubscription)]
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

/// Subscribes to `Some(Packet::Binance(m))` of `Nic` (`Some`, then `packet::Binance`): it runs only for a
/// Binance packet. `#[listener]` tells the plan that this component wants to be
/// told, at build, the keys that `On<_>` subscribers read from it.
#[derive(ComponentSubscription)]
#[subscribe(Some(Packet::Binance(m)): Nic)]
#[listener]
pub struct BinanceWs {
    wanted: BTreeSet<u32>,
}
impl Listener for BinanceWs {
    fn subscribe(&mut self, keys: &[u32]) {
        // A real listener would open one websocket with a combined stream for
        // exactly these symbols.
        self.wanted = keys.iter().copied().collect();
        println!("BinanceWs.subscribe: one connection, streams {}", names(keys));
    }
}
impl Component for BinanceWs {
    type Config = ();
    type Output = Frame;
    fn new((): (), _: Self::Init<'_>) -> (Self, Frame) {
        (BinanceWs { wanted: BTreeSet::new() }, Frame::default())
    }
    fn update(&mut self, i: Self::Input<'_>) -> Frame {
        let m = i.m;
        Frame { key: Some(m.symbol), tob: Tob { bid: m.bid, ask: m.ask, seq: m.seq } }
    }
}

/// The same for Coinbase, with its own framing.
#[derive(ComponentSubscription)]
#[subscribe(Some(Packet::Coinbase(m)): Nic)]
#[listener]
pub struct CoinbaseWs {
    wanted: BTreeSet<u32>,
}
impl Listener for CoinbaseWs {
    fn subscribe(&mut self, keys: &[u32]) {
        self.wanted = keys.iter().copied().collect();
        println!("CoinbaseWs.subscribe: one connection, channels {}", names(keys));
    }
}
impl Component for CoinbaseWs {
    type Config = ();
    type Output = Frame;
    fn new((): (), _: Self::Init<'_>) -> (Self, Frame) {
        (CoinbaseWs { wanted: BTreeSet::new() }, Frame::default())
    }
    fn update(&mut self, i: Self::Input<'_>) -> Frame {
        let m = i.m;
        Frame {
            key: Some(m.product),
            tob: Tob { bid: m.mid - m.half_spread, ask: m.mid + m.half_spread, seq: m.seq },
        }
    }
}

fn names(keys: &[u32]) -> String {
    let n: Vec<_> = keys
        .iter()
        .map(|k| match *k {
            0 => "BTC",
            1 => "ETH",
            2 => "SOL",
            3 => "DOGE",
            _ => "?",
        })
        .collect();
    format!("{n:?}")
}

pub struct Binance;
impl Exchange for Binance {
    const NAME: &'static str = "Binance";
    type Ws = BinanceWs;
}
pub struct Coinbase;
impl Exchange for Coinbase {
    const NAME: &'static str = "Coinbase";
    type Ws = CoinbaseWs;
}
