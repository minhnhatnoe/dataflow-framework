#!/usr/bin/env python3
"""Graphs whose conditions the push asks again (they used the `fired` flags before):
  handler24: 24 guarded components and an always-running handler with 24 handled edges
  chain20:   a chain of 20 guards on stored outputs (each reads the one before) and a handler at the end
  indep8:    8 independent guards and an always-running reader that handles all of them
Writes a scratch crate (Cargo.toml + src/main.rs); `cargo run --release -- SHAPE PACKETS` pushes
random packets and prints ns per packet and a checksum (equal on every checkout)."""
import argparse
import os

p = argparse.ArgumentParser()
p.add_argument("--out", required=True)
p.add_argument("--lib", required=True)
p.add_argument("--depth", type=int, default=20, help="length of the guard chain (shape chain20 is this chain)")
p.add_argument("--indep", type=int, default=8, help="number of independent guards (shape indep8)")
a = p.parse_args()

src = ['''#![allow(dead_code)]
use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline, Root, Select, UpdateHandler, GUARD};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
static SUM: AtomicU64 = AtomicU64::new(0);
#[derive(Root)]
pub struct Pk { pub a: u32, pub b: u32 }
pub struct Mult<const M: u32>;
impl<const M: u32> Select<Pk> for Mult<M> {
    type Out = u32;
    const KEY: u32 = GUARD;
    fn select(x: &Pk) -> Option<&u32> { (x.a % M == 0).then_some(&x.a) }
    fn key_of(_: &Pk) -> u32 { GUARD }
}
pub struct Any;
impl Select<u32> for Any {
    type Out = u32;
    const KEY: u32 = GUARD;
    fn select(x: &u32) -> Option<&u32> { (*x % 7 != 6).then_some(x) }
    fn key_of(_: &u32) -> u32 { GUARD }
}
macro_rules! comp {
    ($name:ident, [$($sub:tt)*], |$i:ident| $body:expr) => {
        #[derive(ComponentSubscription, UpdateHandler)]
        #[subscribe($($sub)*)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = u32;
            fn new((): (), _: Self::Init<'_>) -> (Self, u32) { ($name, 0) }
            fn update(&mut self, $i: Self::Input<'_>) -> u32 { $body }
        }
    };
}
''']

# handler24
ws = list(range(1, 25))
for m in ws:
    src.append(f"comp!(W{m}, [Mult::<{m}>(x): Pk], |i| *i.x + {m});")
src.append("#[derive(ComponentSubscription)]\n#[subscribe(p: Pk, " + ", ".join(f"w{m}: W{m}" for m in ws) + ")]\npub struct Hd { n: u32 }")
src.append("""impl Component for Hd {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) { (Hd { n: 0 }, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u32 { SUM.fetch_add(self.n as u64 + i.p.b as u64, Relaxed); self.n = 0; 0 }
}
impl UpdateHandler<Pk> for Hd { const ACTIVE: bool = false; }""")
for m in ws:
    src.append(f"impl UpdateHandler<W{m}> for Hd {{ fn on_update(&mut self, _: Self::Input<'_>, v: &u32) {{ self.n = self.n.wrapping_add(*v); }} }}")

# chain20
src.append("comp!(K0, [Mult::<2>(x): Pk], |i| *i.x);")
for k in range(1, a.depth):
    src.append(f"comp!(K{k}, [Any(x): K{k-1}], |i| *i.x + {k});")
src.append("""#[derive(ComponentSubscription)]
#[subscribe(p: Pk, e: KLAST)]
pub struct Hc { n: u32 }
impl Component for Hc {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) { (Hc { n: 0 }, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u32 { SUM.fetch_add(self.n as u64 + i.p.b as u64, Relaxed); self.n = 0; 0 }
}
impl UpdateHandler<Pk> for Hc { const ACTIVE: bool = false; }
impl UpdateHandler<KLAST> for Hc { fn on_update(&mut self, _: Self::Input<'_>, v: &u32) { self.n = self.n.wrapping_add(*v); } }""".replace("KLAST", f"K{a.depth - 1}"))

# indep8
for m in range(1, a.indep + 1):
    src.append(f"comp!(I{m}, [Mult::<{m + 2}>(x): Pk], |i| *i.x);")
src.append("#[derive(ComponentSubscription)]\n#[subscribe(p: Pk, " + ", ".join(f"g{m}: I{m}" for m in range(1, a.indep + 1)) + ")]\npub struct Hi { n: u32 }")
src.append("""impl Component for Hi {
    type Config = ();
    type Output = u32;
    fn new((): (), _: Self::Init<'_>) -> (Self, u32) { (Hi { n: 0 }, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u32 { SUM.fetch_add(self.n as u64 + i.p.b as u64, Relaxed); self.n = 0; 0 }
}
impl UpdateHandler<Pk> for Hi { const ACTIVE: bool = false; }""")
for m in range(1, a.indep + 1):
    src.append(f"impl UpdateHandler<I{m}> for Hi {{ fn on_update(&mut self, _: Self::Input<'_>, v: &u32) {{ self.n = self.n.wrapping_add(*v); }} }}")

src.append('''
fn run<S: dataflow_flat::Sinks<Pk>>(n: u32) {
    let mut p = Pipeline::<Pk, S>::build(Configs::new()).unwrap();
    let mut r: u64 = 12345;
    let pk: Vec<Pk> = (0..4096).map(|_| { r = r.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); Pk { a: ((r >> 33) % 60) as u32, b: (r >> 40) as u32 & 0xff } }).collect();
    let t = std::time::Instant::now();
    for k in 0..n as usize {
        let x = &pk[k & 4095];
        p.push(Pk { a: x.a, b: x.b });
    }
    let ns = t.elapsed().as_nanos() as f64 / n as f64;
    println!("ns/packet {:.2} checksum {}", ns, SUM.load(Relaxed));
}
fn main() {
    let shape = std::env::args().nth(1).unwrap();
    let n: u32 = std::env::args().nth(2).unwrap().parse().unwrap();
    match shape.as_str() {
        "handler24" => run::<(Hd,)>(n),
        "chain20" => run::<(Hc,)>(n),
        "indep8" => run::<(Hi,)>(n),
        _ => panic!("shape"),
    }
}
''')
os.makedirs(os.path.join(a.out, "src"), exist_ok=True)
open(os.path.join(a.out, "Cargo.toml"), "w").write(f'''[package]
name = "guardbench"
version = "0.1.0"
edition = "2021"

[dependencies]
dataflow-flat = {{ path = "{a.lib}" }}

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"

[workspace]
''')
open(os.path.join(a.out, "src", "main.rs"), "w").write("\n".join(src) + "\n")
