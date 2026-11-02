#!/usr/bin/env python3
"""N feeds, each with a trade and a depth consumer, dispatched on the message kind (a switch):
framework with a handle source (selectors on a view) or a by-value source (Select<Msg>), against
hand-written code (as handoff/experiments/handle-output/src/sel.rs).
usage: bench/gen_handle_select.py OUTDIR N LIB MODE     MODE = handle-join | handle-direct | value-join | value-direct
  join:   a sink per feed joins its two consumers (N <= 15);   direct: the consumers are the sinks (N <= 7)"""
import sys, os

out, n, lib, mode = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
handle = mode.startswith("handle")
direct = mode.endswith("direct")
os.makedirs(out + "/src", exist_ok=True)
open(out + "/Cargo.toml", "w").write(f'''[package]
name = "hbench"
version = "0.0.0"
edition = "2021"

[dependencies]
dataflow-flat = {{ path = "{lib}/dataflow-flat" }}

[workspace]

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
debug = false
''')

src = r'''#![allow(dead_code)]
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::time::Instant;
static OUT_FW: [AtomicU64; 64] = [const { AtomicU64::new(0) }; 64];
static OUT_H: [AtomicU64; 64] = [const { AtomicU64::new(0) }; 64];
use dataflow_flat::{Component, ComponentSubscription, Configs, Demux, HandleComponent, Pipeline, Root, UpdateHandler};

#[derive(Root)]
pub struct X(pub u64);

pub struct Trade { px: u64, qty: u64 }
pub struct Depth { lv: [u64; 4] }
#[derive(Demux)]
pub enum Msg { Trade(Trade), Depth(Depth) }
#[inline(always)]
fn msg(x: u64) -> Msg {
    if x & 1 == 0 { Msg::Trade(Trade { px: x, qty: x >> 3 }) } else { Msg::Depth(Depth { lv: [x, x >> 1, x >> 2, x >> 3] }) }
}
fn init_ring(i: u64) -> [Msg; 4] {
    [Msg::Trade(Trade { px: i, qty: 1 }), Msg::Depth(Depth { lv: [i; 4] }), Msg::Trade(Trade { px: 0, qty: 0 }), Msg::Depth(Depth { lv: [0; 4] })]
}
'''
if handle:
    src += r'''
#[derive(Demux)]
pub enum MsgView<'a> { Trade(&'a Trade), Depth(&'a Depth) }
macro_rules! feed_src {
    ($F:ident, $i:expr) => {
        #[derive(ComponentSubscription, UpdateHandler)]
        #[subscribe(x: X)]
        #[handle]
        pub struct $F { ring: [Msg; 4], n: u32 }
        impl HandleComponent for $F {
            type Config = ();
            type Handle = u32;
            type View<'a> = MsgView<'a>;
            fn new((): (), _: Self::Init<'_>) -> (Self, u32) { ($F { ring: init_ring($i), n: 0 }, 0) }
            #[inline(always)]
            fn update(&mut self, i: Self::Input<'_>) -> u32 {
                self.n = self.n.wrapping_add(1);
                let slot = self.n & 3;
                self.ring[slot as usize] = msg(i.x.0.wrapping_add($i));
                slot
            }
            #[inline(always)]
            fn view<'a>(&'a self, h: &'a u32) -> MsgView<'a> {
                match &self.ring[(*h & 3) as usize] { Msg::Trade(t) => MsgView::Trade(t), Msg::Depth(d) => MsgView::Depth(d) }
            }
        }
    };
}
'''
    sel_t, sel_d = "MsgView::Trade", "MsgView::Depth"
else:
    src += r'''
macro_rules! feed_src {
    ($F:ident, $i:expr) => {
        #[derive(ComponentSubscription, UpdateHandler)]
        #[subscribe(x: X)]
        pub struct $F;
        impl Component for $F {
            type Config = ();
            type Output = Msg;
            fn new((): (), _: Self::Init<'_>) -> (Self, Msg) { ($F, msg(0)) }
            #[inline(always)]
            fn update(&mut self, i: Self::Input<'_>) -> Msg { msg(i.x.0.wrapping_add($i)) }
        }
    };
}
'''
    sel_t, sel_d = "Msg::Trade", "Msg::Depth"

