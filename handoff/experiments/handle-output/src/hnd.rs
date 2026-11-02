//! Handlers with the plain (unconditional) inputs: `on_update(&mut self, plain: Plain<'_>, v: &Value)`.
//! A handler that ignores `plain` must cost what `on_update(&mut self, v: &Value)` costs.
//! The plain inputs here: an owned view of a handle component (`limit: f64`) and a by-reference
//! view (`name: &Level`).
#![forbid(unsafe_code)]
use std::time::Instant;

pub struct Level { px: i64, qty: i64 }
pub struct Limits { arr: [f64; 4] }                 // handle component, `Output<'a> = f64` (owned)
impl Limits { #[inline(always)] fn output<'a>(&'a self, h: &'a u32) -> f64 { self.arr[(*h & 3) as usize] } }
pub struct Book { lv: [Level; 4] }                  // handle component, `Output<'a> = &'a Level`
impl Book { #[inline(always)] fn output<'a>(&'a self, h: &'a u32) -> &'a Level { &self.lv[(*h & 3) as usize] } }

pub struct Plain<'a> { pub limit: f64, pub top: &'a Level }
pub struct Oms { pending: Vec<(u64, i64)>, total: i64 }

const F: usize = 64;
pub struct P { limits: [Limits; F], lh: [u32; F], books: [Book; F], bh: [u32; F], oms: [Oms; F], quotes: [i64; F] }
impl P {
    fn new() -> Self {
        P { limits: std::array::from_fn(|i| Limits { arr: [i as f64, 2.0, 3.0, 4.0] }), lh: [1; F],
            books: std::array::from_fn(|i| Book { lv: std::array::from_fn(|j| Level { px: (i + j) as i64, qty: j as i64 }) }), bh: [2; F],
            oms: std::array::from_fn(|_| Oms { pending: Vec::with_capacity(1 << 20), total: 0 }), quotes: [0; F] }
    }
    fn sum(&self) -> i64 { self.oms.iter().map(|o| o.total).sum::<i64>() + self.oms.iter().map(|o| o.pending.len() as i64).sum::<i64>() }

    // old signature
    #[inline(always)] fn on_old(o: &mut Oms, px: &i64) { o.pending.push((1, *px)); o.total += *px; }
    // new signature, plain ignored
    #[inline(always)] fn on_new_ignored(o: &mut Oms, _plain: Plain<'_>, px: &i64) { o.pending.push((1, *px)); o.total += *px; }
    // new signature, plain used
    #[inline(always)] fn on_new_used(o: &mut Oms, plain: Plain<'_>, px: &i64) { if (*px as f64) < plain.limit * 1e9 { o.pending.push((1, *px + plain.top.qty)); } o.total += *px; }

    #[inline(never)] pub fn push_old(&mut self, x: i64) {
        for i in 0..F { self.quotes[i] = x + i as i64; Self::on_old(&mut self.oms[i], &self.quotes[i]); }
    }
    #[inline(never)] pub fn push_ignored(&mut self, x: i64) {
        for i in 0..F { self.quotes[i] = x + i as i64;
            let plain = Plain { limit: self.limits[i].output(&self.lh[i]), top: self.books[i].output(&self.bh[i]) };
            Self::on_new_ignored(&mut self.oms[i], plain, &self.quotes[i]); }
    }
    #[inline(never)] pub fn push_used(&mut self, x: i64) {
        for i in 0..F { self.quotes[i] = x + i as i64;
            let plain = Plain { limit: self.limits[i].output(&self.lh[i]), top: self.books[i].output(&self.bh[i]) };
            Self::on_new_used(&mut self.oms[i], plain, &self.quotes[i]); }
    }
}
fn time<F2: FnMut(i64)>(mut f: F2, iters: i64) -> f64 {
    let t = Instant::now();
    for i in 0..iters { f(std::hint::black_box(i)); }
    t.elapsed().as_nanos() as f64 / iters as f64
}
fn main() {
    let iters = 10_000;
    let (mut a, mut b, mut c) = (P::new(), P::new(), P::new());
    let mut best = [f64::MAX; 3];
    for _ in 0..9 {
        for o in a.oms.iter_mut().chain(b.oms.iter_mut()).chain(c.oms.iter_mut()) { o.pending.clear(); }
        best[0] = best[0].min(time(|x| a.push_old(x), iters));
        best[1] = best[1].min(time(|x| b.push_ignored(x), iters));
        best[2] = best[2].min(time(|x| c.push_used(x), iters));
    }
    println!("checksums old {} ignored {} used {}", a.sum(), b.sum(), c.sum());
    for (n, v) in ["old on_update(v)", "new, plain ignored", "new, plain used"].iter().zip(best) { println!("{n:22} {v:8.1} ns/push (64 handlers)"); }
}
