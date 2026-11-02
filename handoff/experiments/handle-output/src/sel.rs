//! Selectors on handle components: (a) "rewrap" (a selector maps a view to a view) against
//! (b) "separate trait" (the parent implements `ViewOf<Edge>` per selected edge, like
//! `UpdateHandler<Edge>`), against hand-written code. 32 feeds, a trade and a depth consumer each,
//! dispatched on the key (what `Switch` / `PG_RUNSW` do). Safe code only.
#![forbid(unsafe_code)]
use std::time::Instant;

pub struct Trade { px: u64, qty: u64 }
pub struct Depth { lv: [u64; 4] }
pub enum Msg { Trade(Trade), Depth(Depth) }

/// The handle component: state is a ring of messages, the handle is the slot of the last one.
pub struct Feed { ring: [Msg; 4], n: u32 }
impl Feed {
    fn new(i: u64) -> Self { Feed { ring: [Msg::Trade(Trade { px: i, qty: 1 }), Msg::Depth(Depth { lv: [i; 4] }), Msg::Trade(Trade { px: 0, qty: 0 }), Msg::Depth(Depth { lv: [0; 4] })], n: 0 } }
    #[inline(always)]
    fn update(&mut self, x: u64) -> u32 {
        self.n = self.n.wrapping_add(1);
        let slot = self.n & 3;
        self.ring[slot as usize] = if x & 1 == 0 { Msg::Trade(Trade { px: x, qty: x >> 3 }) } else { Msg::Depth(Depth { lv: [x, x >> 1, x >> 2, x >> 3] }) };
        slot
    }
    #[inline(always)]
    fn output<'a>(&'a self, h: &'a u32) -> &'a Msg { &self.ring[(*h & 3) as usize] }
}
#[derive(Default)] pub struct TA(u64);
#[derive(Default)] pub struct DA(u64);
impl TA { #[inline(always)] fn update(&mut self, t: &Trade) -> u64 { self.0 = self.0.wrapping_add(t.px.wrapping_mul(t.qty)); self.0 } }
impl DA { #[inline(always)] fn update(&mut self, d: &Depth) -> u64 { self.0 = self.0.wrapping_add(d.lv[0] ^ d.lv[3]); self.0 } }

// ---------------- (a) rewrap: `Select` over views -------------------------------------------
pub trait SelectV {
    type In<'a>;
    type Out<'a>;
    const KEY: u32;
    fn select<'a>(x: Self::In<'a>) -> Option<Self::Out<'a>>;
}
/// One family (an enum's variants): the key of a view.
pub trait FamilyV { type In<'a>; fn key_of(x: &Self::In<'_>) -> u32; }
pub struct SelTrade; pub struct SelDepth; pub struct MsgFam;
impl SelectV for SelTrade { type In<'a> = &'a Msg; type Out<'a> = &'a Trade; const KEY: u32 = 0;
    #[inline(always)] fn select<'a>(x: Self::In<'a>) -> Option<Self::Out<'a>> { if let Msg::Trade(t) = x { Some(t) } else { None } } }
impl SelectV for SelDepth { type In<'a> = &'a Msg; type Out<'a> = &'a Depth; const KEY: u32 = 1;
    #[inline(always)] fn select<'a>(x: Self::In<'a>) -> Option<Self::Out<'a>> { if let Msg::Depth(t) = x { Some(t) } else { None } } }
impl FamilyV for MsgFam { type In<'a> = &'a Msg;
    #[inline(always)] fn key_of(x: &&Msg) -> u32 { match x { Msg::Trade(_) => 0, Msg::Depth(_) => 1 } } }

// ---------------- (b) a trait per edge, on the parent ---------------------------------------
pub trait ViewOf<Edge> { type Out<'a> where Self: 'a; const KEY: u32;
    fn view<'a>(&'a self, h: &'a u32) -> Option<Self::Out<'a>>; }
pub trait Keyed { fn key(&self, h: &u32) -> u32; }
pub struct ETrade; pub struct EDepth;
impl ViewOf<ETrade> for Feed { type Out<'a> = &'a Trade; const KEY: u32 = 0;
    #[inline(always)] fn view<'a>(&'a self, h: &'a u32) -> Option<&'a Trade> { if let Msg::Trade(t) = self.output(h) { Some(t) } else { None } } }
impl ViewOf<EDepth> for Feed { type Out<'a> = &'a Depth; const KEY: u32 = 1;
    #[inline(always)] fn view<'a>(&'a self, h: &'a u32) -> Option<&'a Depth> { if let Msg::Depth(t) = self.output(h) { Some(t) } else { None } } }
impl Keyed for Feed { #[inline(always)] fn key(&self, h: &u32) -> u32 { match self.output(h) { Msg::Trade(_) => 0, Msg::Depth(_) => 1 } } }

const F: usize = 32;
pub struct P { feeds: [Feed; F], hs: [u32; F], ta: [TA; F], da: [DA; F], ot: [u64; F], od: [u64; F] }
impl P {
    fn new() -> Self { P { feeds: std::array::from_fn(|i| Feed::new(i as u64)), hs: [0; F], ta: Default::default(), da: Default::default(), ot: [0; F], od: [0; F] } }
    fn sum(&self) -> u64 { self.ot.iter().chain(self.od.iter()).fold(0u64, |a, b| a.wrapping_add(*b)) }