src += r'''
macro_rules! feed {
    ($F:ident, $TA:ident, $DA:ident, $S:ident, $i:expr) => {
        feed_src!($F, $i);
        #[derive(ComponentSubscription, UpdateHandler)]
        #[subscribe(''' + sel_t + r'''(t): $F)]
        pub struct $TA(u64);
        impl Component for $TA {
            type Config = ();
            type Output = u64;
            fn new((): (), _: Self::Init<'_>) -> (Self, u64) { ($TA(0), 0) }
            #[inline(always)]
            fn update(&mut self, i: Self::Input<'_>) -> u64 {
                self.0 = self.0.wrapping_add(i.t.px.wrapping_mul(i.t.qty));
                if DIRECT { OUT_FW[$i * 2].store(self.0, Relaxed); }
                self.0
            }
        }
        #[derive(ComponentSubscription, UpdateHandler)]
        #[subscribe(''' + sel_d + r'''(d): $F)]
        pub struct $DA(u64);
        impl Component for $DA {
            type Config = ();
            type Output = u64;
            fn new((): (), _: Self::Init<'_>) -> (Self, u64) { ($DA(0), 0) }
            #[inline(always)]
            fn update(&mut self, i: Self::Input<'_>) -> u64 {
                self.0 = self.0.wrapping_add(i.d.lv[0] ^ i.d.lv[3]);
                if DIRECT { OUT_FW[$i * 2 + 1].store(self.0, Relaxed); }
                self.0
            }
        }
        #[derive(ComponentSubscription, UpdateHandler)]
        #[subscribe(ta: $TA, da: $DA)]
        pub struct $S;
        impl Component for $S {
            type Config = ();
            type Output = u64;
            fn new((): (), _: Self::Init<'_>) -> (Self, u64) { ($S, 0) }
            #[inline(always)]
            fn update(&mut self, i: Self::Input<'_>) -> u64 { let v = i.ta.wrapping_add(*i.da); OUT_FW[$i].store(v, Relaxed); v }
        }
    };
}
'''
src += f"const DIRECT: bool = {'true' if direct else 'false'};\n"
for i in range(n):
    src += f"feed!(F{i}, TA{i}, DA{i}, S{i}, {i});\n"
if direct:
    sinks = ", ".join(f"TA{i}, DA{i}" for i in range(n))
else:
    sinks = ", ".join(f"S{i}" for i in range(n))
src += f"type Sinks = ({sinks},);\n"

src += r'''
// ---- hand-written ----
const F: usize = ''' + str(n) + r''';
pub struct HFeed { ring: [Msg; 4], n: u32 }
pub struct Hand { feeds: [HFeed; F], ta: [u64; F], da: [u64; F], ot: [u64; F], od: [u64; F] }
impl Hand {
    fn new() -> Self { Hand { feeds: std::array::from_fn(|i| HFeed { ring: init_ring(i as u64), n: 0 }), ta: [0; F], da: [0; F], ot: [0; F], od: [0; F] } }
    #[inline(never)]
    pub fn push(&mut self, x: u64) {
        for i in 0..F {
            let f = &mut self.feeds[i];
            f.n = f.n.wrapping_add(1);
            let slot = f.n & 3;
            f.ring[slot as usize] = msg(x.wrapping_add(i as u64));
            match &f.ring[slot as usize] {
                Msg::Trade(t) => { self.ta[i] = self.ta[i].wrapping_add(t.px.wrapping_mul(t.qty)); self.ot[i] = self.ta[i]; if DIRECT { OUT_H[i * 2].store(self.ta[i], Relaxed); } }
                Msg::Depth(d) => { self.da[i] = self.da[i].wrapping_add(d.lv[0] ^ d.lv[3]); self.od[i] = self.da[i]; if DIRECT { OUT_H[i * 2 + 1].store(self.da[i], Relaxed); } }
            }
            if !DIRECT { OUT_H[i].store(self.ot[i].wrapping_add(self.od[i]), Relaxed); }
        }
    }
}

#[inline(never)]
fn push_fw(p: &mut Pipeline<X, Sinks>, x: u64) { p.push(X(x)); }

fn time<F2: FnMut(u64)>(mut f: F2, iters: u64) -> f64 {
    let t = Instant::now();
    for i in 0..iters { f(std::hint::black_box(i)); }
    t.elapsed().as_nanos() as f64 / iters as f64
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let iters: u64 = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(200_000);
    let rounds: usize = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(9);
    let (mut h0, mut p0) = (Hand::new(), Pipeline::<X, Sinks>::build(Configs::new()).unwrap());
    for x in 0..1000u64 { h0.push(x); push_fw(&mut p0, x); }
    let sf: u64 = OUT_FW.iter().fold(0u64, |a, b| a.wrapping_add(b.load(Relaxed)));
    let sh: u64 = OUT_H.iter().fold(0u64, |a, b| a.wrapping_add(b.load(Relaxed)));
    assert_eq!(sf, sh, "framework and hand-written disagree");
    let mut p = Pipeline::<X, Sinks>::build(Configs::new()).unwrap();
    let mut h = Hand::new();
    let mut best = [f64::MAX; 2];
    for _ in 0..rounds {
        best[0] = best[0].min(time(|x| h.push(x), iters));
        best[1] = best[1].min(time(|x| push_fw(&mut p, x), iters));
    }
    println!("{} feeds, checksums equal ({sf})", F);
    println!("hand-written {:8.2} ns/push", best[0]);
    println!("framework    {:8.2} ns/push", best[1]);
}
'''
open(out + "/src/main.rs", "w").write(src)
