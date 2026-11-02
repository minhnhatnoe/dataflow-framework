#!/usr/bin/env python3
"""Generator for the handler-component benchmark: an OMS-style graph.

usage: bench/gen_handler.py N OUTDIR [for]

With `for`, the handler lists its quoters as one `for S in catalog![S0, ..] { Quoter<S> }` entry instead of
N written-out entries (same plan, same code).

One root `Msg { key, v }` -> `Feed` (Option<Frame>, an enum) -> `Quoter<Sn>` for N symbols
(`Some(S(u)): Feed`, the variants of the enum `Frame`, exclusive) -> one joining node, in three flavours selected by cargo
features of the generated crate:

  handler    `Oms`, a handler component: `on_update` per fired quoter, then `update`
  component  `Sink`, a regular `Component` with N named inputs: it scans them all and takes the
             quoter with the newest sequence number (what a plain component has to do)
  hand       a hand-written nested match with the quoters' outputs in an array

Each is followed by a probe that folds the result into a digest (a static); the runner checks that
the three digests agree after the same deterministic packet stream and times them interleaved,
pinned (run it under `taskset -c N`). Keys range over N + N/8 values, so some packets are read by
no quoter.
"""
import os
import sys

n = int(sys.argv[1])
out = sys.argv[2]
os.makedirs(os.path.join(out, "src"), exist_ok=True)

variants = "\n".join(f"    S{i}(u64)," for i in range(n))
feed_arms = "\n".join(f"            {i} => Frame::S{i}(i.m.v)," for i in range(n))
if len(sys.argv) > 3 and sys.argv[3] == "for":
    handles = "for S: Sym in catalog![" + ", ".join(f"frame::S{i}" for i in range(n)) + "] { Quoter<S> }"
else:
    handles = ", ".join(f"Quoter<frame::S{i}>" for i in range(n))
fields = ", ".join(f"q{i}: Quoter<frame::S{i}>" for i in range(n))
hand_fields = " ".join(f"q{i}: (u64, u64)," for i in range(n))
hand_init = " ".join(f"q{i}: (0, 0)," for i in range(n))
hand_arms = "\n".join(f"            {i} => {{ self.q{i} = (m.v, m.v * 2 + {i}); self.oms = (self.q{i}.1, 1); self.tail(); }}" for i in range(n))
pick = "\n".join(f"        if i.q{i}.0 > best.0 {{ best = *i.q{i}; }}" for i in range(n))

def _cargo_config(d):
    os.makedirs(os.path.join(d, ".cargo"), exist_ok=True)
    open(os.path.join(d, ".cargo", "config.toml"), "w").write(
        '[build]\nrustflags = ["-Zshare-generics=n", "-Ctarget-cpu=native"]\n\n[env]\nMALLOC_CONF = "dirty_decay_ms:-1,muzzy_decay_ms:-1"\n')

cargo = f"""[package]
name = "handler_bench"
version = "0.0.0"
edition = "2021"

[dependencies]
dataflow-flat = {{ path = "{os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'dataflow-flat'))}" }}

[features]
default = ["handler", "component", "hand"]
handler = []
component = []
hand = []

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
open(os.path.join(out, "Cargo.toml"), "w").write(cargo)
_cargo_config(out)

main = f"""#![allow(dead_code, unused_imports)]
use std::sync::atomic::{{AtomicU64, Ordering}};
use std::time::Instant;

use dataflow_flat::{{catalog, Component, ComponentSubscription, Configs, Demux, Pipeline, Root, Select, UpdateHandler}};

pub const N: usize = {n};

#[derive(Root)]
pub struct Msg {{ pub key: u32, pub v: u64 }}
/// One variant per symbol (the selectors are `frame::S0`, ..), and `Other` for keys nobody reads.
#[derive(Demux)]
pub enum Frame {{
{variants}
    Other(u64),
}}
/// A symbol: a selector of `Frame` that yields the payload.
pub trait Sym: Select<Frame, Out = u64> {{}}
impl<S: Select<Frame, Out = u64>> Sym for S {{}}

