//! The vocabulary: wire packets, the exchange enum, symbols and the keyed frame.

use dataflow_flat::{Demux, Node, Select};

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

/// The markets, as types. One marker per market, shared by every exchange's
/// frame enum: `Btc` selects the Btc variant of `BinanceFrame` and of
/// `CoinbaseFrame`. Doge is on the wire but nobody reads it.
pub mod markets {
    pub struct Btc;
    pub struct Eth;
    pub struct Sol;
    pub struct Doge;
}

/// What the Binance listener publishes: the top of book of one market, as the
/// variant of that market. The variants are the selectors (`markets::Btc`, ...).
#[derive(Clone, Copy, Debug, PartialEq, Demux)]
#[demux(markers = markets)]
pub enum BinanceFrame {
    Btc(Tob),
    Eth(Tob),
    Sol(Tob),
    Doge(Tob),
}

/// The same for Coinbase, with the same markers.
#[derive(Clone, Copy, Debug, PartialEq, Demux)]
#[demux(markers = markets)]
pub enum CoinbaseFrame {
    Btc(Tob),
    Eth(Tob),
    Sol(Tob),
    Doge(Tob),
}

/// A market's name, for reports.
pub trait Market: 'static {
    const NAME: &'static str;
}
/// The Binance stream of a market.
pub trait BinanceStream {
    const STREAM: &'static str;
}
/// The Coinbase product of a market.
pub trait CoinbaseProduct {
    const PRODUCT: &'static str;
}

/// A market that both exchanges frame: what the books and the joins are generic over.
pub trait Sym: Market + Select<BinanceFrame, Out = Tob> + Select<CoinbaseFrame, Out = Tob> {}
impl<S: Market + Select<BinanceFrame, Out = Tob> + Select<CoinbaseFrame, Out = Tob>> Sym for S {}

/// An exchange: the listener component that frames its packets.
pub trait Exchange: 'static {
    const NAME: &'static str;
    type Frame: 'static;
    type Ws: Node<Output = Option<Self::Frame>>;
}

macro_rules! markets {
    ($($name:ident = $bin:expr, $cb:expr;)*) => {$(
        impl Market for markets::$name {
            const NAME: &'static str = stringify!($name);
        }
        impl BinanceStream for markets::$name {
            const STREAM: &'static str = $bin;
        }
        impl CoinbaseProduct for markets::$name {
            const PRODUCT: &'static str = $cb;
        }
    )*};
}
markets! {
    Btc = "btcusdt@bookTicker", "BTC-USD";
    Eth = "ethusdt@bookTicker", "ETH-USD";
    Sol = "solusdt@bookTicker", "SOL-USD";
    Doge = "dogeusdt@bookTicker", "DOGE-USD";
}
