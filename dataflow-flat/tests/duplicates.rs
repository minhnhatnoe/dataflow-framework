//! A component may subscribe to the same dependency twice through an alias
//! (`type Y = X;`, `a: X, b: Y`). It behaves exactly as `a: X, b: X` does (where
//! the derive merges the two entries): `X` is constructed once, updated once per
//! run, dropped once, and both fields are the same reference.
//!
//! Covered: `X` first reached through the duplicating component (A), or earlier
//! through another component (B); both entries in one leaf, or in different
//! nodes of a wide list; the same duplicating component under two roots.

use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, UpdateHandler, Configs, Pipeline, Root, Sinks};

#[derive(Root)]
pub struct Poll;

/// What the sinks computed in the last push.
static RESULT: AtomicUsize = AtomicUsize::new(0);
fn result() -> u64 {
    RESULT.load(SeqCst) as u64
}

static SERIAL: Mutex<()> = Mutex::new(());
static NEW: AtomicUsize = AtomicUsize::new(0);
static UPDATE: AtomicUsize = AtomicUsize::new(0);
static DROP: AtomicUsize = AtomicUsize::new(0);
/// How many `ptr::eq` checks ran, and how many of them failed.
static COMPARED: AtomicUsize = AtomicUsize::new(0);
static DIFFERENT: AtomicUsize = AtomicUsize::new(0);

fn reset() {
    for c in [&NEW, &UPDATE, &DROP, &COMPARED, &DIFFERENT] {
        c.store(0, SeqCst);
    }
}

/// The shared dependency: counts its own lifecycle. Output 7.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Poll)]
pub struct X;
impl Component for X {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        NEW.fetch_add(1, SeqCst);
        (X, 7)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u64 {
        UPDATE.fetch_add(1, SeqCst);
        7
    }
}
impl Drop for X {
    fn drop(&mut self) {
        DROP.fetch_add(1, SeqCst);
    }
}
/// Another spelling of the same type.
type Y = X;

macro_rules! leaf {
    ($name:ident, $n:expr) => {
        #[derive(ComponentSubscription, UpdateHandler)]
        #[subscribe(p: Poll)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
                ($name, $n)
            }
            fn update(&mut self, _: Self::Input<'_>) -> u64 {
                $n
            }
        }
    };
}
/// Many distinct leaves, for the wide lists: output `N + 100`.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Poll)]
pub struct G<const N: usize>;
impl<const N: usize> Component for G<N> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (G, N as u64 + 100)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u64 {
        N as u64 + 100
    }
}

leaf!(Z0, 1);
leaf!(Z1, 2);
leaf!(Z2, 3);
leaf!(Z3, 4);
leaf!(Z4, 5);
leaf!(Z5, 6);

/// Output: the sum of all inputs weighted by position; checks that the listed
/// fields are all the same reference.
macro_rules! dup {
    ($name:ident, [$($f:ident: $d:ty),*], same: [$first:ident $(, $rest:ident)+]) => {
        #[derive(ComponentSubscription, UpdateHandler)]
        #[subscribe($($f: $d),*)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
                ($name, 0)
            }
            fn update(&mut self, input: Self::Input<'_>) -> u64 {
                $(
                    COMPARED.fetch_add(1, SeqCst);
                    if !std::ptr::eq(input.$first, input.$rest) {
                        DIFFERENT.fetch_add(1, SeqCst);
                    }
                )+
                let mut acc: u64 = 0;
                let mut w: u64 = 1;
                $(acc = acc.wrapping_add(w.wrapping_mul(*input.$f)); w = w.wrapping_mul(10);)*
                let _ = w;
                acc
            }
        }
    };
}

