#![forbid(unsafe_code)]
#![allow(clippy::all)]
use handle_proto::variants::*;
use std::time::Instant;

fn time<F: FnMut(u64)>(mut f: F, iters: u64) -> f64 {
    let t = Instant::now();
    for i in 0..iters {
        f(std::hint::black_box(i));
    }
    t.elapsed().as_nanos() as f64 / iters as f64
}

fn main() {
    let iters: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(20000);
    let rounds = 9;
    let (mut vv, mut hh, mut oo, mut mm, mut hv, mut hhh, mut ho) = (VV::new(), HH::new(), OO::new(), MM::new(), HandV::new(), HandH::new(), HandO::new());
    let names = ["VV (by value, tree)", "HandV (by value, struct)", "HH (handle &u64, tree)", "HandH (handle &u64, struct)", "OO (handle u64 owned, tree)", "HandO (owned, struct)", "MM (mixed v/h/o, tree)"];
    let mut best = [f64::MAX; 7];
    for _ in 0..rounds {
        best[0] = best[0].min(time(|x| vv.push(x), iters));
        best[1] = best[1].min(time(|x| hv.push(x), iters));
        best[2] = best[2].min(time(|x| hh.push(x), iters));
        best[3] = best[3].min(time(|x| hhh.push(x), iters));
        best[4] = best[4].min(time(|x| oo.push(x), iters));
        best[5] = best[5].min(time(|x| ho.push(x), iters));
        best[6] = best[6].min(time(|x| mm.push(x), iters));
    }
    // all variants ran the same number of pushes with the same logic: same checksums
    println!("checksums: VV {} HandV {} | HH {} HandH {} OO {} HandO {} (values differ between v and h families: handle state seeds differ) | MM {}", vv.sum(), hv.sum(), hh.sum(), hhh.sum(), oo.sum(), ho.sum(), mm.sum());
    for (n, b) in names.iter().zip(best) {
        println!("{n:34} {b:9.1} ns/push");
    }
}
