#!/usr/bin/env python3
"""Generator for the enum-keys comparison: demux one listener's output by symbol.

usage: bench/gen_enum_keys.py N OUTDIR

Writes crates under OUTDIR (each its own workspace, path dependency on dataflow-flat):

  e  variant E: the listener outputs `enum FrameN { S0(Tob), .. }`; one marker per symbol (`markets::S0`) with a
     hand-written `impl Select<FrameN>` whose `key_of` is ONE shared function; `Book<S: Select<FrameN, Out = Tob>>`
     reads `S(t): Ws`
  d  variant E with `#[derive(Demux)]` on the enum as it stands (compile time only; selectors are `framen::S0`)
  h  hand-written baseline: a `match` on the symbol with the books in an array

Every side has the same 827 `Book<S>` bodies, the same handler sink `Oms` + `Probe`, and the same pre-computed packet
table. Run `BIN random|sorted [packets]`: prints ns/packet per chunk and the final digest (all sides must agree).
"""
import os
import sys

n = int(sys.argv[1])
out = sys.argv[2]
# DIFF=1: Book<S>::update (and the hand-written step) runs one of 8 structurally different operations chosen by the
# symbol's index, each with a per-symbol constant, so the arms cannot be merged into one tail.
DIFF = os.environ.get("DIFF", "0") == "1"
FN = f"Frame{n}"
MOD = f"frame{n}"

COMMON = f"""#![allow(dead_code, unused_imports, non_snake_case)]
{"#![allow(long_running_const_eval)]" if n >= 1000 else ""}
use std::hint::black_box;
use std::sync::atomic::{{AtomicU64, Ordering}};
use std::time::Instant;
use dataflow_flat::*;

pub const N: usize = {n};
pub static DIGEST: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Root)]
pub struct Packet {{ pub symbol: u32, pub bid: i64, pub ask: i64, pub seq: u64 }}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tob {{ pub bid: i64, pub ask: i64, pub seq: u64 }}

//@@SINK
/// The same sink on every side: a handler component folding every fired book into a pair, and a probe.
#[derive(ComponentSubscription)]
#[subscribe(%HANDLES%)]
pub struct Oms {{ acc: (u64, u64) }}
impl Component for Oms {{
    type Config = ();
    type Output = (u64, u64);
    fn new((): (), _: Self::Init<'_>) -> (Self, (u64, u64)) {{ (Oms {{ acc: (0, 0) }}, (0, 0)) }}
    #[inline(always)]
    fn update(&mut self, _: Self::Input<'_>) -> (u64, u64) {{ std::mem::take(&mut self.acc) }}
}}
%UPDATE_HANDLER%
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(o: Oms)]
pub struct Probe {{ d: u64 }}
impl Component for Probe {{
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {{ (Probe {{ d: 0 }}, ()) }}
    fn update(&mut self, i: Self::Input<'_>) {{
        self.d = self.d.rotate_left(1) ^ i.o.0.wrapping_add(i.o.1);
        DIGEST.store(self.d, Ordering::Relaxed);
    }}
}}

//@@FOLD
#[inline(always)]
fn fold(buf: &mut (u64, u64), t: &Tob) {{
    buf.0 = buf.0.wrapping_add(t.bid as u64).wrapping_add(t.ask as u64) ^ t.seq;
    buf.1 += 1;
}}

//@@DRIVE
/// The packet table, precomputed: symbols uniform over 0..N; `sorted` sorts them by symbol (stable).
fn table(sorted: bool, len: usize) -> Vec<Packet> {{
    let mut x = 0x2545F4914F6CDD1Du64;
    let mut v: Vec<Packet> = (0..len)
        .map(|i| {{
            x ^= x << 13; x ^= x >> 7; x ^= x << 17;
            let symbol = ((x >> 11) % N as u64) as u32;
            Packet {{ symbol, bid: 1000 + (x >> 40) as i64 % 64, ask: 1100 + (x >> 50) as i64 % 64, seq: i as u64 + 1 }}
        }})
        .collect();
    if sorted {{ v.sort_by_key(|p| p.symbol); }}
    v
}}

const TABLE: usize = 1 << 14;
const CHUNK_PASSES: usize = 100;

/// Runs `total` packets in chunks, prints ns/packet of each chunk and the digest.
fn drive(mut push: impl FnMut(Packet), args: &[String]) {{
    let sorted = args.get(1).map(|s| s == "sorted").unwrap_or(false);
    let total: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(40_000_000);
    let pk = table(sorted, TABLE);
    for p in &pk {{ push(*p); }}
    let chunks = (total / (TABLE * CHUNK_PASSES)).max(1);
    let mut ns = Vec::new();
    for _ in 0..chunks {{
        let t = Instant::now();
        for _ in 0..CHUNK_PASSES {{ for p in &pk {{ push(black_box(*p)); }} }}
        ns.push(t.elapsed().as_nanos() as f64 / (CHUNK_PASSES * TABLE) as f64);
    }}
    let mut s = ns.clone();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!("mode={{}} packets={{}} chunks={{}} min_ns={{:.3}} median_ns={{:.3}} digest={{:#x}}", if sorted {{ "sorted" }} else {{ "random" }},
        chunks * CHUNK_PASSES * TABLE + TABLE, chunks, s[0], s[s.len() / 2], DIGEST.load(Ordering::Relaxed));
}}

//@@MAIN
#[inline(never)]
#[no_mangle]
pub fn asm_push(p: &mut Pipeline<Packet, (Probe,)>, pkt: Packet) {{ p.push(pkt) }}

fn main() {{
    let args: Vec<String> = std::env::args().collect();
    let mut p = Pipeline::<Packet, (Probe,)>::build(Configs::new()).unwrap();
    black_box(asm_push as fn(&mut Pipeline<Packet, (Probe,)>, Packet));
    drive(|x| asm_push(&mut p, x), &args);
}}
"""