// Both entries in one leaf; `c: X` is spelled like `a` and merged by the derive.
dup!(DupLeaf, [a: X, m: Z0, b: Y, n: Z1, c: X], same: [a, b, c]);
// 8 unique subscriptions. Fill-first puts the first 7 in child 0 and the 8th
// in child 1 of an inner node, so `a` (index 1) and `b` (index 7) are in
// different nodes.
dup!(DupWide, [s0: Z0, a: X, s1: Z1, s2: Z2, s3: Z3, s4: Z4, s5: Z5, b: Y], same: [a, b]);
// 70 unique subscriptions (capacity 56 at depth 1, so depth 2): the root node's
// child 0 holds indices 0..56 (an inner node with 8 children of 7), child 1
// indices 56..70. `a` (index 5) is in the first leaf of child 0, `b` (index 69)
// in a leaf of child 1: different nodes at every level.
dup!(
    DupDeep,
    [
    g0: G<0>, g1: G<1>, g2: G<2>, g3: G<3>, g4: G<4>, a: X, g5: G<5>, g6: G<6>, g7: G<7>,
     g8: G<8>, g9: G<9>, g10: G<10>, g11: G<11>, g12: G<12>, g13: G<13>, g14: G<14>,
     g15: G<15>, g16: G<16>, g17: G<17>, g18: G<18>, g19: G<19>, g20: G<20>, g21: G<21>,
     g22: G<22>, g23: G<23>, g24: G<24>, g25: G<25>, g26: G<26>, g27: G<27>, g28: G<28>,
     g29: G<29>, g30: G<30>, g31: G<31>, g32: G<32>, g33: G<33>, g34: G<34>, g35: G<35>,
     g36: G<36>, g37: G<37>, g38: G<38>, g39: G<39>, g40: G<40>, g41: G<41>, g42: G<42>,
     g43: G<43>, g44: G<44>, g45: G<45>, g46: G<46>, g47: G<47>, g48: G<48>, g49: G<49>,
     g50: G<50>, g51: G<51>, g52: G<52>, g53: G<53>, g54: G<54>, g55: G<55>, g56: G<56>,
     g57: G<57>, g58: G<58>, g59: G<59>, g60: G<60>, g61: G<61>, g62: G<62>, g63: G<63>,
     g64: G<64>, g65: G<65>, g66: G<66>, g67: G<67>, b: Y
    ],
    same: [a, b]
);

/// Reaches X first, so that the duplicating component sees it as already done.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(x: X)]
pub struct P;
impl Component for P {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (P, 0)
    }
    fn update(&mut self, input: Self::Input<'_>) -> u64 {
        *input.x + 1000
    }
}

macro_rules! root {
    ($name:ident, [$($f:ident: $d:ty),*], $body:expr) => {
        #[derive(ComponentSubscription, UpdateHandler)]
        #[subscribe($($f: $d),*)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
                ($name, 0)
            }
            #[allow(unused_variables)]
            fn update(&mut self, input: Self::Input<'_>) -> u64 {
                let f: fn(&Self::Input<'_>) -> u64 = $body;
                let v = f(&input);
                RESULT.store(v as usize, SeqCst);
                v
            }
        }
    };
}

// Case A: X is first reached through the duplicating component.
root!(RootLeafA, [d: DupLeaf], |i| *i.d);
root!(RootWideA, [d: DupWide], |i| *i.d);
root!(RootDeepA, [d: DupDeep], |i| *i.d);
// Case B: X is reached earlier, through P (listed before the duplicating one).
root!(RootLeafB, [p: P, d: DupLeaf], |i| *i.p * 1_000_000 + *i.d);
root!(RootWideB, [p: P, d: DupWide], |i| *i.p * 1_000_000 + *i.d);
root!(RootDeepB, [p: P, d: DupDeep], |i| *i.p * 1_000_000 + *i.d);
// Case B': the duplicating component is listed first, P later (X is first
// reached through the duplicating one, and P's edge is not the first).
root!(RootWideC, [d: DupWide, p: P], |i| *i.p * 1_000_000 + *i.d);

