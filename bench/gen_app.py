#!/usr/bin/env python3
"""Generator for the realistic multi-crate market app (benchmark / profiling harness).

usage: bench/gen_app.py --size tiny|small|typical|perps|full|m<L>-<B>-<H> [--mode 1a|enum|req|on] [--out DIR] [--lib CHECKOUT]
                        [--markets L,B,H] [--sink-scan N|all] [--big-width N] [--core-inline none|hint]

Writes a self-contained cargo workspace to DIR (default: ./gen/<size>-<mode>):

  marketlib/   the stable library: Symbol catalog (one type per market, KEY/NAME), the wire types,
               non-generic cores (BookCore, ...), thin generic components (Book<W,S>, Mid<W,S>, ...)
  app/         the wiring: per-exchange listener aliases + sinks, cross-venue joins, risk, one big component
  baseline/    the hand-written equivalent (nested match on exchange / key / kind; stored outputs in fields)
  runner/      binary: builds both, verifies they agree, times them (features `fw` and `base`)

Modes:
  1a   phase-1a master: no selectors, every component runs on every push. Components guard themselves
       (key/kind/seq checks) so that results equal the cone-only semantics. The baseline is built in two
       flavours: `run_batch` (cone only: the target) and `run_all_batch` (every component per packet).
  enum selectors via one `#[derive(Demux)]` enum per exchange with a variant per market
       (`#[subscribe(u: W => <S as Symbol>::Sel, body::Depth)]`); works on phase 1b. The baseline is a
       `match` on the same enum. Every component runs only when its edges fire.
  req  `enum`, but each exchange's feed component is a `#[listener]` whose `Requests` counts the distinct
       selectors the graph reads from it (`Request<S>::add`); measures the cost of the request pass.
  on   the spec's keyed selector: `#[subscribe(u: W => On<S>, body::Depth)]` plus `Keyed` for the frame.
       Not on master yet: generated against the spec, UNTESTED.

Layout of what is generated (per market: Book, Trades, Mid, Imbalance, Spread, Vwap, Vol, Momentum, Fair, Quoter):

  marketlib/src/{catalog/{lighter,binance,hyper}.rs, cores.rs, components.rs, wire.rs, symbol.rs, feed_gen.rs, stats.rs}
  app/src/{lib.rs, cross.rs, risk.rs, big.rs, lighter/{mod,sender}.rs, binance/..., hyper/...}
      cross.rs  one Cross2/Cross3 join per base asset listed on >= 2 venues (feeds the quoters of those markets
                and the Risk sink); big.rs the wide, long-bodied component; senders/risk are wide sinks
      Sinks = (lighter::Sender, binance::Sender, hyper::Sender, risk::Risk, big::Big)
      `// EDIT` lines (let tweak: u64 = ...) in big.rs, risk.rs, lighter/sender.rs, hyper/sender.rs are what
      scripts/measure.py rewrites for the dev incremental measurements
  baseline/src/{hand.rs, arms.rs, all.rs}   hand-written, one arm per market, `run_batch` / `run_all_batch`
  runner/src/main.rs                        verification + timing; features fw / base

Each sink keeps an idempotent digest of (a sample of) its inputs and writes it to a static; quoters XOR a
checksum into a static whenever their quote changes. The runner asserts that framework and baseline leave the
same values behind after the same packet stream.
"""
import argparse, os, shutil, sys

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
REQ = False  # mode req: enum plus #[listener] feeds (set in main)
LIB = REPO  # the checkout whose dataflow-flat / dataflow-macros are used (--lib)

# Spec table: Lighter 224, Binance 2214, Hyperliquid 527 markets (total 2965; perps 213+827+179).
SIZES = {
    "tiny": (1, 7, 2),
    "small": (4, 30, 6),
    "typical": (6, 60, 14),
    "perps": (213, 827, 179),
    "full": (224, 2214, 527),
}
EX = [  # module, Wire variant, listener type
    ("lighter", "Lighter", "LighterWs"),
    ("binance", "Binance", "BinanceWs"),
    ("hyper", "Hyper", "HyperWs"),
]
TICKERS = "BTC ETH SOL BNB XRP DOGE ADA AVAX LINK DOT LTC TRX".split()


def ticker(b):
    return TICKERS[b] if b < len(TICKERS) else f"T{b}"


def mname(e, b):
    t = ticker(b)
    return [f"{t}-PERP", f"{t}USDT", t][e]


CARGO_CONFIG = """[build]
# Without this, exported_generic_symbols re-runs on any edit that changes a line's length (dev rebuild 4.9 s vs 0.8 s).
rustflags = ["-Zshare-generics=n"]
"""


def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        f.write(text)


