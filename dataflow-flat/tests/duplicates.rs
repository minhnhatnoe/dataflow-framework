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

use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline};

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
#[derive(ComponentSubscription)]
pub struct X;
impl Component for X {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self {
        NEW.fetch_add(1, SeqCst);
        X
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
        #[derive(ComponentSubscription)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): ()) -> Self {
                $name
            }
            fn update(&mut self, _: Self::Input<'_>) -> u64 {
                $n
            }
        }
    };
}
/// Many distinct leaves, for the wide lists: output `N + 100`.
#[derive(ComponentSubscription)]
pub struct G<const N: usize>;
impl<const N: usize> Component for G<N> {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self {
        G
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

/// Output: the sum of all inputs weighted by position; checks that the listed
/// fields are all the same reference.
macro_rules! dup {
    ($name:ident, [$($f:ident: $d:ty),*], same: [$first:ident $(, $rest:ident)+]) => {
        #[derive(ComponentSubscription)]
        #[subscribe($($f: $d),*)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): ()) -> Self {
                $name
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
// 7 unique subscriptions: `a` is in the node, `b` in a child.
dup!(DupWide, [s0: Z0, a: X, s1: Z1, s2: Z2, s3: Z3, b: Y, s4: Z4], same: [a, b]);
// 22 unique subscriptions: X and Y in two different children of an inner node
// that has inner children.
dup!(
    DupDeep,
    [g0: G<0>, g1: G<1>, g2: G<2>, g3: G<3>, g4: G<4>, a: X, g5: G<5>, g6: G<6>, g7: G<7>,
     g8: G<8>, g9: G<9>, g10: G<10>, g11: G<11>, g12: G<12>, g13: G<13>, g14: G<14>,
     g15: G<15>, g16: G<16>, g17: G<17>, g18: G<18>, g19: G<19>, b: Y],
    same: [a, b]
);

/// Reaches X first, so that the duplicating component sees it as already done.
#[derive(ComponentSubscription)]
#[subscribe(x: X)]
pub struct P;
impl Component for P {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self {
        P
    }
    fn update(&mut self, input: Self::Input<'_>) -> u64 {
        *input.x + 1000
    }
}

macro_rules! root {
    ($name:ident, [$($f:ident: $d:ty),*], $body:expr) => {
        #[derive(ComponentSubscription)]
        #[subscribe($($f: $d),*)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): ()) -> Self {
                $name
            }
            #[allow(unused_variables)]
            fn update(&mut self, input: Self::Input<'_>) -> u64 {
                let f: fn(&Self::Input<'_>) -> u64 = $body;
                f(&input)
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
fn check<A: dataflow_flat::Root<Output = u64>>(expected: u64, checks_per_run: usize) {
    let _guard = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    {
        let mut p = Pipeline::<A>::build(Configs::new()).unwrap();
        assert_eq!(NEW.load(SeqCst), 1, "X constructed once");
        for run in 1..=3 {
            assert_eq!(p.run(), expected);
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
// DupWide (s0, a, s1, s2, s3, b, s4 = 1, 7, 2, 3, 4, 7, 5).
const WIDE: u64 = 1 + 70 + 200 + 3000 + 40000 + 700000 + 5_000_000;
fn deep() -> u64 {
    // The field order of `DupDeep`, weighted 10^position (wrapping, like `dup!`).
    let g = |n: u64| n + 100;
    let mut vals = vec![g(0), g(1), g(2), g(3), g(4), 7];
    vals.extend((5..=19).map(g));
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
        let mut a = Pipeline::<RootLeafA>::build(Configs::new()).unwrap();
        let mut b = Pipeline::<RootLeafB>::build(Configs::new()).unwrap();
        let mut c = Pipeline::<RootWideA>::build(Configs::new()).unwrap();
        let mut d = Pipeline::<RootWideB>::build(Configs::new()).unwrap();
        assert_eq!(NEW.load(SeqCst), 4, "one X per pipeline");
        assert_eq!(a.run(), LEAF);
        assert_eq!(b.run(), 1007 * 1_000_000 + LEAF);
        assert_eq!(c.run(), WIDE);
        assert_eq!(d.run(), 1007 * 1_000_000 + WIDE);
        assert_eq!(UPDATE.load(SeqCst), 4, "one update of X per pipeline per run");
        assert_eq!(b.run(), 1007 * 1_000_000 + LEAF);
        assert_eq!(UPDATE.load(SeqCst), 5);
    }
    assert_eq!(DROP.load(SeqCst), 4);
    assert_eq!(COMPARED.load(SeqCst), 2 + 2 + 1 + 1 + 2);
    assert_eq!(DIFFERENT.load(SeqCst), 0);
}
