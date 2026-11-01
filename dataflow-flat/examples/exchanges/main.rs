//! Run with `cargo run -p dataflow-flat --example exchanges`.
//!
//! One NIC, two exchanges, three symbols, merged listeners:
//!
//! ```text
//! Poll (root) -> Nic -> Some -> packet::Binance   -> BinanceWs  -> On<S> -> Book<Binance, S>  -+
//!                    \-> Some -> packet::Coinbase -> CoinbaseWs -> On<S> -> Book<Coinbase, S> -+-> BestBid<S> -> Quoter<S> -> OrderSender
//! ```
//!
//! * Demuxing is on the edges (selectors), not nodes: `Some`, then the enum
//!   variant, then the symbol key (`On<S>`). A packet is from one exchange and
//!   has one symbol, so only that cone runs; everything else keeps its stored
//!   output.
//! * The plan knows which keys are read from each `#[listener]`; at build it
//!   calls `subscribe` once with exactly those. Doge is on the wire but nobody
//!   reads it, so it is never subscribed.
//! * `BestBid<S>` and `OrderSender` join branches that are mutually exclusive
//!   (different variants / keys at the same place), which the plan accepts. Join
//!   two branches that can fire together and it is a compile error.

mod book;
mod feed;
mod market;
mod strategy;

use std::collections::BTreeMap;

use dataflow_flat::{Configs, Pipeline};

use book::BOOK_RUNS;
use feed::{Nic, Poll, Wire};
use market::Packet;
use strategy::{OrderSender, QUOTER_RUNS};

const SEED: u64 = 0x5EED;
const POLLS: u64 = 40;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let configs = Configs::new().with::<Nic>(SEED);
    // The graph is everything upstream of the sink.
    let mut pipeline = Pipeline::<Poll, (OrderSender,)>::build(configs)?;

    println!("{} components, execution order:", pipeline.len());
    for (i, n) in pipeline.order().iter().enumerate() {
        println!("  {i:2}. {}", n.replace("exchanges::", ""));
    }

    println!("\nrunning {POLLS} polls");
    for n in 1..=POLLS {
        println!("poll {n}");
        pipeline.push(Poll);
    }

    // Replay the wire and count what each book and quoter must have seen.
    let mut want_books: BTreeMap<(String, String), u64> = BTreeMap::new();
    let mut want_quoters: BTreeMap<String, u64> = BTreeMap::new();
    let mut wire = Wire::new(SEED);
    for n in 1..=POLLS {
        let (x, k) = match wire.poll(n) {
            Some(Packet::Binance(m)) => ("Binance", m.symbol),
            Some(Packet::Coinbase(m)) => ("Coinbase", m.product),
            None => continue,
        };
        let s = match k {
            0 => "Btc",
            1 => "Eth",
            2 => "Sol",
            _ => continue, // Doge: nobody reads it
        };
        *want_books.entry((x.to_string(), s.to_string())).or_default() += 1;
        *want_quoters.entry(s.to_string()).or_default() += 1;
    }
    let got_books: BTreeMap<_, _> = BOOK_RUNS.with(|m| m.borrow().iter().map(|((x, s), n)| ((x.to_string(), s.to_string()), *n)).collect());
    let got_quoters: BTreeMap<_, _> = QUOTER_RUNS.with(|m| m.borrow().iter().map(|(s, n)| (s.to_string(), *n)).collect());
    println!("\nbook updates {got_books:?}\nquoter updates {got_quoters:?}");
    assert_eq!(got_books, want_books, "only the matching book ran for each packet");
    assert_eq!(got_quoters, want_quoters, "a quoter ran once per update of either exchange's book");
    assert!(got_books.values().sum::<u64>() < 3 * 2 * POLLS);
    println!("ok");
    Ok(())
}