# ---------------------------------------------------------------------------------------------
# The model of the graph (shared by the app wiring and the baseline)
# ---------------------------------------------------------------------------------------------
class Model:
    def __init__(self, counts, scan, big_width):
        self.n = list(counts)
        self.scan = scan
        # base b exists on venue e iff b < n[e]; a cross-venue join exists for bases on >= 2 venues.
        self.cross = []  # list of lists of (e, key)
        self.cross_of = {}  # (e, key) -> index
        for b in range(max(self.n)):
            mem = [(e, b) for e in range(3) if b < self.n[e]]
            if len(mem) >= 2:
                for m in mem:
                    self.cross_of[m] = len(self.cross)
                self.cross.append(mem)
        # sinks
        self.quoters = [[(e, k) for k in range(self.n[e])] for e in range(3)]
        # Risk: all crosses; falls back to the first quoter of each exchange when there are none.
        if self.cross:
            self.risk_deps = [("x", i) for i in range(len(self.cross))]
        else:
            self.risk_deps = [("q", e, 0) for e in range(3)]
        # Big: Fair values of the first `big_width // 3` markets of each exchange
        per = max(1, big_width // 3)
        self.big_deps = [(e, k) for e in range(3) for k in range(min(per, self.n[e]))]

    def sample(self, lst):
        if self.scan == "all":
            return list(range(len(lst)))
        k = min(int(self.scan), len(lst))
        return sorted({j * len(lst) // k for j in range(k)})

    def components(self):
        mk = sum(self.n)
        return 10 * mk + len(self.cross) + 3 + 5 + 1  # markets, crosses, listeners, sinks, root

    def edges(self):
        e = 0
        for ex in range(3):
            for k in range(self.n[ex]):
                e += 17 + (1 if (ex, k) in self.cross_of else 0)
                e += 1  # the sender's edge
        e += sum(len(c) for c in self.cross) + len(self.cross) + len(self.big_deps) + 3
        return e


# ---------------------------------------------------------------------------------------------
# marketlib
# ---------------------------------------------------------------------------------------------
def gen_lib(out, M, MODE, core_inline):
    SEL, ENUM, ON = MODE != "1a", MODE == "enum", MODE == "on"
    inl = "#[inline]" if core_inline == "hint" else ""
    lib = f"{out}/marketlib"
    write(f"{lib}/Cargo.toml", f"""[package]
name = "marketlib"
version = "0.1.0"
edition = "2021"

[dependencies]
dataflow-flat = {{ path = "{LIB}/dataflow-flat" }}
""")
    sel_use = {"1a": "", "enum": "use dataflow_flat::Demux;\n", "on": "use dataflow_flat::{Demux, Keyed, On};\n"}[MODE]
    write(f"{lib}/src/lib.rs", """//! Stable market library: symbols as types, wire types, cores, thin generic components.
pub mod catalog;
pub mod cores;
pub mod components;
pub mod feed_gen;
pub mod stats;
pub mod symbol;
pub mod wire;
pub use components::*;
pub use cores::*;
pub use symbol::Symbol;
pub use wire::*;
""")
    enum_sym = """    /// The exchange's event enum (one variant per market) and this market's selector in it.
    type Ev: 'static;
    type Sel: dataflow_flat::Select<Self::Ev, Out = crate::wire::Body>;
""" if ENUM else ""
    if ON:
        # `On<S>` needs dataflow_flat::Symbol (KEY, NAME); ours adds the exchange index.
        write(f"{lib}/src/symbol.rs", """/// A market: `dataflow_flat::Symbol` (dense per-exchange key and name) plus its exchange.
pub trait Symbol: dataflow_flat::Symbol {
    /// 0 = Lighter, 1 = Binance, 2 = Hyperliquid.
    const EXCH: usize;
}
""")
    else:
      write(f"{lib}/src/symbol.rs", f"""/// A market: a type with a dense per-exchange key and a name.
pub trait Symbol: 'static {{
    const KEY: u32;
    const NAME: &'static str;
    /// 0 = Lighter, 1 = Binance, 2 = Hyperliquid.
    const EXCH: usize;
{enum_sym}}}
""")
    # catalog
    cat = ["//! Generated market catalog: one type per market."]
    for e, (mod, _, _) in enumerate(EX):
        cat.append(f"pub mod {mod};")
        lines = [f"//! {mod} markets ({M.n[e]}).", "use crate::Symbol;",
                 f"pub const COUNT: u32 = {M.n[e]};"]
        if ENUM:
            lines += ["use crate::wire::Body;", "use dataflow_flat::Demux;",
                      "/// One variant per market: demuxing on the market is a `match` on this enum.",
                      "#[derive(Clone, Copy, Debug, PartialEq, Demux)]",
                      "pub enum Ev { " + ", ".join(f"M{k}(Body)" for k in range(M.n[e])) + " }",
                      "impl Default for Ev { fn default() -> Self { Ev::M0(Body::default()) } }",
                      "pub fn make_ev(key: u32, body: Body) -> Ev {", "    match key {"]
            lines += [f"        {k} => Ev::M{k}(body)," for k in range(M.n[e])]
            lines += ["        _ => panic!(\"no such market\"),", "    }", "}"]
            lines += ["pub fn key_of(e: &Ev) -> u32 {", "    match e {"]
            lines += [f"        Ev::M{k}(_) => {k}," for k in range(M.n[e])]
            lines += ["    }", "}"]
        for k in range(M.n[e]):
            extra = f" type Ev = Ev; type Sel = ev::M{k};" if ENUM else ""
            if ON:
                lines.append(
                    f'pub struct M{k};\nimpl dataflow_flat::Symbol for M{k} {{ const KEY: u32 = {k}; const NAME: &\'static str = "{mname(e, k)}"; }}\nimpl Symbol for M{k} {{ const EXCH: usize = {e}; }}')
                continue
            lines.append(
                f'pub struct M{k};\nimpl Symbol for M{k} {{ const KEY: u32 = {k}; const NAME: &\'static str = "{mname(e, k)}"; const EXCH: usize = {e};{extra} }}')
        write(f"{lib}/src/catalog/{mod}.rs", "\n".join(lines) + "\n")
    write(f"{lib}/src/catalog/mod.rs", "\n".join(cat) + "\n")

    derive_wire = "#[derive(Clone, Copy, Debug, PartialEq, Root, Demux)]" if SEL else "#[derive(Clone, Copy, Debug, PartialEq, Root)]"
    wire_variants = (", ".join(f"{v}(crate::catalog::{m}::Ev)" for m, v, _ in EX) if ENUM
                     else "Lighter(Frame), Binance(Frame), Hyper(Frame)")
    frame_def = "" if ENUM else """/// One message of one exchange: the market key and the payload.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Frame { pub key: u32, pub body: Body }
"""
    derive_body = "#[derive(Clone, Copy, Debug, PartialEq, Demux)]" if SEL else "#[derive(Clone, Copy, Debug, PartialEq)]"
    keyed = """
impl Keyed for Frame {
    type Item = Body;
    fn key(&self) -> Option<u32> { Some(self.key) }
    fn item(&self) -> &Body { &self.body }
}
""" if ON else ""
    write(f"{lib}/src/wire.rs", f"""//! What arrives on the NIC.
use dataflow_flat::Root;
{sel_use}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DepthUpd {{ pub seq: u32, pub px: u64, pub qty: u64, pub bid: bool }}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TradeUpd {{ pub seq: u32, pub px: u64, pub qty: u64 }}

{derive_body}
pub enum Body {{ Depth(DepthUpd), Trade(TradeUpd) }}
impl Default for Body {{ fn default() -> Self {{ Body::Depth(DepthUpd::default()) }} }}

{frame_def}{keyed}
/// The root: one packet from one of the three exchanges.
{derive_wire}
pub enum Wire {{ {wire_variants} }}
""")
    write(f"{lib}/src/stats.rs", """//! Observable results: components write checksums here so the framework and the baseline can be compared.
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
pub static QCHK: [AtomicU64; 3] = [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)];
pub static SENT: [AtomicU64; 3] = [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)];
/// `on` mode: how many keys each exchange's listener was asked to subscribe to, and a digest of them.
pub static SUBS: [AtomicU64; 3] = [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)];
pub static SUBKEYS: [AtomicU64; 3] = [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)];
/// `req` mode: the listener requests; counts the distinct selectors read and digests their names.
#[derive(Default)]
pub struct Reqs { pub n: u64, pub digest: u64 }
impl<S: 'static> dataflow_flat::Request<S> for Reqs {
    fn add(&mut self) {
        self.n += 1;
        self.digest = self.digest.wrapping_mul(31).wrapping_add(std::any::type_name::<S>().len() as u64);
    }
}
pub static RISK: AtomicU64 = AtomicU64::new(0);
pub static BIG: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn xor(a: &AtomicU64, v: u64) { a.store(a.load(Relaxed) ^ v, Relaxed) }
pub fn reset() {
    for a in QCHK.iter().chain(SENT.iter()) { a.store(0, Relaxed) }
    RISK.store(0, Relaxed);
    BIG.store(0, Relaxed);
}
pub fn snapshot() -> [u64; 8] {
    [QCHK[0].load(Relaxed), QCHK[1].load(Relaxed), QCHK[2].load(Relaxed),
     SENT[0].load(Relaxed), SENT[1].load(Relaxed), SENT[2].load(Relaxed),
     RISK.load(Relaxed), BIG.load(Relaxed)]
}
""")
    if ENUM:
        PUSH = """        v.push(match e {
            0 => Wire::Lighter(crate::catalog::lighter::make_ev(key, body)),
            1 => Wire::Binance(crate::catalog::binance::make_ev(key, body)),
            _ => Wire::Hyper(crate::catalog::hyper::make_ev(key, body)),
        });"""
    else:
        PUSH = """        let f = Frame { key, body };
        v.push(match e { 0 => Wire::Lighter(f), 1 => Wire::Binance(f), _ => Wire::Hyper(f) });"""
    USE = ""
    write(f"{lib}/src/feed_gen.rs", """//! Deterministic packet stream: Lighter 15%, Binance 60%, Hyperliquid 25%; keys skewed to low numbers
//! (a product of two uniforms); 75% depth updates, 25% trades.
use crate::wire::*;

pub fn packets(n: usize, counts: [u32; 3], seed: u64) -> Vec<Wire> {
    let mut s = seed | 1;
    let mut next = move || { s ^= s << 13; s ^= s >> 7; s ^= s << 17; s };
    let mut v = Vec::with_capacity(n);
    for i in 0..n {
        let r = next();
        let e = match r % 100 { 0..=14 => 0, 15..=74 => 1, _ => 2 };
        let c = counts[e] as u64;
        let key = ((next() % c) * (next() % c) / c) as u32;
        let seq = i as u32 + 1;
        let body = if next() % 100 < 75 {
            Body::Depth(DepthUpd { seq, px: 1000 + next() % 997, qty: 1 + next() % 50, bid: next() & 1 == 0 })
        } else {
            Body::Trade(TradeUpd { seq, px: 1000 + next() % 997, qty: 1 + next() % 50 })
        };
@PUSH@
    }
    v
}
""".replace("@PUSH@", PUSH).replace("@USE@", USE))
    # cores: non-generic, compiled once in the library. All formulas are homogeneous (zero in, zero out) and
    # every state change is guarded by a version number from the input, so that running a component again
    # on unchanged input changes nothing (this is what makes `1a` mode agree with the cone-only baseline).
    write(f"{lib}/src/cores.rs", f"""//! Non-generic cores: the actual work of the components, compiled once in this crate.
//! The generic components in `components.rs` are thin wrappers around these.
use crate::stats;
use crate::wire::*;

#[derive(Clone, Copy, Debug, Default, PartialEq)] pub struct BookTop {{ pub bid: u64, pub ask: u64, pub bq: u64, pub aq: u64 }}
#[derive(Clone, Copy, Debug, Default, PartialEq)] pub struct TradeTop {{ pub px: u64, pub qty: u64, pub vol: u64, pub n: u32 }}
#[derive(Clone, Copy, Debug, Default, PartialEq)] pub struct FairOut {{ pub px: u64, pub conf: u64 }}
#[derive(Clone, Copy, Debug, Default, PartialEq)] pub struct Quote {{ pub bid: u64, pub ask: u64 }}
#[derive(Clone, Copy, Debug, Default, PartialEq)] pub struct XOut {{ pub px: u64, pub spread: u64 }}

#[derive(Default)]
pub struct BookCore {{ top: BookTop, ladder: [u64; 8], last_seq: u32 }}
impl BookCore {{
    {inl}
    pub fn apply(&mut self, d: &DepthUpd) -> BookTop {{
        if d.seq != self.last_seq {{
            self.last_seq = d.seq;
            let i = (d.px as usize) & 7;
            self.ladder[i] = self.ladder[i].wrapping_add(d.qty);
            let mut t = self.top;
            if d.bid {{ t.bid = d.px; t.bq = d.qty }} else {{ t.ask = d.px; t.aq = d.qty }}
            t.bq = t.bq.wrapping_add(self.ladder[(i + 1) & 7] >> 3);
            self.top = t;
            return t;
        }}
        self.top
    }}
    {inl}
    pub fn top(&self) -> BookTop {{ self.top }}
}}

#[derive(Default)]
pub struct TradeCore {{ top: TradeTop, last_seq: u32 }}
impl TradeCore {{
    {inl}
    pub fn apply(&mut self, t: &TradeUpd) -> TradeTop {{
        if t.seq != self.last_seq {{
            self.last_seq = t.seq;
            let r = TradeTop {{ px: t.px, qty: t.qty, vol: self.top.vol.wrapping_add(t.qty), n: self.top.n + 1 }};
            self.top = r;
            return r;
        }}
        self.top
    }}
    {inl}
    pub fn top(&self) -> TradeTop {{ self.top }}
}}

{inl}
pub fn mid(b: &BookTop) -> u64 {{ b.bid.wrapping_add(b.ask) / 2 }}
{inl}
pub fn imbalance(b: &BookTop) -> i64 {{ (b.bq as i64).wrapping_sub(b.aq as i64).wrapping_mul(1000) / (b.bq.wrapping_add(b.aq).wrapping_add(1) as i64) }}
{inl}
pub fn spread(b: &BookTop) -> u64 {{ b.ask.wrapping_sub(b.bid) }}
{inl}
pub fn momentum(mid: u64, vwap: u64) -> i64 {{ (mid as i64).wrapping_sub(vwap as i64) }}

#[derive(Default)]
pub struct VwapCore {{ num: u64, den: u64, last_n: u32 }}
impl VwapCore {{
    {inl}
    pub fn step(&mut self, t: &TradeTop, mid: u64) -> u64 {{
        if t.n != self.last_n {{
            self.last_n = t.n;
            self.num = self.num.wrapping_add(t.px.wrapping_mul(t.qty));
            self.den = self.den.wrapping_add(t.qty);
        }}
        if self.den == 0 {{ mid }} else {{ (self.num / self.den).wrapping_add(mid) / 2 }}
    }}
}}

#[derive(Default)]
pub struct VolCore {{ ewma: u64, last_px: u64, last_n: u32 }}
impl VolCore {{
    {inl}
    pub fn step(&mut self, t: &TradeTop) -> u64 {{
        if t.n != self.last_n {{
            self.last_n = t.n;
            let d = t.px.abs_diff(self.last_px);
            self.last_px = t.px;
            self.ewma = (self.ewma.wrapping_mul(7).wrapping_add(d)) / 8;
        }}
        self.ewma
    }}
}}

{inl}
pub fn fair(mid: u64, imb: i64, spr: u64, vwap: u64, vol: u64, mom: i64) -> FairOut {{
    let adj = imb.wrapping_mul(spr as i64) / 2000;
    FairOut {{
        px: mid.wrapping_add(vwap).wrapping_add(adj as u64).wrapping_add(mom as u64 >> 4),
        conf: vol.wrapping_add(spr).wrapping_mul(3),
    }}
}}

#[derive(Default)]
pub struct QuoterCore {{ last: Quote }}
impl QuoterCore {{
    /// `id` identifies the market in the checksum; the checksum changes only when the quote does.
    {inl}
    pub fn step(&mut self, id: u64, exch: usize, f: &FairOut, x: Option<&XOut>) -> Quote {{
        let half = f.conf / 4;
        let skew = match x {{ Some(x) => x.px.wrapping_sub(f.px) >> 6, None => 0 }};
        let q = Quote {{ bid: f.px.wrapping_sub(half).wrapping_add(skew), ask: f.px.wrapping_add(half).wrapping_add(skew) }};
        if q != self.last {{
            self.last = q;
            stats::xor(&stats::QCHK[exch], (q.bid ^ q.ask.rotate_left(17)).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ id);
        }}
        q
    }}
}}

{inl}
pub fn cross2(a: &FairOut, b: &FairOut) -> XOut {{
    XOut {{ px: a.px.wrapping_add(b.px) / 2, spread: a.px.abs_diff(b.px) }}
}}
{inl}
pub fn cross3(a: &FairOut, b: &FairOut, c: &FairOut) -> XOut {{
    XOut {{ px: a.px.wrapping_add(b.px).wrapping_add(c.px) / 3,
           spread: a.px.max(b.px).max(c.px) - a.px.min(b.px).min(c.px) }}
}}

/// Shared by the order senders, the risk sink and the big component: "something I read changed" is
/// detected by comparing a digest with the last one.
#[derive(Default)]
pub struct DigestCore {{ last: u64, count: u64 }}
impl DigestCore {{
    {inl}
    pub fn step(&mut self, digest: u64, out: &std::sync::atomic::AtomicU64) {{
        if digest != self.last {{
            self.last = digest;
            self.count += 1;
            out.store(self.count ^ digest, std::sync::atomic::Ordering::Relaxed);
        }}
    }}
}}
""")

    # components
    if ENUM:
        book_sub, trade_sub = "u: W => <S as Symbol>::Sel, body::Depth", "u: W => <S as Symbol>::Sel, body::Trade"
        book_upd = trade_upd = "self.core.apply(i.u)"
        ws_use = "use crate::wire::{body, wire};\n"
        lsub = lambda v: f"w: Wire => wire::{v}"
        feed_def = """/// A listener component: a node whose output is the exchange's event enum, the one `S` selects from.
pub trait Feed<S: Symbol>: Node<Output = <S as Symbol>::Ev> {}
impl<S: Symbol, W: Node<Output = <S as Symbol>::Ev>> Feed<S> for W {}
"""
    elif ON:
        book_sub, trade_sub = "u: W => On<S>, body::Depth", "u: W => On<S>, body::Trade"
        book_upd = trade_upd = "self.core.apply(i.u)"
        ws_use = "use dataflow_flat::On;\nuse crate::wire::{body, wire};\n"
        lsub = lambda v: f"w: Wire => wire::{v}"
        feed_def = "/// A listener component: a node whose output is the exchange's latest frame.\npub trait Feed<S>: Node<Output = Frame> {}\n"
    else:
        book_sub, trade_sub = "u: W", "u: W"
        book_upd = "match &i.u.body { Body::Depth(d) if i.u.key == S::KEY => self.core.apply(d), _ => self.core.top() }"
        trade_upd = "match &i.u.body { Body::Trade(t) if i.u.key == S::KEY => self.core.apply(t), _ => self.core.top() }"
        ws_use = ""
        lsub = lambda v: "w: Wire"
        feed_def = "/// A listener component: a node whose output is the exchange's latest frame.\npub trait Feed<S>: Node<Output = Frame> {}\n"
    comp = [f"""//! Thin generic components over the non-generic cores (the recommended pattern).
use std::marker::PhantomData;
use dataflow_flat::{{Component, ComponentSubscription, Node}};
use crate::cores::*;
use crate::symbol::Symbol;
use crate::wire::*;
{ws_use}
{feed_def}"""]
    for e, (mod, var, ws) in enumerate(EX):
        if ENUM:
            evt = f"crate::catalog::{mod}::Ev"
            lattr = "\n#[listener]" if REQ else ""
            limpl = f"impl dataflow_flat::Listener for {ws} {{ type Requests = crate::stats::Reqs; }}\n" if REQ else ""
            newbody = (f"{{ crate::stats::SUBS[{e}].store(i.requests.n, std::sync::atomic::Ordering::Relaxed); crate::stats::SUBKEYS[{e}].store(i.requests.digest, std::sync::atomic::Ordering::Relaxed); ({ws}, {evt}::default()) }}"
                       if REQ else f"{{ ({ws}, {evt}::default()) }}")
            comp.append(f"""#[derive(ComponentSubscription)]
#[subscribe({lsub(var)})]{lattr}
pub struct {ws};
{limpl}impl Component for {ws} {{
    type Config = ();
    type Output = {evt};
    fn new((): (), {"i" if REQ else "_"}: Self::Init<'_>) -> (Self, {evt}) {newbody}
    fn update(&mut self, i: Self::Input<'_>) -> {evt} {{ *i.w }}
}}
""")
            continue
        if SEL:
            upd = "*i.w"
        else:
            upd = f"if let Wire::{var}(f) = i.w {{ self.last = *f }}\n        self.last"
        listen = ""
        if ON:
            listen = f"""
impl dataflow_flat::KeyListener for {ws} {{
    /// Records the keys the graph reads from this exchange (a real listener opens one combined subscription).
    fn subscribe(&mut self, keys: &[u32]) {{
        self.keys = keys.to_vec();
        crate::stats::SUBS[{e}].store(keys.len() as u64, std::sync::atomic::Ordering::Relaxed);
        crate::stats::SUBKEYS[{e}].store(keys.iter().fold(0u64, |a, k| a.wrapping_mul(31).wrapping_add(*k as u64 + 1)), std::sync::atomic::Ordering::Relaxed);
    }}
}}
"""
        comp.append(f"""#[derive(ComponentSubscription)]
#[subscribe({lsub(var)})]{"" if not ON else chr(10) + "#[key_listener]"}
pub struct {ws} {{ last: Frame{", keys: Vec<u32>" if ON else ""} }}{listen}
impl Component for {ws} {{
    type Config = ();
    type Output = Frame;
    fn new((): (), _: Self::Init<'_>) -> (Self, Frame) {{ ({ws} {{ last: Frame::default(){', keys: Vec::new()' if ON else ''} }}, Frame::default()) }}
    fn update(&mut self, i: Self::Input<'_>) -> Frame {{ {upd} }}
}}
impl<S> Feed<S> for {ws} {{}}
""")

    def generic(name, subs, out_t, core_t, new_out, body, extra_bounds="", gen="W: Feed<S>, S: Symbol", state="core", core_init=None):
        core_init = core_init or f"{core_t}::default()"
        wc = f"<{gen}>"
        names = ", ".join(g.split(":")[0].strip() for g in gen.split(","))
        return f"""#[derive(ComponentSubscription)]
#[subscribe({subs})]
pub struct {name}{wc} {{ {state}: {core_t}, _p: PhantomData<fn() -> ({names})> }}
impl{wc} Component for {name}<{names}> {extra_bounds} {{
    type Config = ();
    type Output = {out_t};
    fn new((): (), _: Self::Init<'_>) -> (Self, {out_t}) {{ ({name} {{ {state}: {core_init}, _p: PhantomData }}, {new_out}) }}
    fn update(&mut self, i: Self::Input<'_>) -> {out_t} {{ {body} }}
}}
"""
    comp.append(generic("Book", book_sub, "BookTop", "BookCore", "BookTop::default()", book_upd))
    comp.append(generic("Trades", trade_sub, "TradeTop", "TradeCore", "TradeTop::default()", trade_upd))
    comp.append(generic("Mid", "b: Book<W, S>", "u64", "()", "0", "mid(i.b)", core_init="()"))
    comp.append(generic("Imbalance", "b: Book<W, S>", "i64", "()", "0", "imbalance(i.b)", core_init="()"))
    comp.append(generic("Spread", "b: Book<W, S>", "u64", "()", "0", "spread(i.b)", core_init="()"))
    comp.append(generic("Vwap", "t: Trades<W, S>, m: Mid<W, S>", "u64", "VwapCore", "0", "self.core.step(i.t, *i.m)"))
    comp.append(generic("Vol", "t: Trades<W, S>", "u64", "VolCore", "0", "self.core.step(i.t)"))
    comp.append(generic("Momentum", "m: Mid<W, S>, v: Vwap<W, S>", "i64", "()", "0", "momentum(*i.m, *i.v)", core_init="()"))
    comp.append(generic("Fair", "m: Mid<W, S>, imb: Imbalance<W, S>, sp: Spread<W, S>, v: Vwap<W, S>, vol: Vol<W, S>, mom: Momentum<W, S>",
                        "FairOut", "()", "FairOut::default()", "fair(*i.m, *i.imb, *i.sp, *i.v, *i.vol, *i.mom)", core_init="()"))
    comp.append(generic("Quoter", "f: Fair<W, S>", "Quote", "QuoterCore", "Quote::default()",
                        "self.core.step(((S::EXCH as u64) << 32) | S::KEY as u64, S::EXCH, i.f, None)"))
    comp.append(generic("QuoterX", "f: Fair<W, S>, x: X", "Quote", "QuoterCore", "Quote::default()",
                        "self.core.step(((S::EXCH as u64) << 32) | S::KEY as u64, S::EXCH, i.f, Some(i.x))",
                        gen="W: Feed<S>, S: Symbol, X: Node<Output = XOut>"))
    comp.append(generic("Cross2", "a: A, b: B", "XOut", "()", "XOut::default()", "cross2(i.a, i.b)",
                        gen="A: Node<Output = FairOut>, B: Node<Output = FairOut>", core_init="()"))
    comp.append(generic("Cross3", "a: A, b: B, c: C", "XOut", "()", "XOut::default()", "cross3(i.a, i.b, i.c)",
                        gen="A: Node<Output = FairOut>, B: Node<Output = FairOut>, C: Node<Output = FairOut>", core_init="()"))
    write(f"{lib}/src/components.rs", "\n".join(comp))


# ---------------------------------------------------------------------------------------------
# app
# ---------------------------------------------------------------------------------------------
def sender_sum(M, e, acc):
    """The digest of an order sender: sampled quoter outputs. acc(k) -> expression for the Quote of (e, k)."""
    idx = M.sample(M.quoters[e])
    return " ^ ".join(f"{acc(k)}.bid ^ {acc(k)}.ask.rotate_left({(j % 13) + 1})" for j, k in enumerate(idx))


def risk_sum(M, accx, accq):
    idx = M.sample(M.risk_deps)
    parts = []
    for j, ix in enumerate(idx):
        d = M.risk_deps[ix]
        if d[0] == "x":
            parts.append(f"{accx(d[1])}.px ^ {accx(d[1])}.spread.rotate_left({(j % 13) + 1})")
        else:
            parts.append(f"{accq(d[1], d[2])}.bid ^ {accq(d[1], d[2])}.ask.rotate_left({(j % 13) + 1})")
    return " ^ ".join(parts)


def big_math(M, acc):
    """The body of the big component: several rounds of mixing over all the Fair values it reads.
    acc(j) -> expression for the FairOut of dependency j. Emitted into both the app and the baseline."""
    lines = ["let tweak: u64 = 1000; // EDIT", "let mut acc: u64 = 0;"]
    n = len(M.big_deps)
    for r in range(4):
        for j in range(n):
            lines.append(f"acc = acc.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add({acc(j)}.px ^ (acc >> {7 + r}));")
        for j in range(0, n, 3):
            lines.append(f"acc ^= {acc(j)}.conf.rotate_left({r * 5 + (j % 7) + 1});")
        lines.append(f"acc = acc.rotate_left({r + 9}) ^ (acc >> 3);")
    lines.append("let acc = acc.wrapping_mul(tweak);")
    return lines


def ALLOW(M):
    """Apps above ~1,500 components need the long-running-const-eval allow (user decision)."""
    return "#![allow(long_running_const_eval)]\n" if M.components() > 1500 else ""


def gen_app(out, M, MODE, big_inline=False):
    app = f"{out}/app"
    write(f"{app}/Cargo.toml", f"""[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
dataflow-flat = {{ path = "{LIB}/dataflow-flat" }}
marketlib = {{ path = "../marketlib" }}
""")
    mods = "\n".join(f"pub mod {m};" for m, _, _ in EX)
    write(f"{app}/src/lib.rs", f"""//! The app: wiring only. Components come from `marketlib`; this crate says which markets exist, joins
//! them across venues and defines the sinks.
{ALLOW(M)}{mods}
pub mod big;
pub mod cross;
pub mod risk;

use dataflow_flat::{{Configs, Pipeline}};
use marketlib::Wire;

pub type Sinks = (lighter::Sender, binance::Sender, hyper::Sender, risk::Risk, big::Big);
pub type Pipe = Pipeline<Wire, Sinks>;

pub fn build() -> Pipe {{ Pipeline::build(Configs::new()).unwrap() }}

/// The unit of work the benchmark times; the push is inlined into this loop.
#[inline(never)]
pub fn run_batch(p: &mut Pipe, pkts: &[Wire]) {{
    for w in pkts {{ p.push(*w) }}
}}
""")
    # cross-venue joins
    cx = ["//! Cross-venue joins: one per base asset listed on two or more venues.",
          "use marketlib::{Cross2, Cross3, Fair, LighterWs, BinanceWs, HyperWs};",
          "use marketlib::catalog::{lighter as cl, binance as cb, hyper as ch};"]
    wsn = ["LighterWs", "BinanceWs", "HyperWs"]
    cat = ["cl", "cb", "ch"]
    for i, mem in enumerate(M.cross):
        f = ", ".join(f"Fair<{wsn[e]}, {cat[e]}::M{k}>" for e, k in mem)
        cx.append(f"pub type X{i} = Cross{len(mem)}<{f}>;")
    write(f"{app}/src/cross.rs", "\n".join(cx) + "\n")
    # per-exchange modules
    for e, (mod, var, ws) in enumerate(EX):
        al = [f"//! {mod}: the markets' quoters and the order sender.",
              "#[allow(unused_imports)]", "use marketlib::{Quoter, QuoterX};", f"use marketlib::catalog::{mod} as cat;",
              f"pub use marketlib::{ws} as Ws;", "pub mod sender;", "pub use sender::Sender;"]
        for k in range(M.n[e]):
            if (e, k) in M.cross_of:
                al.append(f"pub type Q{k} = QuoterX<Ws, cat::M{k}, crate::cross::X{M.cross_of[(e, k)]}>;")
            else:
                al.append(f"pub type Q{k} = Quoter<Ws, cat::M{k}>;")
        write(f"{app}/src/{mod}/mod.rs", "\n".join(al) + "\n")
        subs = ",\n    ".join(f"q{k}: crate::{mod}::Q{k}" for k in range(M.n[e]))
        digest = sender_sum(M, e, lambda k: f"i.q{k}")
        write(f"{app}/src/{mod}/sender.rs", f"""//! Order sender for {mod}: subscribes to every quoter of the exchange (wide).
use dataflow_flat::{{Component, ComponentSubscription}};
use marketlib::DigestCore;
use marketlib::stats::SENT;

#[derive(ComponentSubscription)]
#[subscribe(
    {subs}
)]
pub struct Sender {{ core: DigestCore }}

impl Component for Sender {{
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {{ (Sender {{ core: DigestCore::default() }}, ()) }}
    fn update(&mut self, i: Self::Input<'_>) {{
        let tweak: u64 = 1000; // EDIT
        let digest = ({digest}).wrapping_mul(tweak);
        self.core.step(digest, &SENT[{e}]);
    }}
}}
""")
    # risk
    rsubs = []
    for ix, d in enumerate(M.risk_deps):
        rsubs.append(f"x{ix}: crate::cross::X{d[1]}" if d[0] == "x" else f"x{ix}: crate::{EX[d[1]][0]}::Q{d[2]}")
    rsum = risk_sum(M, lambda i: f"i.x{i}", lambda e, k: f"i.x{M.risk_deps.index(('q', e, k))}")
    write(f"{app}/src/risk.rs", f"""//! Risk sink: reads every cross-venue join.
use dataflow_flat::{{Component, ComponentSubscription}};
use marketlib::DigestCore;
use marketlib::stats::RISK;

#[derive(ComponentSubscription)]
#[subscribe(
    {(",\n    ").join(rsubs)}
)]
pub struct Risk {{ core: DigestCore }}

impl Component for Risk {{
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {{ (Risk {{ core: DigestCore::default() }}, ()) }}
    fn update(&mut self, i: Self::Input<'_>) {{
        let tweak: u64 = 1000; // EDIT
        let digest = ({rsum}).wrapping_mul(tweak);
        self.core.step(digest, &RISK);
    }}
}}
""")
    # big
    bsubs = ",\n    ".join(f"f{j}: marketlib::Fair<marketlib::{EX[e][2]}, marketlib::catalog::{EX[e][0]}::M{k}>"
                           for j, (e, k) in enumerate(M.big_deps))
    body = "\n        ".join(big_math(M, lambda j: f"i.f{j}"))
    big_attr = "#[inline(always)]" if big_inline else ""
    write(f"{app}/src/big.rs", f"""//! The big component: wide, with a long body. Lives alone in its file.
use dataflow_flat::{{Component, ComponentSubscription}};
use marketlib::DigestCore;
use marketlib::stats::BIG;

#[derive(ComponentSubscription)]
#[subscribe(
    {bsubs}
)]
pub struct Big {{ core: DigestCore }}

impl Component for Big {{
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {{ (Big {{ core: DigestCore::default() }}, ()) }}
    {big_attr}
    fn update(&mut self, i: Self::Input<'_>) {{
        {body}
        self.core.step(acc, &BIG);
    }}
}}
""")


# ---------------------------------------------------------------------------------------------
# baseline: hand-written nested match
# ---------------------------------------------------------------------------------------------
def gen_baseline(out, M, MODE, core_inline):
    ENUM = MODE == "enum"
    b = f"{out}/baseline"
    write(f"{b}/Cargo.toml", """[package]
name = "baseline"
version = "0.1.0"
edition = "2021"

[dependencies]
marketlib = { path = "../marketlib" }
""")
    ns = M.n
    write(f"{b}/src/lib.rs", """//! Hand-written equivalent: stored outputs in struct fields that persist across packets, a nested `match`
//! on exchange / key / kind, and only the matching cone updated. `run_all_batch` is the every-component-
//! runs-on-every-packet version (what phase-1a master does).
mod arms;
mod hand;
mod all;
pub use hand::Hand;
pub use arms::run_batch;
pub use all::run_all_batch;
""")
    sample = lambda e: sender_sum(M, e, lambda k: f"h.m[{e}][{k}].quote")
    accx = lambda i: f"h.x[{i}]"
    accq = lambda e, k: f"h.m[{e}][{k}].quote"
    big_lines = big_math(M, lambda j: f"h.m[{M.big_deps[j][0]}][{M.big_deps[j][1]}].fair")
    xof = lambda e: "vec![" + ", ".join(f"Some({M.cross_of[(e, k)]})" if (e, k) in M.cross_of else "None" for k in range(M.n[e])) + "]"
    all_fn = "" if ENUM else """    /// Everything, as phase-1a master would run it (the guards make the stale parts no-ops).
    #[inline(always)]
    pub fn all(&mut self, key: u32, f: &Frame) {
        let top = match &f.body { Body::Depth(d) if f.key == key => self.book.apply(d), _ => self.book.top() };
        let tt = match &f.body { Body::Trade(t) if f.key == key => self.trades.apply(t), _ => self.trades.top() };
        self.bt = top; self.tt = tt;
        self.mid = mid(&self.bt);
        self.imb = imbalance(&self.bt);
        self.spr = spread(&self.bt);
        self.vw = self.vwap.step(&self.tt, self.mid);
        self.vl = self.vol.step(&self.tt);
        self.mom = momentum(self.mid, self.vw);
        self.fair = fair(self.mid, self.imb, self.spr, self.vw, self.vl, self.mom);
    }"""
    last_field = "" if ENUM else "    pub last: [Frame; 3],\n"
    last_init = "" if ENUM else "            last: [Frame::default(); 3],\n"
    write(f"{b}/src/hand.rs", f"""//! State of the hand-written baseline.
use marketlib::*;
use marketlib::stats::*;

#[derive(Default)]
pub struct Mkt {{
    pub book: BookCore, pub trades: TradeCore, pub vwap: VwapCore, pub vol: VolCore, pub quoter: QuoterCore,
    pub bt: BookTop, pub tt: TradeTop, pub mid: u64, pub imb: i64, pub spr: u64, pub vw: u64, pub vl: u64,
    pub mom: i64, pub fair: FairOut, pub quote: Quote,
}}
impl Mkt {{
    #[inline(always)]
    pub fn on_depth(&mut self, d: &DepthUpd) {{
        self.bt = self.book.apply(d);
        self.mid = mid(&self.bt);
        self.imb = imbalance(&self.bt);
        self.spr = spread(&self.bt);
        self.vw = self.vwap.step(&self.tt, self.mid);
        self.mom = momentum(self.mid, self.vw);
        self.fair = fair(self.mid, self.imb, self.spr, self.vw, self.vl, self.mom);
    }}
    #[inline(always)]
    pub fn on_trade(&mut self, t: &TradeUpd) {{
        self.tt = self.trades.apply(t);
        self.vw = self.vwap.step(&self.tt, self.mid);
        self.vl = self.vol.step(&self.tt);
        self.mom = momentum(self.mid, self.vw);
        self.fair = fair(self.mid, self.imb, self.spr, self.vw, self.vl, self.mom);
    }}
    #[inline(always)]
    pub fn requote(&mut self, id: u64, exch: usize, x: Option<&XOut>) {{
        self.quote = self.quoter.step(id, exch, &self.fair, x);
    }}
{all_fn}
}}
pub struct Hand {{
    pub m: [Vec<Mkt>; 3],
    pub x: Vec<XOut>,
    pub xof: [Vec<Option<usize>>; 3],
{last_field}    pub send: [DigestCore; 3],
    pub risk: DigestCore,
    pub big: DigestCore,
}}
impl Hand {{
    pub fn new() -> Self {{
        let mk = |n: u32| (0..n).map(|_| Mkt::default()).collect::<Vec<_>>();
        Hand {{
            m: [mk({ns[0]}), mk({ns[1]}), mk({ns[2]})],
            x: vec![XOut::default(); {len(M.cross)}],
            xof: [{xof(0)}, {xof(1)}, {xof(2)}],
{last_init}            send: Default::default(),
            risk: DigestCore::default(),
            big: DigestCore::default(),
        }}
    }}
}}
impl Default for Hand {{ fn default() -> Self {{ Self::new() }} }}

#[inline(always)]
pub fn sender_digest(h: &Hand, e: usize) -> u64 {{
    match e {{
{chr(10).join(f"        {e} => {sample(e)}," for e in range(3))}
        _ => 0,
    }}
}}
#[inline(always)]
pub fn risk_digest(h: &Hand) -> u64 {{ {risk_sum(M, accx, accq)} }}
#[inline(always)]
pub fn big_digest(h: &Hand) -> u64 {{
    {(chr(10) + "    ").join(big_lines)}
    acc
}}
#[inline(always)]
pub fn step_sender(h: &mut Hand, e: usize) {{ let d = sender_digest(h, e).wrapping_mul(1000); h.send[e].step(d, &SENT[e]) }}
#[inline(always)]
pub fn step_risk(h: &mut Hand) {{ let d = risk_digest(h).wrapping_mul(1000); h.risk.step(d, &RISK) }}
#[inline(always)]
pub fn step_big(h: &mut Hand) {{ let d = big_digest(h); h.big.step(d, &BIG) }}
""")
    # cone arms
    arms = ["use crate::hand::*;", "use marketlib::*;", ""]
    big_set = set(M.big_deps)
    risk_x = {d[1] for d in M.risk_deps if d[0] == "x"}
    risk_q = {(d[1], d[2]) for d in M.risk_deps if d[0] == "q"}
    for e in range(3):
        for k in range(ns[e]):
            lines = [f"#[inline(always)]\nfn m_{e}_{k}(h: &mut Hand, body: &Body) {{",
                     f"    match body {{ Body::Depth(d) => h.m[{e}][{k}].on_depth(d), Body::Trade(t) => h.m[{e}][{k}].on_trade(t) }}"]
            ran_q, fire_risk, senders = [], False, set()
            if (e, k) in M.cross_of:
                xi = M.cross_of[(e, k)]
                mem = M.cross[xi]
                args = ", ".join(f"&h.m[{ee}][{kk}].fair" for ee, kk in mem)
                lines.append(f"    h.x[{xi}] = cross{len(mem)}({args});")
                lines.append(f"    let x = h.x[{xi}];")
                for ee, kk in mem:
                    lines.append(f"    h.m[{ee}][{kk}].requote({(ee << 32) | kk}, {ee}, Some(&x));")
                    ran_q.append((ee, kk))
                fire_risk = xi in risk_x
            else:
                lines.append(f"    h.m[{e}][{k}].requote({(e << 32) | k}, {e}, None);")
                ran_q.append((e, k))
            for q in ran_q:
                senders.add(q[0])
                if q in risk_q:
                    fire_risk = True
            for s in sorted(senders):
                lines.append(f"    step_sender(h, {s});")
            if fire_risk:
                lines.append("    step_risk(h);")
            if (e, k) in big_set:
                lines.append("    step_big(h);")
            lines.append("}")
            arms.append("\n".join(lines))
    # nested match
    run = ["#[inline(never)]", "pub fn run_batch(h: &mut Hand, pkts: &[Wire]) {", "    for w in pkts {", "        match w {"]
    for e, (mod, var, _) in enumerate(EX):
        if ENUM:
            run.append(f"            Wire::{var}(ev) => match ev {{")
            for k in range(ns[e]):
                run.append(f"                marketlib::catalog::{mod}::Ev::M{k}(b) => m_{e}_{k}(h, b),")
            run.append("            },")
        else:
            # BASE_COPY=1 (experiment): dispatch on a copy of the frame that went through memory, as the framework does
            cp = bool(os.environ.get("BASE_COPY"))
            run.append(f"            Wire::{var}(f0) => {{ let f = {'std::hint::black_box(*f0)' if cp else '*f0'}; match f.key {{")
            for k in range(ns[e]):
                run.append(f"                {k} => m_{e}_{k}(h, &f.body),")
            run.append("                _ => {}")
            run.append("            } }")
    run += ["        }", "    }", "}"]
    write(f"{b}/src/arms.rs", "\n".join(arms) + "\n" + "\n".join(run) + "\n")
    # run_all (phase 1a semantics: every component, every packet)
    if ENUM:
        write(f"{b}/src/all.rs", "use crate::hand::*;\nuse marketlib::*;\npub fn run_all_batch(_: &mut Hand, _: &[Wire]) { unimplemented!(\"1a mode only\") }\n")
        return
    cross_tbl = ",\n    ".join("&[" + ", ".join(f"({e}, {k})" for e, k in mem) + "]" for mem in M.cross)
    write(f"{b}/src/all.rs", f"""use crate::hand::*;
use marketlib::*;

static CROSS: [&[(usize, usize)]; {len(M.cross)}] = [
    {cross_tbl}
];
static COUNTS: [usize; 3] = [{ns[0]}, {ns[1]}, {ns[2]}];

/// Every component on every packet, in dependency order.
#[inline(never)]
pub fn run_all_batch(h: &mut Hand, pkts: &[Wire]) {{
    for w in pkts {{
        match w {{
            Wire::Lighter(f) => h.last[0] = *f,
            Wire::Binance(f) => h.last[1] = *f,
            Wire::Hyper(f) => h.last[2] = *f,
        }}
        for e in 0..3 {{
            let f = h.last[e];
            for k in 0..COUNTS[e] {{ h.m[e][k].all(k as u32, &f) }}
        }}
        for (i, mem) in CROSS.iter().enumerate() {{
            h.x[i] = match mem.len() {{
                2 => cross2(&h.m[mem[0].0][mem[0].1].fair, &h.m[mem[1].0][mem[1].1].fair),
                _ => cross3(&h.m[mem[0].0][mem[0].1].fair, &h.m[mem[1].0][mem[1].1].fair, &h.m[mem[2].0][mem[2].1].fair),
            }};
        }}
        for e in 0..3 {{
            for k in 0..COUNTS[e] {{
                let xo = h.xof[e][k].map(|i| h.x[i]);
                h.m[e][k].requote(((e as u64) << 32) | k as u64, e, xo.as_ref());
            }}
        }}
        for e in 0..3 {{ step_sender(h, e) }}
        step_risk(h);
        step_big(h);
    }}
}}
""")


def gen_runner(out, M, mode):
    r = f"{out}/runner"
    write(f"{r}/Cargo.toml", """[package]
name = "runner"
version = "0.1.0"
edition = "2021"

[features]
default = ["fw", "base"]
fw = ["dep:app", "dep:dataflow-flat"]
base = ["dep:baseline"]

[dependencies]
marketlib = { path = "../marketlib" }
app = { path = "../app", optional = true }
baseline = { path = "../baseline", optional = true }
dataflow-flat = { path = "%s/dataflow-flat", optional = true }
""" % LIB)
    KINDF = "" if mode == "enum" else """    // --kind E:B (E = 0 lighter, 1 binance, 2 hyper; B = d depth, t trade): time only those packets
    let pk = match std::env::args().skip_while(|x| x != "--kind").nth(1) {
        None => pk,
        Some(k) => {
            let (e, b) = k.split_once(':').unwrap();
            let (e, d) = (e.parse::<usize>().unwrap(), b == "d");
            pk.into_iter().filter(|w| {
                let (we, f) = match w { Wire::Lighter(f) => (0, f), Wire::Binance(f) => (1, f), Wire::Hyper(f) => (2, f) };
                we == e && matches!(f.body, Body::Depth(_)) == d
            }).collect::<Vec<_>>()
        }
    };
    let npk = pk.len();
    eprintln!("packets {npk}");
"""
    KEYOF = "marketlib::catalog::{}::key_of(f)" if mode == "enum" else "f.key"
    write(f"{r}/src/main.rs", f"""//! usage: runner [--packets N] [--reps R] [--rounds K] [--all] [--keys N] [--sort]
//! Verifies framework == baseline (checksums), then times them in alternating rounds and prints
//! `RESULT <side> min=<ns/packet> median=<ns/packet>` lines.
{ALLOW(M)}use marketlib::{{feed_gen, stats}};
#[allow(unused_imports)] use marketlib::wire::*;
use std::time::Instant;

const COUNTS: [u32; 3] = [{M.n[0]}, {M.n[1]}, {M.n[2]}];

fn arg(name: &str, default: usize) -> usize {{
    let a: Vec<String> = std::env::args().collect();
    a.iter().position(|x| x == name).map(|i| a[i + 1].parse().unwrap()).unwrap_or(default)
}}

fn main() {{
    let npk = arg("--packets", 1 << 16);
    let reps = arg("--reps", 4);
    let rounds = arg("--rounds", 11);
    let with_all = std::env::args().any(|x| x == "--all");
    let pk = feed_gen::packets(npk, COUNTS, 0x5EED);
{KINDF}
    // --keys N: keep only packets whose key is below N; --sort: order the packets by (exchange, key), stable,
    // so that dispatch is predictable (the sorted/unsorted experiment)
    let wkey = |w: &Wire| match w {{ Wire::Lighter(f) => (0, {KEYOF.format('lighter')}), Wire::Binance(f) => (1, {KEYOF.format('binance')}), Wire::Hyper(f) => (2, {KEYOF.format('hyper')}) }};
    let mut pk = pk;
    let keys = arg("--keys", usize::MAX);
    if keys != usize::MAX {{ pk.retain(|w| (wkey(w).1 as usize) < keys); eprintln!("packets with key < {{keys}}: {{}}", pk.len()); }}
    if std::env::args().any(|x| x == "--sort") {{ pk.sort_by_key(|w| wkey(w)); }}
    let npk = pk.len();
    #[cfg(feature = "fw")]
    let mut fw = app::build();
    #[cfg(feature = "fw")]
    println!("listener subscriptions (keys per exchange) {{:?}}", [0, 1, 2].map(|e| stats::SUBS[e].load(std::sync::atomic::Ordering::Relaxed)));
    #[cfg(feature = "base")]
    let mut hand = baseline::Hand::new();
    #[cfg(feature = "base")]
    let mut hand_all = baseline::Hand::new();

    // Verification: one pass each from a fresh state; the checksums written by the components must agree.
    #[cfg(feature = "fw")]
    let sig_fw = {{ stats::reset(); app::run_batch(&mut fw, &pk); stats::snapshot() }};
    #[cfg(feature = "base")]
    let sig_base = {{ stats::reset(); baseline::run_batch(&mut hand, &pk); stats::snapshot() }};
    #[cfg(feature = "base")]
    let sig_all = if with_all {{ stats::reset(); baseline::run_all_batch(&mut hand_all, &pk); Some(stats::snapshot()) }} else {{ None }};
    #[cfg(feature = "fw")]
    println!("signature framework {{:?}}", sig_fw);
    #[cfg(feature = "base")]
    println!("signature baseline  {{:?}}", sig_base);
    #[cfg(all(feature = "fw", feature = "base"))]
    {{
        assert_eq!(sig_fw, sig_base, "framework and cone-only baseline disagree");
        if let Some(a) = sig_all {{ assert_eq!(a, sig_base, "every-component baseline and cone-only baseline disagree") }}
        println!("VERIFIED framework == baseline");
    }}

    let mut t_fw = Vec::new();
    let mut t_base = Vec::new();
    let mut t_all = Vec::new();
    let per = (npk * reps) as f64;
    for _ in 0..rounds {{
        #[cfg(feature = "fw")]
        {{
            let t = Instant::now();
            for _ in 0..reps {{ app::run_batch(&mut fw, &pk) }}
            t_fw.push(t.elapsed().as_nanos() as f64 / per);
        }}
        #[cfg(feature = "base")]
        {{
            let t = Instant::now();
            for _ in 0..reps {{ baseline::run_batch(&mut hand, &pk) }}
            t_base.push(t.elapsed().as_nanos() as f64 / per);
            if with_all {{
                let t = Instant::now();
                for _ in 0..reps {{ baseline::run_all_batch(&mut hand_all, &pk) }}
                t_all.push(t.elapsed().as_nanos() as f64 / per);
            }}
        }}
    }}
    let report = |name: &str, mut v: Vec<f64>| {{
        if v.is_empty() {{ return }}
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("RESULT {{name}} min={{:.2}} median={{:.2}}", v[0], v[v.len() / 2]);
    }};
    report("framework", t_fw);
    report("baseline", t_base);
    report("baseline-all", t_all);
}}
""")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--size", default="tiny")
    ap.add_argument("--mode", default="1a", choices=["1a", "enum", "req", "on"],
                    help="1a: no selectors, everything runs; enum: selectors via one #[derive(Demux)] enum per exchange (works on phase 1b); on: On<S> keyed selectors per the spec (not on master yet)")
    ap.add_argument("--lib", help="path of the dataflow checkout to depend on (default: this repo)")
    ap.add_argument("--out")
    ap.add_argument("--markets", help="L,B,H overrides the size")
    ap.add_argument("--sink-scan", default="8", help="how many of its inputs each order sender / risk samples (or 'all')")
    ap.add_argument("--big-width", type=int, default=42)
    ap.add_argument("--big-inline", action="store_true", help="mark the framework Big::update #[inline(always)] (user-side hint)")
    ap.add_argument("--core-inline", default="hint", choices=["none", "hint"])
    a = ap.parse_args()
    if a.markets:
        counts = tuple(int(x) for x in a.markets.split(","))
    elif a.size in SIZES:
        counts = SIZES[a.size]
    else:  # custom: m<L>-<B>-<H>, e.g. m3-12-3
        counts = tuple(int(x) for x in a.size[1:].split("-"))
    assert len(counts) == 3 and all(c >= 1 for c in counts)
    out = os.path.abspath(a.out or f"gen/{a.size}-{a.mode}")
    global LIB, REQ
    REQ = a.mode == "req"
    if REQ:
        a.mode = "enum"
    if a.lib:
        LIB = os.path.abspath(a.lib)
    M = Model(counts, a.sink_scan, a.big_width)
    if os.path.exists(out):
        for d in ("marketlib", "app", "baseline", "runner"):
            shutil.rmtree(f"{out}/{d}", ignore_errors=True)
    write(f"{out}/Cargo.toml", """[workspace]
resolver = "2"
members = ["marketlib", "app", "baseline", "runner"]
""")
    shutil.copy(f"{LIB}/rust-toolchain.toml", f"{out}/rust-toolchain.toml")
    write(f"{out}/.cargo/config.toml", CARGO_CONFIG)
    if os.path.exists(f"{LIB}/Cargo.lock") and not os.path.exists(f"{out}/Cargo.lock"):
        shutil.copy(f"{LIB}/Cargo.lock", f"{out}/Cargo.lock")
    gen_lib(out, M, a.mode, a.core_inline)
    gen_app(out, M, a.mode, a.big_inline)
    gen_baseline(out, M, a.mode, a.core_inline)
    gen_runner(out, M, a.mode)
    print(f"{out}: mode={a.mode} markets L/B/H={M.n} components={M.components()} crosses={len(M.cross)} edges~{M.edges()}")


if __name__ == "__main__":
    main()
