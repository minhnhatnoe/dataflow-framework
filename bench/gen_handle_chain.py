#!/usr/bin/env python3
"""Chain benchmark for handle components: does zero-copy pay off with large outputs?

usage: bench/gen_handle_chain.py K [N] [--lib CHECKOUT] [--out DIR]
  K  words (u64) per output: 8 (64 B), 64 (512 B), 512 (4 KB)
  N  chain length (default 64)

Generates a crate with four variants of the same chain `Tick -> L0 -> L1 -> .. -> L(N-1) -> Sink`
and a runner that times them (min of alternating rounds, run it pinned: taskset -c 9):

  fw_value   framework, by-value components: `update` returns `[u64; K]`, stored in the pipeline,
             the next component reads `&[u64; K]`
  fw_handle  framework, `#[handle]` components: the state holds the buffer, `update` writes it in
             place and returns `()`, the next component reads the view `&[u64; K]`
  hand_value hand-written: `o[i] = step(&o[i-1])`
  hand_inplace hand-written: `step_in(&mut b[i], &b[i-1])` (what a handle makes possible)

All four compute the same values; the sink adds the sum of all words of the last output to a counter.
"""
import argparse, os

ap = argparse.ArgumentParser()
ap.add_argument("K", type=int)
ap.add_argument("N", type=int, nargs="?", default=64)
ap.add_argument("--lib", default=os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
ap.add_argument("--out", required=True)
a = ap.parse_args()
K, N = a.K, a.N
out = a.out
os.makedirs(f"{out}/src", exist_ok=True)

open(f"{out}/Cargo.toml", "w").write(f"""[package]
name = "chain"
version = "0.0.0"
edition = "2021"

[dependencies]
dataflow-flat = {{ path = "{a.lib}/dataflow-flat" }}

[workspace]

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
debug = false
debug-assertions = false
overflow-checks = false
""")
open(f"{out}/rust-toolchain.toml", "w").write('[toolchain]\nchannel = "nightly-2026-10-30"\n')

L = []
L.append(f"""#![allow(long_running_const_eval)]
#![recursion_limit = "4096"]
use dataflow_flat::{{Component, ComponentSubscription, Configs, HandleComponent, Pipeline, Root}};
use std::sync::atomic::{{AtomicU64, Ordering::Relaxed}};
use std::time::Instant;

const K: usize = {K};
const N: usize = {N};
type Out = [u64; K];
static SINK: AtomicU64 = AtomicU64::new(0);

#[derive(Root)]
pub struct Tick(pub u64);

/// The sink reads every word, so no lane of the chain is dead code.
#[inline(always)]
fn fold(o: &Out) -> u64 {{
    let mut s = 0u64;
    for w in o.iter() {{ s = s.wrapping_add(*w); }}
    s
}}
#[inline(always)]
fn first(t: u64, c: u64) -> Out {{
    let mut o = [0u64; K];
    for j in 0..K {{ o[j] = t.wrapping_add(c).wrapping_add(j as u64); }}
    o
}}
#[inline(always)]
fn first_in(b: &mut Out, t: u64, c: u64) {{
    for j in 0..K {{ b[j] = t.wrapping_add(c).wrapping_add(j as u64); }}
}}
#[inline(always)]
fn step(p: &Out, c: u64) -> Out {{
    let mut o = [0u64; K];
    for j in 0..K {{ o[j] = p[j].wrapping_mul(31).wrapping_add(c + j as u64); }}
    o
}}
#[inline(always)]
fn step_in(b: &mut Out, p: &Out, c: u64) {{
    for j in 0..K {{ b[j] = p[j].wrapping_mul(31).wrapping_add(c + j as u64); }}
}}
""")

# framework, by value
for i in range(N):
    sub = "t: Tick" if i == 0 else f"p: V{i-1}"
    body = "first(i.t.0, 0)" if i == 0 else f"step(i.p, {i})"
    L.append(f"""#[derive(ComponentSubscription)]
#[subscribe({sub})]
pub struct V{i};
impl Component for V{i} {{
    type Config = ();
    type Output = Out;
    fn new((): (), _: Self::Init<'_>) -> (Self, Out) {{ (V{i}, [0; K]) }}
    #[inline(always)]
    fn update(&mut self, i: Self::Input<'_>) -> Out {{ {body} }}
}}""")
L.append(f"""#[derive(ComponentSubscription)]
#[subscribe(p: V{N-1})]
pub struct SinkV;
impl Component for SinkV {{
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {{ (SinkV, ()) }}
    #[inline(always)]
    fn update(&mut self, i: Self::Input<'_>) {{ SINK.fetch_add(fold(i.p), Relaxed); }}
}}""")
# framework, handle
for i in range(N):
    sub = "t: Tick" if i == 0 else f"p: H{i-1}"
    body = "first_in(&mut self.buf, i.t.0, 0)" if i == 0 else f"step_in(&mut self.buf, i.p, {i})"
    L.append(f"""#[derive(ComponentSubscription)]
#[subscribe({sub})]
#[handle]
pub struct H{i} {{ buf: Out }}
impl HandleComponent for H{i} {{
    type Config = ();
    type Handle = ();
    type Output<'a> = &'a Out;
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {{ (H{i} {{ buf: [0; K] }}, ()) }}
    #[inline(always)]
    fn update(&mut self, i: Self::Input<'_>) {{ {body} }}
    #[inline(always)]
    fn output<'a>(&'a self, (): &'a ()) -> &'a Out {{ &self.buf }}
}}""")
L.append(f"""#[derive(ComponentSubscription)]
#[subscribe(p: H{N-1})]
pub struct SinkH;
impl Component for SinkH {{
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {{ (SinkH, ()) }}
    #[inline(always)]
    fn update(&mut self, i: Self::Input<'_>) {{ SINK.fetch_add(fold(i.p), Relaxed); }}
}}""")

# hand-written
hv = ["pub struct HandVal { o: [Out; N] }", "impl HandVal {", "    fn new() -> Self { HandVal { o: [[0; K]; N] } }",
      "    #[inline(never)] fn push(&mut self, t: u64) {", "        self.o[0] = first(t, 0);"]
for i in range(1, N):
    hv.append(f"        self.o[{i}] = step(&self.o[{i-1}], {i});")
hv += [f"        SINK.fetch_add(fold(&self.o[{N-1}]), Relaxed);", "    }", "}"]
hi = ["pub struct HandIn { b: [Out; N] }", "impl HandIn {", "    fn new() -> Self { HandIn { b: [[0; K]; N] } }",
      "    #[inline(never)] fn push(&mut self, t: u64) {", "        first_in(&mut self.b[0], t, 0);"]
for i in range(1, N):
    hi.append(f"        {{ let (l, r) = self.b.split_at_mut({i}); step_in(&mut r[0], &l[{i-1}], {i}); }}")
hi += [f"        SINK.fetch_add(fold(&self.b[{N-1}]), Relaxed);", "    }", "}"]
L.append("\n".join(hv))
L.append("\n".join(hi))

iters = max(200, 20_000_000 // (N * K))
L.append(f"""
#[inline(never)]
fn push_v(p: &mut Pipeline<Tick, (SinkV,)>, x: u64) {{ p.push(Tick(x)); }}
#[inline(never)]
fn push_h(p: &mut Pipeline<Tick, (SinkH,)>, x: u64) {{ p.push(Tick(x)); }}

fn time<F: FnMut(u64)>(mut f: F, iters: u64) -> f64 {{
    let t = Instant::now();
    for i in 0..iters {{ f(std::hint::black_box(i)); }}
    t.elapsed().as_nanos() as f64 / iters as f64
}}
fn main() {{
    let iters: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or({iters});
    let rounds: usize = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(15);
    let mut fv = Pipeline::<Tick, (SinkV,)>::build(Configs::new()).unwrap();
    let mut fh = Pipeline::<Tick, (SinkH,)>::build(Configs::new()).unwrap();
    let mut hv = HandVal::new();
    let mut hi = HandIn::new();
    let names = ["fw_value", "fw_handle", "hand_value", "hand_inplace"];
    let mut best = [f64::MAX; 4];
    let mut sums = [0u64; 4];
    for _ in 0..rounds {{
        SINK.store(0, Relaxed);
        best[0] = best[0].min(time(|x| push_v(&mut fv, x), iters));
        sums[0] = SINK.load(Relaxed);
        SINK.store(0, Relaxed);
        best[1] = best[1].min(time(|x| push_h(&mut fh, x), iters));
        sums[1] = SINK.load(Relaxed);
        SINK.store(0, Relaxed);
        best[2] = best[2].min(time(|x| hv.push(x), iters));
        sums[2] = SINK.load(Relaxed);
        SINK.store(0, Relaxed);
        best[3] = best[3].min(time(|x| hi.push(x), iters));
        sums[3] = SINK.load(Relaxed);
    }}
    assert!(sums.iter().all(|s| *s == sums[0]), "variants disagree: {{sums:?}}");
    for (n, b) in names.iter().zip(best) {{
        println!("RESULT K={K} bytes={{}} N={N} {{n}} {{b:.1}} ns/push", K * 8);
    }}
}}
""")
open(f"{out}/src/main.rs", "w").write("\n".join(L) + "\n")
print("ok", K, N, out)