BOOK_BODY = """    fn new((): (), _: Self::Init<'_>) -> (Self, Tob) {
        (Book { n: 0, _p: core::marker::PhantomData }, Tob::default())
    }
    fn update(&mut self, i: Self::Input<'_>) -> Tob {
        self.n += 1;
        Tob { bid: i.t.bid, ask: i.t.ask + (self.n & 1) as i64, seq: i.t.seq.wrapping_add(self.n) }
    }"""

# `KEXPR` is the symbol's index as a u32 constant expression; `self.n` is the book's counter.
DIFF_OPS = """        let k: u32 = KEXPR;
        let c = (k as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
        match k % 8 {
            0 => { self.n += 1; Tob { bid: i.t.bid, ask: i.t.ask + (self.n & 1) as i64, seq: i.t.seq.wrapping_add(self.n) } }
            1 => { self.n ^= c; Tob { bid: i.t.bid + (self.n & 3) as i64, ask: i.t.ask, seq: i.t.seq ^ self.n } }
            2 => { self.n = self.n.rotate_left(k % 13 + 1).wrapping_add(c); Tob { bid: i.t.bid ^ (k as i64), ask: i.t.ask.wrapping_mul(3), seq: i.t.seq.wrapping_add(self.n >> 7) } }
            3 => { self.n += 1; if self.n % (k as u64 + 3) == 0 { Tob { bid: i.t.ask, ask: i.t.bid, seq: i.t.seq } } else { Tob { bid: i.t.bid - k as i64, ask: i.t.ask + k as i64, seq: i.t.seq + self.n } } }
            4 => { self.n = self.n.wrapping_mul(c).wrapping_add(1); Tob { bid: i.t.bid.wrapping_mul(k as i64 | 1), ask: i.t.ask >> (k % 5), seq: i.t.seq ^ (self.n >> 11) } }
            5 => { let mut a = i.t.ask; for j in 0..(self.n & 3) { a = a.wrapping_add((j as i64) * (k as i64 + 1)); } self.n += 1; Tob { bid: i.t.bid, ask: a, seq: i.t.seq.wrapping_add(c) } }
            6 => { self.n = (self.n ^ (self.n >> 3)).wrapping_add(k as u64); Tob { bid: i.t.bid.min(i.t.ask - k as i64), ask: i.t.ask.max(i.t.bid + k as i64), seq: i.t.seq.wrapping_sub(self.n) } }
            _ => { self.n += (k as u64 & 7) + 1; Tob { bid: i.t.bid + (i.t.seq & 7) as i64, ask: i.t.ask - (self.n & 7) as i64, seq: i.t.seq.rotate_left(k % 7 + 1) ^ self.n } }
        }"""