/// X was built once, is updated `runs` times, and was dropped once with the
/// pipeline; every identity check passed.
fn check<A: 'static>(expected: u64, checks_per_run: usize)
where
    (A,): Sinks<Poll>,
{
    let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    {
        let mut p = Pipeline::<Poll, (A,)>::build(Configs::new()).unwrap();
        assert_eq!(NEW.load(SeqCst), 1, "X constructed once");
        for run in 1..=3 {
            p.push(Poll);
            assert_eq!(result(), expected);
            assert_eq!(UPDATE.load(SeqCst), run, "X updated once per run");
            assert_eq!(COMPARED.load(SeqCst), run * checks_per_run);
        }
        assert_eq!(DROP.load(SeqCst), 0);
    }
    assert_eq!(DROP.load(SeqCst), 1, "X dropped once");
    assert_eq!(DIFFERENT.load(SeqCst), 0, "both fields are the same reference");
}

// Expected outputs: the inputs weighted by 10^position in field order (see `dup!`).
// DupLeaf (a, m, b, n, c = 7, 1, 7, 2, 7).
const LEAF: u64 = 7 + 10 + 700 + 2000 + 70000;
// DupWide (s0, a, s1, s2, s3, s4, s5, b = 1, 7, 2, 3, 4, 5, 6, 7).
const WIDE: u64 = 1 + 70 + 200 + 3000 + 40000 + 500_000 + 6_000_000 + 70_000_000;
fn deep() -> u64 {
    // The field order of `DupDeep`, weighted 10^position (wrapping, like `dup!`).
    let g = |n: u64| n + 100;
    let mut vals = vec![g(0), g(1), g(2), g(3), g(4), 7];
    vals.extend((5..=67).map(g));
    vals.push(7);
    let (mut acc, mut w) = (0u64, 1u64);
    for v in vals {
        acc = acc.wrapping_add(w.wrapping_mul(v));
        w = w.wrapping_mul(10);
    }
    acc
}

#[test]
fn a_one_leaf() {
    check::<RootLeafA>(LEAF, 2);
}
#[test]
fn a_across_nodes() {
    check::<RootWideA>(WIDE, 1);
    check::<RootDeepA>(deep(), 1);
}
#[test]
fn b_one_leaf() {
    check::<RootLeafB>(1007 * 1_000_000 + LEAF, 2);
}
#[test]
fn b_across_nodes() {
    check::<RootWideB>(1007 * 1_000_000 + WIDE, 1);
    check::<RootDeepB>(1007 * 1_000_000 + deep(), 1);
}
#[test]
fn x_reached_through_duplicating_component_first() {
    check::<RootWideC>(1007 * 1_000_000 + WIDE, 1);
}

/// The same duplicating components under two roots (one per case), both
/// pipelines alive at once: each has its own X, built and updated once.
#[test]
fn same_component_under_two_roots() {
    let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    {
        let mut a = Pipeline::<Poll, (RootLeafA,)>::build(Configs::new()).unwrap();
        let mut b = Pipeline::<Poll, (RootLeafB,)>::build(Configs::new()).unwrap();
        let mut c = Pipeline::<Poll, (RootWideA,)>::build(Configs::new()).unwrap();
        let mut d = Pipeline::<Poll, (RootWideB,)>::build(Configs::new()).unwrap();
        assert_eq!(NEW.load(SeqCst), 4, "one X per pipeline");
        a.push(Poll);
        assert_eq!(result(), LEAF);
        b.push(Poll);
        assert_eq!(result(), 1007 * 1_000_000 + LEAF);
        c.push(Poll);
        assert_eq!(result(), WIDE);
        d.push(Poll);
        assert_eq!(result(), 1007 * 1_000_000 + WIDE);
        assert_eq!(UPDATE.load(SeqCst), 4, "one update of X per pipeline per run");
        b.push(Poll);
        assert_eq!(result(), 1007 * 1_000_000 + LEAF);
        assert_eq!(UPDATE.load(SeqCst), 5);
    }
    assert_eq!(DROP.load(SeqCst), 4);
    assert_eq!(COMPARED.load(SeqCst), 2 + 2 + 1 + 1 + 2);
    assert_eq!(DIFFERENT.load(SeqCst), 0);
}
