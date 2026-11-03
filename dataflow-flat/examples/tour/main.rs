//! A tour of every kind of component, in one small graph. Run with
//! `cargo run -p dataflow-flat --example tour`.
//!
//! ```text
//! Wire (root) -> Nic -+-> Some -> msg::Book -> Ws (listener) -+-> Some(S) -> Book<S> -+-> Gap ----------+
//!                     |                                       |                       +-> Risk (set) ---+--> Dash
//!                     |                                       |                       +-> Quoter<S> ----+--> Desk
//!                     |                                       +-> Journal (handle) -> Btc -> BtcSpread
//!                     +-> Some -> msg::Trade ----+-> LastQty -> Dash       Desk also reads Trade itself
//!                     +-> Some -> msg::Heartbeat -+            Volume (handle, owned view) -> Dash
//! ```
//!
//! Sinks: `Desk`, `Dash`, `BtcSpread`. Files: `feed.rs` (root, vocabulary, everything upstream
//! of the books), `strategy.rs` (readers and sinks). Each flavor is explained where it is
//! defined. A push runs only what a fired edge reaches, in depth-first post-order of the
//! subscription lists; everything else keeps its stored output. The asserts below pin the
//! lines each push prints.

// The derives generate `Input` structs and re-exports that this example never reads by name.
#![allow(dead_code, unused_imports)]

mod feed;
mod strategy;

use std::cell::RefCell;

use dataflow_flat::{Configs, Pipeline, Sinks};

use feed::{Frame, Msg, Quote, Trade, Wire};
use strategy::{Risk, BtcSpread, Dash, Desk};

thread_local! {
    static LOG: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}
/// Print a line and remember it, so `main` can check what each push did.
pub fn say(s: String) {
    println!("    {s}");
    LOG.with(|l| l.borrow_mut().push(s));
}
fn take() -> Vec<String> {
    LOG.with(|l| std::mem::take(&mut *l.borrow_mut()))
}

/// 13. Generic code over any pipeline of root `Wire` needs one bound, `S: Sinks<Wire>`.
fn replay<S: Sinks<Wire>>(p: &mut Pipeline<Wire, S>, wires: Vec<Wire>) -> Vec<Vec<String>> {
    wires
        .into_iter()
        .map(|w| {
            println!("push seq {}: {:?}", w.seq, w.msg);
            p.push(w);
            take()
        })
        .collect()
}

fn book(seq: u64, f: Frame) -> Wire {
    Wire { seq, msg: Some(Msg::Book(f)) }
}
fn q(bid: i64, ask: i64) -> Quote {
    Quote { bid, ask }
}

fn main() {
    // 1. `Pipeline<Root, Sinks>`: the graph is everything upstream of the sinks. `Configs`
    // gives `Risk` its limit; every other component gets `Default`.
    println!("build:");
    let mut p = Pipeline::<Wire, (Desk, Dash, BtcSpread)>::build(Configs::new().with::<Risk>(150)).unwrap();
    let construction = take();
    // Listeners are told what is read before anything is constructed. Sol is read (by `Risk`
    // and `Quoter<Sol>`), Doge is not.
    assert_eq!(construction[0], r#"Ws: subscribe to ["BTC", "ETH", "SOL"]"#);
    assert_eq!(construction[1], "Risk: 3 books, limit 150");
    assert_eq!(construction[2], r#"Journal: asked for ["BTC"]"#);

    let order: Vec<String> = p.order().iter().map(|n| n.replace("tour::", "").replace("feed::", "").replace("strategy::", "")).collect();
    println!("{} components, execution order:", p.len());
    for (i, n) in order.iter().enumerate() {
        println!("  {i:2}. {n}");
    }
    // `BtcBook` and `Book<Btc>` are one component, reached by `Gap`, `Risk` and `Quoter<Btc>`.
    assert_eq!(order.iter().filter(|n| n.contains("Book<")).count(), 3);
    assert_eq!(order.iter().filter(|n| n.ends_with("Book<mkt::Btc>")).count(), 1);
    assert_eq!(p.len(), 16);

    let runs = replay(
        &mut p,
        vec![
            book(1, Frame::Btc(q(100, 102))),
            book(2, Frame::Eth(q(50, 51))),
            Wire { seq: 3, msg: Some(Msg::Trade(Trade { qty: 5 })) },
            Wire { seq: 4, msg: Some(Msg::Heartbeat) },
            Wire { seq: 4, msg: Some(Msg::Heartbeat) }, // a replay: `Nic` drops it
            book(5, Frame::Sol(q(20, 21))),
            book(6, Frame::Doge(q(1, 2))), // on the wire, read by nobody
            book(7, Frame::Btc(q(101, 103))),
        ],
    );
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    // A Btc book: `Book<BTC>` runs, then what depends on it, sink by sink in the order of the
    // sinks tuple (`Desk`, `Dash`, `BtcSpread`), each after its own upstream.
    assert_eq!(
        runs[0],
        s(&[
            "Book<BTC> 100/102",
            "Risk: btc 100 total 100 bids [100, 0, 0]",
            "Desk: breached false orders [(\"BTC\", 101)] trade prints 0",
            "Dash: gap 100 last_qty 0 volume 0",
            "BtcSpread: 2",
        ])
    );
    // An Eth book: `Gap` reads the stored Btc book. `BtcSpread` does not run: the journal's
    // frame is an Eth one now.
    assert_eq!(
        runs[1],
        s(&[
            "Book<ETH> 50/51",
            "Risk: btc 100 total 150 bids [100, 50, 0]",
            "Desk: breached false orders [(\"ETH\", 51)] trade prints 0",
            "Dash: gap 50 last_qty 0 volume 0",
        ])
    );
    // A trade: neither books nor quoters run. `Desk` runs for its selected edge alone.
    assert_eq!(
        runs[2],
        s(&["Desk: breached false orders [] trade prints 1", "Dash: gap 50 last_qty 5 volume 5"])
    );
    // A heartbeat: `LastQty` runs and reads the last matched trade.
    assert_eq!(runs[3], s(&["Dash: gap 50 last_qty 5 volume 5"]));
    assert_eq!(runs[4], s(&["Nic: seq 4 is a replay, dropped"]));
    // Sol pushes the total over the limit. `Gap` does not read Sol, so `Dash` stays quiet.
    assert_eq!(
        runs[5],
        s(&[
            "Book<SOL> 20/21",
            "Risk: btc 100 total 170 bids [100, 50, 20]",
            "Desk: breached true orders [(\"SOL\", 21)] trade prints 1",
        ])
    );
    // Doge: `Ws` and `Journal` run (silently) and no book matches, so nothing downstream does.
    assert!(runs[6].is_empty());
    assert_eq!(
        runs[7],
        s(&[
            "Book<BTC> 101/103",
            "Risk: btc 101 total 171 bids [101, 50, 20]",
            "Desk: breached true orders [(\"BTC\", 102)] trade prints 1",
            "Dash: gap 51 last_qty 5 volume 5",
            "BtcSpread: 2",
        ])
    );
    println!("ok");
}