def book_body(kexpr):
    if not DIFF:
        return BOOK_BODY
    return BOOK_BODY.split("    fn update")[0] + "    fn update(&mut self, i: Self::Input<'_>) -> Tob {\n" + DIFF_OPS.replace("KEXPR", kexpr) + "\n    }"

def handles(prefix):
    return ", ".join(f"Book<{prefix}S{i}>" for i in range(n))

def cargo(name):
    root = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
    return f"""[package]
name = "{name}"
version = "0.0.0"
edition = "2021"

[dependencies]
dataflow-flat = {{ path = "{root}/dataflow-flat" }}

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
debug = false
debug-assertions = false
overflow-checks = false

[workspace]
"""

def _cargo_config(d):
    os.makedirs(os.path.join(d, ".cargo"), exist_ok=True)
    open(os.path.join(d, ".cargo", "config.toml"), "w").write(
        '[build]\nrustflags = ["-Zshare-generics=n", "-Ctarget-cpu=native"]\n\n[env]\nMALLOC_CONF = "dirty_decay_ms:-1,muzzy_decay_ms:-1"\n')

def write(name, main):
    d = os.path.join(out, name)
    os.makedirs(os.path.join(d, "src"), exist_ok=True)
    open(os.path.join(d, "Cargo.toml"), "w").write(cargo(f"ek_{name}"))
    _cargo_config(d)
    open(os.path.join(d, "src", "main.rs"), "w").write(main)

# ---- E and D ---------------------------------------------------------------------------------------------
variants = ",\n".join(f"    S{i}(Tob)" for i in range(n))
ws_arms = "\n".join(f"            {i} => {FN}::S{i}(tob)," for i in range(n))

def ws_and_book(selprefix_unused=None):
    return f"""
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Packet)]
pub struct Ws;
impl Component for Ws {{
    type Config = ();
    type Output = {FN};
    fn new((): (), _: Self::Init<'_>) -> (Self, {FN}) {{ (Ws, {FN}::S0(Tob::default())) }}
    fn update(&mut self, i: Self::Input<'_>) -> {FN} {{
        let m = i.p;
        let tob = Tob {{ bid: m.bid, ask: m.ask, seq: m.seq }};
        match m.symbol {{
{ws_arms}
            _ => unreachable!("symbol out of range"),
        }}
    }}
}}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(S(t): Ws)]
pub struct Book<S: Select<{FN}, Out = Tob>> {{ n: u64, _p: core::marker::PhantomData<S> }}
impl<S: Select<{FN}, Out = Tob>> Component for Book<S> {{
    type Config = ();
    type Output = Tob;
{book_body("<S as Select<%s>>::KEY" % FN)}
}}
"""

UH_E = f"""impl<S: Select<{FN}, Out = Tob>> UpdateHandler<Book<S>> for Oms {{
    #[inline(always)]
    fn on_update(&mut self, _: Self::Input<'_>, t: &Tob) {{ fold(&mut self.acc, t) }}
}}"""