    /// hand-written
    #[inline(never)]
    pub fn push_hand(&mut self, x: u64) {
        for i in 0..F {
            let h = self.feeds[i].update(x.wrapping_add(i as u64)); self.hs[i] = h;
            match &self.feeds[i].ring[(h & 3) as usize] {
                Msg::Trade(t) => self.ot[i] = self.ta[i].update(t),
                Msg::Depth(d) => self.od[i] = self.da[i].update(d),
            }
        }
    }
    /// (a) rewrap
    #[inline(never)]
    pub fn push_a(&mut self, x: u64) {
        for i in 0..F {
            let h = self.feeds[i].update(x.wrapping_add(i as u64)); self.hs[i] = h;
            let v = self.feeds[i].output(&self.hs[i]);
            match <MsgFam as FamilyV>::key_of(&v) {
                SelTrade::KEY => self.ot[i] = self.ta[i].update(SelTrade::select(v).expect("key matched")),
                SelDepth::KEY => self.od[i] = self.da[i].update(SelDepth::select(v).expect("key matched")),
                _ => {}
            }
        }
    }
    /// (b) per-edge trait
    #[inline(never)]
    pub fn push_b(&mut self, x: u64) {
        for i in 0..F {
            let h = self.feeds[i].update(x.wrapping_add(i as u64)); self.hs[i] = h;
            let f = &self.feeds[i];
            match f.key(&self.hs[i]) {
                <Feed as ViewOf<ETrade>>::KEY => self.ot[i] = self.ta[i].update(<Feed as ViewOf<ETrade>>::view(f, &self.hs[i]).expect("key matched")),
                <Feed as ViewOf<EDepth>>::KEY => self.od[i] = self.da[i].update(<Feed as ViewOf<EDepth>>::view(f, &self.hs[i]).expect("key matched")),
                _ => {}
            }
        }
    }
}

fn time<F2: FnMut(u64)>(mut f: F2, iters: u64) -> f64 {
    let t = Instant::now();
    for i in 0..iters { f(std::hint::black_box(i)); }
    t.elapsed().as_nanos() as f64 / iters as f64
}
fn main() {
    let iters = 200_000;
    let (mut h, mut a, mut b) = (P::new(), P::new(), P::new());
    let mut best = [f64::MAX; 3];
    for _ in 0..9 {
        best[0] = best[0].min(time(|x| h.push_hand(x), iters));
        best[1] = best[1].min(time(|x| a.push_a(x), iters));
        best[2] = best[2].min(time(|x| b.push_b(x), iters));
    }
    println!("checksums hand {} a {} b {}", h.sum(), a.sum(), b.sum());
    for (n, v) in ["hand-written", "(a) rewrap", "(b) per-edge trait"].iter().zip(best) { println!("{n:22} {v:8.1} ns/push (32 feeds)"); }
}
