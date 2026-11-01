//! The vocabulary: wire packets, the exchange enum, symbols and the keyed frame.

use dataflow_flat::{Demux, Keyed, Node, Symbol};

/// What one exchange sends for one market: raw integer ticks, as on a wire.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BinanceMsg {
    pub symbol: u32,
    pub bid: i64,
    pub ask: i64,
    pub seq: u64,
}

/// Coinbase uses another layout (a product id and a mid/half-spread pair), so the
/// two listeners really do different framing work.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoinbaseMsg {
    pub product: u32,
    pub mid: i64,
    pub half_spread: i64,
    pub seq: u64,
}

/// One packet from the NIC. It is one enum, so a packet is from exactly one
/// exchange: the selectors `packet::Binance` and `packet::Coinbase` are
/// mutually exclusive, which is what lets a component join both exchanges.
#[derive(Clone, Copy, Debug, PartialEq, Demux)]
pub enum Packet {
    Binance(BinanceMsg),
    Coinbase(CoinbaseMsg),
}

/// A normalised top-of-book update, the same for every exchange.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tob {
    pub bid: i64,
    pub ask: i64,
    pub seq: u64,
}

/// What a listener publishes: a `Tob` tagged with its symbol key. `Keyed` is
/// what `On<S>` selects from.
#[derive(Clone, Copy, Debug, Default)]
pub struct Frame {
    pub key: Option<u32>,
    pub tob: Tob,
}
impl Keyed for Frame {
    type Item = Tob;
    fn key(&self) -> Option<u32> {
        self.key
    }
    fn item(&self) -> &Tob {
        &self.tob
    }
}

/// An exchange: the listener component that frames its packets.
pub trait Exchange: 'static {
    const NAME: &'static str;
    type Ws: Node<Output = Frame>;
}

macro_rules! symbols {
    ($($name:ident = $key:expr),*) => {$(
        pub struct $name;
        impl Symbol for $name {
            const KEY: u32 = $key;
            const NAME: &'static str = stringify!($name);
        }
    )*};
}
// Dense keys, the same on both exchanges. Doge (3) is on the wire but nobody reads it.
symbols!(Btc = 0, Eth = 1, Sol = 2, Doge = 3);