#[derive(ComponentSubscription)]
#[subscribe(m: Msg)]
pub struct Feed;
impl Component for Feed {{
    type Config = ();
    type Output = Option<Frame>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Option<Frame>) {{ (Feed, None) }}
    fn update(&mut self, i: Self::Input<'_>) -> Option<Frame> {{ Some(match i.m.key {{
{feed_arms}
        _ => Frame::Other(i.m.v),
    }}) }}
}}

/// (sequence, value)
#[derive(ComponentSubscription)]
#[subscribe(Some(S(u)): Feed)]
pub struct Quoter<S: Sym>(std::marker::PhantomData<S>);
impl<S: Sym> Component for Quoter<S> {{
    type Config = ();
    type Output = (u64, u64);
    fn new((): (), _: Self::Init<'_>) -> (Self, (u64, u64)) {{ (Quoter(std::marker::PhantomData), (0, 0)) }}
    fn update(&mut self, i: Self::Input<'_>) -> (u64, u64) {{ (*i.u, *i.u * 2 + <S as Select<Frame>>::KEY as u64) }}
}}

pub static DIGEST: AtomicU64 = AtomicU64::new(0);

// -- handler flavour ---------------------------------------------------------------------------
#[cfg(feature = "handler")]
pub mod h {{
    use super::*;
    #[derive(ComponentSubscription)]
    #[subscribe({handles})]
    pub struct Oms {{ acc: (u64, u64) }}
    impl Component for Oms {{
        type Config = ();
        type Output = (u64, u64);
        fn new((): (), _: Self::Init<'_>) -> (Self, (u64, u64)) {{ (Oms {{ acc: (0, 0) }}, (0, 0) ) }}
        #[inline(always)]
        fn update(&mut self, _: Self::Input<'_>) -> (u64, u64) {{ std::mem::take(&mut self.acc) }}
    }}
    impl<S: Sym> UpdateHandler<Quoter<S>> for Oms {{
        #[inline(always)]
        fn on_update(&mut self, v: &(u64, u64)) {{ self.acc.0 += v.1; self.acc.1 += 1; }}
    }}
    #[derive(ComponentSubscription)]
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
}}

// -- regular component flavour -----------------------------------------------------------------
#[cfg(feature = "component")]
pub mod c {{
    use super::*;
    #[derive(ComponentSubscription)]
    #[subscribe({fields})]
    pub struct Sink;
    impl Component for Sink {{
        type Config = ();
        type Output = (u64, u64);
        fn new((): (), _: Self::Init<'_>) -> (Self, (u64, u64)) {{ (Sink, (0, 0)) }}
        fn update(&mut self, i: Self::Input<'_>) -> (u64, u64) {{
            let mut best = (0u64, 0u64);
{pick}
            (best.1, 1)
        }}
    }}
    #[derive(ComponentSubscription)]
    #[subscribe(o: Sink)]
    pub struct ProbeC {{ d: u64 }}
    impl Component for ProbeC {{
        type Config = ();
        type Output = ();
        fn new((): (), _: Self::Init<'_>) -> (Self, ()) {{ (ProbeC {{ d: 0 }}, ()) }}
        fn update(&mut self, i: Self::Input<'_>) {{
            self.d = self.d.rotate_left(1) ^ i.o.0.wrapping_add(i.o.1);
            DIGEST.store(self.d, Ordering::Relaxed);
        }}
    }}
}}

// -- hand-written ------------------------------------------------------------------------------
#[cfg(feature = "hand")]
pub struct Hand {{ feed: Option<(u32, u64)>, oms: (u64, u64), {hand_fields} d: u64 }}
#[cfg(feature = "hand")]
impl Hand {{
    pub fn new() -> Self {{ Hand {{ feed: None, oms: (0, 0), {hand_init} d: 0 }} }}
    #[inline(always)]
    pub fn push(&mut self, m: Msg) {{
        self.feed = Some((m.key, m.v));
        match m.key {{
{hand_arms}
            _ => {{}}
        }}
    }}
    #[inline(always)]
    fn tail(&mut self) {{
        self.d = self.d.rotate_left(1) ^ self.oms.0.wrapping_add(self.oms.1);
        DIGEST.store(self.d, Ordering::Relaxed);
    }}
}}

// Out-of-line copies of the push, for reading the assembly (`objdump -d --disassemble=asm_push_handler`).
#[cfg(feature = "handler")]
#[no_mangle]
#[inline(never)]
pub fn asm_push_handler(p: &mut Pipeline<Msg, (h::Probe,)>, key: u32, v: u64) {{ p.push(Msg {{ key, v }}) }}
#[cfg(feature = "hand")]
#[no_mangle]
#[inline(never)]
pub fn asm_push_hand(p: &mut Hand, key: u32, v: u64) {{ p.push(Msg {{ key, v }}) }}

fn stream(len: usize) -> Vec<(u32, u64)> {{
    let mut x = 0x2545F4914F6CDD1Du64;
    let range = (N + N / 8 + 1) as u64;
    (0..len)
        .map(|i| {{
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (((x >> 11) % range) as u32, i as u64 + 1)
        }})
        .collect()
}}

fn main() {{
    #[cfg(feature = "handler")]
    std::hint::black_box(asm_push_handler as fn(&mut Pipeline<Msg, (h::Probe,)>, u32, u64));
    #[cfg(feature = "hand")]
    std::hint::black_box(asm_push_hand as fn(&mut Hand, u32, u64));
    let packets = stream(1 << 18);
    let rounds: usize = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(15);
    #[allow(unused_mut)]
    let mut names: Vec<&str> = Vec::new();
    #[allow(unused_mut)]
    let mut digests: Vec<u64> = Vec::new();
    #[allow(unused_mut)]
    let mut times: Vec<Vec<f64>> = Vec::new();

    #[cfg(feature = "handler")]
    let mut ph = Pipeline::<Msg, (h::Probe,)>::build(Configs::new()).unwrap();
    #[cfg(feature = "component")]
    let mut pc = Pipeline::<Msg, (c::ProbeC,)>::build(Configs::new()).unwrap();
    #[cfg(feature = "hand")]
    let mut hand = Hand::new();

    macro_rules! flavour {{
        ($name:expr, $push:expr) => {{{{
            DIGEST.store(0, Ordering::Relaxed);
            for &(k, v) in &packets {{ $push(Msg {{ key: k, v }}); }}
            names.push($name);
            digests.push(DIGEST.load(Ordering::Relaxed));
            times.push(Vec::new());
        }}}};
    }}
    #[cfg(feature = "handler")]
    flavour!("handler", |m| ph.push(m));
    #[cfg(feature = "component")]
    flavour!("component", |m| pc.push(m));
    #[cfg(feature = "hand")]
    flavour!("hand", |m| hand.push(m));
    assert!(digests.windows(2).all(|w| w[0] == w[1]), "flavours disagree: {{names:?}} {{digests:?}}");
    println!("digests agree: {{:#x}} ({{}} packets)", digests[0], packets.len());

    for _ in 0..rounds {{
        let mut ix = 0;
        #[cfg(feature = "handler")]
        {{
            let t = Instant::now();
            for &(k, v) in &packets {{ ph.push(Msg {{ key: k, v }}); }}
            times[ix].push(t.elapsed().as_nanos() as f64 / packets.len() as f64);
            ix += 1;
        }}
        #[cfg(feature = "component")]
        {{
            let t = Instant::now();
            for &(k, v) in &packets {{ pc.push(Msg {{ key: k, v }}); }}
            times[ix].push(t.elapsed().as_nanos() as f64 / packets.len() as f64);
            ix += 1;
        }}
        #[cfg(feature = "hand")]
        {{
            let t = Instant::now();
            for &(k, v) in &packets {{ hand.push(Msg {{ key: k, v }}); }}
            times[ix].push(t.elapsed().as_nanos() as f64 / packets.len() as f64);
            ix += 1;
        }}
        let _ = ix;
    }}
    for (n, t) in names.iter().zip(&mut times) {{
        t.sort_by(|a, b| a.partial_cmp(b).unwrap());
        println!("N={{}} {{:<10}} min {{:6.2}}  median {{:6.2}} ns/packet", N, n, t[0], t[t.len() / 2]);
    }}
}}
"""
open(os.path.join(out, "src", "main.rs"), "w").write(main)