key_arms = "\n".join(f"        {FN}::S{i}(..) => {i}," for i in range(n))
or_pat = " | ".join(f"{FN}::S{i}(t)" for i in range(n))
impls = "\n".join(
    f"""impl Select<{FN}> for markets::S{i} {{
    type Out = Tob;
    const KEY: u32 = {i};
    #[inline(always)]
    fn select(x: &{FN}) -> Option<&Tob> {{ if key_of(x) == {i} {{ Some(tob_of(x)) }} else {{ None }} }}
    #[inline(always)]
    fn key_of(x: &{FN}) -> u32 {{ key_of(x) }}
}}"""
    for i in range(n)
)
marks = "\n".join(f"    pub struct S{i};" for i in range(n))
e_body = COMMON.replace("%HANDLES%", handles("markets::")).replace("%UPDATE_HANDLER%", UH_E) + f"""
#[derive(Clone, Copy, Debug)]
pub enum {FN} {{
{variants},
}}
pub mod markets {{
{marks}
}}
/// The one shared key function: the variant index.
#[inline]
pub fn key_of(x: &{FN}) -> u32 {{
    match x {{
{key_arms}
    }}
}}
/// The one shared payload accessor.
#[inline]
pub fn tob_of(x: &{FN}) -> &Tob {{
    match x {{
        {or_pat} => t,
    }}
}}
{impls}
""" + ws_and_book()
write("e", e_body)

# ---- E2 (diagnostic): E with a branchless-looking `select`: (key_of(x) == i).then(|| tob_of(x)) ----
e2_body = e_body.replace("{ if key_of(x) == ", "{ (key_of(x) == ").replace(" { Some(tob_of(x)) } else { None } }", ").then(|| tob_of(x)) }")
import re
e2_body = re.sub(r"\(key_of\(x\) == (\d+) \)", r"(key_of(x) == \1)", e2_body)
write("e2", e2_body)

d_body = COMMON.replace("%HANDLES%", handles(f"{MOD}::")).replace("%UPDATE_HANDLER%", UH_E) + f"""
#[derive(Clone, Copy, Debug, Demux)]
pub enum {FN} {{
{variants},
}}
""" + ws_and_book()
write("d", d_body)

# ---- H: hand-written ---------------------------------------------------------------------------------------
arms = "\n".join(
    f"            {i} => {{ let o = self.books[{i}].step::<{i}>(&tob); fold(&mut buf, &o); }}" for i in range(n)
)
HEAD = COMMON.split("//@@SINK")[0]
DRIVE = COMMON.split("//@@DRIVE")[1].split("//@@MAIN")[0]
h_body = HEAD + f"""
#[inline(always)]
fn fold(buf: &mut (u64, u64), t: &Tob) {{
    buf.0 = buf.0.wrapping_add(t.bid as u64).wrapping_add(t.ask as u64) ^ t.seq;
    buf.1 += 1;
}}
#[derive(Default, Clone, Copy)]
pub struct BookH {{ n: u64 }}
impl BookH {{
    #[inline(always)]
    fn step<const K: u32>(&mut self, t: &Tob) -> Tob {{
%STEP%
    }}
}}
pub struct Hand {{ books: [BookH; N], d: u64 }}
impl Hand {{
    #[inline(always)]
    pub fn push(&mut self, m: Packet) {{
        let tob = Tob {{ bid: m.bid, ask: m.ask, seq: m.seq }};
        let mut buf = (0u64, 0u64);
        match m.symbol {{
{arms}
            _ => unreachable!(),
        }}
        self.d = self.d.rotate_left(1) ^ buf.0.wrapping_add(buf.1);
        DIGEST.store(self.d, Ordering::Relaxed);
    }}
}}
//@@DRIVE
""" + DRIVE + """
#[inline(never)]
#[no_mangle]
pub fn asm_push(p: &mut Hand, pkt: Packet) { p.push(pkt) }
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut h = Hand { books: [BookH::default(); N], d: 0 };
    drive(|x| asm_push(&mut h, x), &args);
}
"""
step = ("        let i = Inp { t };\n" + DIFF_OPS.replace("KEXPR", "K")) if DIFF else "        self.n += 1;\n        Tob { bid: t.bid, ask: t.ask + (self.n & 1) as i64, seq: t.seq.wrapping_add(self.n) }"
h_body = h_body.replace("%STEP%", step).replace("#[derive(Default, Clone, Copy)]\npub struct BookH", "struct Inp<'a> { t: &'a Tob }\n#[derive(Default, Clone, Copy)]\npub struct BookH", 1)
write("h", h_body)
print(f"wrote e e2 d h with N={n} under {out}")
