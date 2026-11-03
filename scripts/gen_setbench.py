#!/usr/bin/env python3
"""Generate a scratch crate comparing a set reader (`feeds: for S in CatN { Feed<S> }` with
`fold`) with the same reader as N named entries and with a hand-written sum, for N in the given sizes.

usage: gen_setbench.py OUT_DIR LIB_DIR [N ...]    (default N: 16 300)

The crate has one binary: `setbench <n> <variant>` times pushes; `#[inline(never)]` wrappers
(`push_set_N`, `push_named_N`, `push_hand_N`) make the code easy to find in the disassembly."""
import os
import sys

out, lib = sys.argv[1], os.path.abspath(sys.argv[2])
sizes = [int(x) for x in sys.argv[3:]] or [16, 300]
os.makedirs(out + "/src", exist_ok=True)
open(out + "/Cargo.toml", "w").write(f"""[package]
name = "setbench"
version = "0.1.0"
edition = "2021"

[dependencies]
dataflow-flat = {{ path = "{lib}/dataflow-flat" }}

[workspace]

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
""")
s = """#![allow(long_running_const_eval)]
use std::marker::PhantomData;
use std::hint::black_box;
use std::time::Instant;
use dataflow_flat::{catalog, Component, ComponentSubscription, Configs, Pipeline, Root, UpdateHandler};

#[derive(Root)]
pub struct Msg { pub v: u64 }
pub struct Sy<const K: usize>;
pub trait Num: 'static { const N: u64; }
impl<const K: usize> Num for Sy<K> { const N: u64 = K as u64 + 1; }

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct Feed<S: Num>(PhantomData<S>);
impl<S: Num> Component for Feed<S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (Feed(PhantomData), 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u64 { i.m.v.wrapping_mul(S::N) }
}
"""
for n in sizes:
    cat = ", ".join(f"Sy<{k}>" for k in range(n))
    names = ", ".join(f"f{k}: Feed<Sy<{k}>>" for k in range(n))
    s += f"""
pub type Cat{n} = catalog![{cat}];
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(feeds: for S: Num in Cat{n} {{ Feed<S> }})]
pub struct SetSum{n}(u64);
impl Component for SetSum{n} {{
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {{ (SetSum{n}(0), 0) }}
    fn update(&mut self, i: Self::Input<'_>) -> u64 {{
        i.feeds.fold(0u64, |a, v: &u64| a.wrapping_add(*v))
    }}
}}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe({names})]
pub struct NamedSum{n}(u64);
impl Component for NamedSum{n} {{
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {{ (NamedSum{n}(0), 0) }}
    fn update(&mut self, i: Self::Input<'_>) -> u64 {{
        0u64{"".join(f".wrapping_add(*i.f{k})" for k in range(n))}
    }}
}}
pub struct Hand{n} {{ f: [u64; {n}], total: u64 }}
impl Hand{n} {{
    #[inline(always)]
    fn push(&mut self, v: u64) {{
        {"".join(f"self.f[{k}] = v.wrapping_mul({k + 1}); " for k in range(n))}
        self.total = 0u64{"".join(f".wrapping_add(self.f[{k}])" for k in range(n))};
    }}
}}
#[inline(never)] pub fn push_set_{n}(p: &mut Pipeline<Msg, (SetSum{n},)>, v: u64) {{ p.push(Msg {{ v }}) }}
#[inline(never)] pub fn push_named_{n}(p: &mut Pipeline<Msg, (NamedSum{n},)>, v: u64) {{ p.push(Msg {{ v }}) }}
#[inline(never)] pub fn push_hand_{n}(p: &mut Hand{n}, v: u64) {{ p.push(v) }}
"""
s += "\nfn main() {\n    let a: Vec<String> = std::env::args().collect();\n    let n: usize = a[1].parse().unwrap();\n    let variant = a[2].as_str();\n    let iters: u64 = 2_000_000;\n"
s += "    match (n, variant) {\n"
for n in sizes:
    s += f"""        ({n}, "set") => {{ let mut p = Pipeline::<Msg, (SetSum{n},)>::build(Configs::new()).unwrap(); let t = Instant::now(); for i in 0..iters {{ push_set_{n}(&mut p, black_box(i)); }} println!("set {n}: {{:.2}} ns/push", t.elapsed().as_nanos() as f64 / iters as f64); }}
        ({n}, "named") => {{ let mut p = Pipeline::<Msg, (NamedSum{n},)>::build(Configs::new()).unwrap(); let t = Instant::now(); for i in 0..iters {{ push_named_{n}(&mut p, black_box(i)); }} println!("named {n}: {{:.2}} ns/push", t.elapsed().as_nanos() as f64 / iters as f64); }}
        ({n}, "hand") => {{ let mut p = Hand{n} {{ f: [0; {n}], total: 0 }}; let t = Instant::now(); for i in 0..iters {{ push_hand_{n}(&mut p, black_box(i)); }} black_box(p.total); println!("hand {n}: {{:.2}} ns/push", t.elapsed().as_nanos() as f64 / iters as f64); }}
"""
s += "        _ => panic!(\"usage: setbench <n> set|named|hand\"),\n    }\n}\n"
open(out + "/src/main.rs", "w").write(s)
