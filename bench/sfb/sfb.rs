// Store-forwarding repro: field-wise update then copy the struct, vs build a local then store it.
// build: rustc -C opt-level=3 bench/sfb/sfb.rs -o sfb && taskset -c 2 ./sfb
//
// Result (rustc 1.101.0-nightly, LLVM 23.1.4, x86-64 Xeon 2.1 GHz VM, best of 5 x 50M calls):
//   field 16.3-17.3 ns/call, local 7.9-8.9 ns/call (the consumer reads out.bid, i.e. the copied half).
//   -C llvm-args=-x86-disable-avoid-SFB: no change (16.5 / 8.3). -C target-cpu=native: 15.7 / 7.8.
// `field` stores scalars into c.top in branch blocks and copies c.top after the join:
//     movq %rcx, 64(%rdi)     ; or  movq %rcx, 72(%rdi) / movq %r8, 88(%rdi)   (blocks .LBB6_2/.LBB6_4)
//   .LBB6_5:
//     movups 64(%rdi), %xmm0  ; 16-byte load spanning the two 8-byte stores: store forwarding fails
//     movq 80(%rdi), %rax ... ; movups %xmm0, (%rsi)
// The 16-byte load cannot be forwarded from two narrower in-flight stores (~17 cycles). X86AvoidSFB only
// looks for the preceding store within the load's own basic block, so it never fires here, and SROA cannot
// split the memcpy because the stores are conditional. `local` keeps the fields in registers (8-byte
// stores only), so there is no wide load. Missed optimisation in LLVM, not a rustc bug as such.
use std::hint::black_box;
use std::time::Instant;

#[derive(Clone, Copy, Default)]
pub struct Top { bid: u64, ask: u64, bq: u64, aq: u64 }

#[derive(Default)]
pub struct Core { top: Top, ladder: [u64; 8], last: u32 }

// A: scalar stores into c.top, then the output is a copy of c.top (a 32-byte memcpy after the join).
#[inline(never)]
pub fn field(c: &mut Core, out: &mut Top, seq: u32, px: u64, qty: u64, bid: bool) {
    if seq != c.last {
        c.last = seq;
        let i = (px & 7) as usize;
        c.ladder[i] = c.ladder[i].wrapping_add(qty);
        if bid { c.top.bid = px; c.top.bq = qty } else { c.top.ask = px; c.top.aq = qty }
        c.top.bq = c.top.bq.wrapping_add(c.ladder[(i + 1) & 7] >> 3);
    }
    *out = c.top;
}

// B: build a local, store it once into the core, and return from the local.
#[inline(never)]
pub fn local(c: &mut Core, out: &mut Top, seq: u32, px: u64, qty: u64, bid: bool) {
    if seq != c.last {
        c.last = seq;
        let i = (px & 7) as usize;
        c.ladder[i] = c.ladder[i].wrapping_add(qty);
        let mut t = c.top;
        if bid { t.bid = px; t.bq = qty } else { t.ask = px; t.aq = qty }
        t.bq = t.bq.wrapping_add(c.ladder[(i + 1) & 7] >> 3);
        c.top = t;
        *out = t;
        return;
    }
    *out = c.top;
}

type F = fn(&mut Core, &mut Top, u32, u64, u64, bool);

fn bench(name: &str, f: F) {
    let (mut c, mut out, mut s) = (Core::default(), Top::default(), 0x9E37u64);
    let n = 50_000_000u64;
    let mut best = f64::MAX;
    for r in 0..5 {
        let t = Instant::now();
        for k in 0..n {
            s ^= s << 13; s ^= s >> 7; s ^= s << 17;
            f(&mut c, &mut out, (r * n + k) as u32 + 1, s >> 40, (s & 63) + 1, s & 0x100 != 0);
            s ^= black_box(out.bid ^ out.ask) & 1; // the consumer reads the copy
        }
        best = best.min(t.elapsed().as_nanos() as f64 / n as f64);
    }
    println!("{name:6} {best:.2} ns/call");
}

fn main() {
    bench("field", field);
    bench("local", local);
}
