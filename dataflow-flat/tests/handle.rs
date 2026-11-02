//! Handle components (`#[handle]` + `impl HandleComponent`): the pipeline stores a handle and a
//! subscriber's field is the component's `View<'a>` itself, owned or borrowed, computed from the
//! parent's state each time the subscriber runs or is constructed. Checks values, the view of a
//! parent that did not run in this step, a wide subscriber, an aliased parent, mixed by-value and
//! handle parents, lifecycle counts, execution order and generic user code. No `#![feature]` here.

use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, UpdateHandler, Configs, Demux, HandleComponent, Node, Pipeline, Root, Sinks};

static SERIAL: Mutex<()> = Mutex::new(());
static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static NEW: AtomicUsize = AtomicUsize::new(0);
static UPDATE: AtomicUsize = AtomicUsize::new(0);
static DROP: AtomicUsize = AtomicUsize::new(0);

fn log(s: impl Into<String>) {
    LOG.lock().unwrap().push(s.into());
}
fn take() -> Vec<String> {
    std::mem::take(&mut *LOG.lock().unwrap())
}
fn counts() -> (usize, usize, usize) {
    (NEW.load(SeqCst), UPDATE.load(SeqCst), DROP.load(SeqCst))
}
fn reset() {
    take();
    for c in [&NEW, &UPDATE, &DROP] {
        c.store(0, SeqCst);
    }
}

struct Life;
impl Life {
    fn new() -> Life {
        NEW.fetch_add(1, SeqCst);
        Life
    }
}
impl Drop for Life {
    fn drop(&mut self) {
        DROP.fetch_add(1, SeqCst);
    }
}

/// The root: `A` events feed the ring, `B` events do not.
#[derive(Clone, Copy, Root, Demux)]
pub enum Msg {
    A(u64),
    B(u64),
}
fn val(m: &Msg) -> u64 {
    match m {
        Msg::A(v) | Msg::B(v) => *v,
    }
}

// ---------------------------------------------------------------------------
// Handle components
// ---------------------------------------------------------------------------

/// A ring of four values; the handle is the index of the last one written; the view is a
/// reference into the state. Runs on `A` events only.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Msg::A(v): Msg)]
#[handle]
pub struct Ring {
    arr: [u64; 4],
    n: usize,
    _life: Life,
}
impl HandleComponent for Ring {
    type Config = u64;
    type Handle = usize;
    type View<'a> = &'a u64;
    fn new(start: u64, _: Self::Init<'_>) -> (Self, usize) {
        log("new Ring");
        (Ring { arr: [start; 4], n: 0, _life: Life::new() }, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> usize {
        log("update Ring");
        UPDATE.fetch_add(1, SeqCst);
        self.n += 1;
        let slot = self.n % 4;
        self.arr[slot] = *i.v;
        slot
    }
    fn view<'a>(&'a self, h: &'a usize) -> &'a u64 {
        &self.arr[*h]
    }
}

/// `Handle = ()`: the whole state is the value; the view is owned.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
#[handle]
pub struct Level {
    v: u64,
    _life: Life,
}
impl HandleComponent for Level {
    type Config = ();
    type Handle = ();
    type View<'a> = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        log("new Level");
        (Level { v: 7, _life: Life::new() }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        log("update Level");
        UPDATE.fetch_add(1, SeqCst);
        self.v += val(i.m);
    }
    fn view<'a>(&'a self, (): &'a ()) -> u64 {
        self.v
    }
}

/// A by-value component.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct Twice(Life);
impl Component for Twice {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        log("new Twice");
        (Twice(Life::new()), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log("update Twice");
        UPDATE.fetch_add(1, SeqCst);
        val(i.m) * 2
    }
}

/// Mixed parents, in this listing order: a handle with a borrowed view, a by-value component, a
/// handle with an owned view, the root. Its `new` gets the views of the constructed parents.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(r: Ring, t: Twice, l: Level, m: Msg)]
pub struct Mixed(Life);
static NEW_SEEN: Mutex<Vec<(u64, u64, u64)>> = Mutex::new(Vec::new());
static OUT: AtomicUsize = AtomicUsize::new(0);
impl Component for Mixed {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) {
        log("new Mixed");
        // `i.r` is `&u64` (the ring's view), `i.t` is `&u64`, `i.l` is `u64`.
        let r: &u64 = i.r;
        let t: &u64 = i.t;
        let l: u64 = i.l;
        NEW_SEEN.lock().unwrap().push((*r, *t, l));
        (Mixed(Life::new()), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log("update Mixed");
        UPDATE.fetch_add(1, SeqCst);
        let r: &u64 = i.r;
        let l: u64 = i.l;
        let out = *r * 1_000_000 + *i.t * 1_000 + l;
        OUT.store(out as usize, SeqCst);
        out
    }
}

fn out() -> u64 {
    OUT.load(SeqCst) as u64
}

#[test]
fn mixed_parents_owned_and_borrowed_views() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    NEW_SEEN.lock().unwrap().clear();
    let mut p = Pipeline::<Msg, (Mixed,)>::build(Configs::new().with::<Ring>(5)).unwrap();
    // `new` saw the views of the constructed parents: ring start (5), twice 0, level 7.
    assert_eq!(*NEW_SEEN.lock().unwrap(), vec![(5, 0, 7)]);
    // The construction order is the execution order: Ring, Twice, Level, Mixed.
    assert_eq!(p.order().len(), 4);
    take();
    p.push(Msg::A(3));
    // ring slot 1 = 3; twice = 6; level = 7 + 3.
    assert_eq!(out(), 3 * 1_000_000 + 6 * 1_000 + 10);
    p.push(Msg::A(4));
    assert_eq!(out(), 4 * 1_000_000 + 8 * 1_000 + 14);
}

#[test]
fn a_parent_that_did_not_run_gives_the_same_view() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut p = Pipeline::<Msg, (Mixed,)>::build(Configs::new()).unwrap();
    take();
    p.push(Msg::A(9));
    assert_eq!(take(), ["update Ring", "update Twice", "update Level", "update Mixed"]);
    assert_eq!(out(), 9 * 1_000_000 + 18 * 1_000 + 16);
    // A `B` event: Ring does not run (its edge needs `A`), Mixed still does and reads the ring's
    // stored handle and state as they were.
    p.push(Msg::B(1));
    assert_eq!(take(), ["update Twice", "update Level", "update Mixed"]);
    assert_eq!(out(), 9 * 1_000_000 + 2 * 1_000 + 17);
    p.push(Msg::B(2));
    assert_eq!(out(), 9 * 1_000_000 + 4 * 1_000 + 19);
}

#[test]
fn lifecycle_counts_and_order() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    {
        let mut p = Pipeline::<Msg, (Mixed,)>::build(Configs::new()).unwrap();
        assert_eq!(counts(), (4, 0, 0));
        assert_eq!(take(), ["new Ring", "new Twice", "new Level", "new Mixed"]);
        for k in 0..3 {
            p.push(Msg::A(k));
        }
        p.push(Msg::B(1));
        // Ring 3, Twice 4, Level 4, Mixed 4 updates; none constructed again.
        assert_eq!(counts(), (4, 15, 0));
    }
    assert_eq!(counts(), (4, 15, 4));
}

// ---------------------------------------------------------------------------
// An aliased parent, listed twice
// ---------------------------------------------------------------------------

type RingAgain = Ring;

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(a: Ring, b: RingAgain, c: Ring)]
pub struct Aliased;
static ALIASED: Mutex<Vec<(u64, u64, u64)>> = Mutex::new(Vec::new());
impl Component for Aliased {
    type Config = ();
    type Output = ();
    fn new((): (), i: Self::Init<'_>) -> (Self, ()) {
        ALIASED.lock().unwrap().push((*i.a, *i.b, *i.c));
        (Aliased, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        ALIASED.lock().unwrap().push((*i.a, *i.b, *i.c));
    }
}

#[test]
fn an_aliased_parent_is_one_component_with_one_view() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    ALIASED.lock().unwrap().clear();
    let mut p = Pipeline::<Msg, (Aliased,)>::build(Configs::new().with::<Ring>(1)).unwrap();
    assert_eq!(p.len(), 2);
    p.push(Msg::A(8));
    p.push(Msg::B(2));
    assert_eq!(*ALIASED.lock().unwrap(), vec![(1, 1, 1), (8, 8, 8)]);
    assert_eq!(counts().0, 1);
}

// ---------------------------------------------------------------------------
// A handle component that subscribes to a handle component
// ---------------------------------------------------------------------------

/// Doubles the ring's view into its own state; the view is the state.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(r: Ring, m: Msg)]
#[handle]
pub struct Doubler {
    last: u64,
    _life: Life,
}
impl HandleComponent for Doubler {
    type Config = ();
    type Handle = ();
    type View<'a> = &'a u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, ()) {
        (Doubler { last: *i.r * 2, _life: Life::new() }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        self.last = *i.r * 2 + val(i.m);
    }
    fn view<'a>(&'a self, (): &'a ()) -> &'a u64 {
        &self.last
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(d: Doubler, r: Ring)]
pub struct AfterDoubler;
static CHAIN: Mutex<Vec<(u64, u64)>> = Mutex::new(Vec::new());
impl Component for AfterDoubler {
    type Config = ();
    type Output = ();
    fn new((): (), i: Self::Init<'_>) -> (Self, ()) {
        CHAIN.lock().unwrap().push((*i.d, *i.r));
        (AfterDoubler, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        CHAIN.lock().unwrap().push((*i.d, *i.r));
    }
}

#[test]
fn a_handle_component_can_read_a_handle_component() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    CHAIN.lock().unwrap().clear();
    let mut p = Pipeline::<Msg, (AfterDoubler,)>::build(Configs::new().with::<Ring>(10)).unwrap();
    p.push(Msg::A(4));
    p.push(Msg::B(1));
    assert_eq!(*CHAIN.lock().unwrap(), vec![(20, 10), (8 + 4, 4), (8 + 1, 4)]);
}

// ---------------------------------------------------------------------------
// A wide subscriber: 30 handle parents, across the blocks of the storage tree
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
#[handle]
pub struct Cell<const K: usize> {
    v: u64,
}
impl<const K: usize> HandleComponent for Cell<K> {
    type Config = ();
    type Handle = u64;
    type View<'a> = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Cell { v: K as u64 }, 1)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        self.v += val(i.m) + K as u64;
        K as u64
    }
    fn view<'a>(&'a self, h: &'a u64) -> u64 {
        self.v * 1000 + *h
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(
    c0: Cell<0>,
    c1: Cell<1>,
    c2: Cell<2>,
    c3: Cell<3>,
    c4: Cell<4>,
    c5: Cell<5>,
    c6: Cell<6>,
    c7: Cell<7>,
    c8: Cell<8>,
    c9: Cell<9>,
    c10: Cell<10>,
    c11: Cell<11>,
    c12: Cell<12>,
    c13: Cell<13>,
    c14: Cell<14>,
    c15: Cell<15>,
    c16: Cell<16>,
    c17: Cell<17>,
    c18: Cell<18>,
    c19: Cell<19>,
    c20: Cell<20>,
    c21: Cell<21>,
    c22: Cell<22>,
    c23: Cell<23>,
    c24: Cell<24>,
    c25: Cell<25>,
    c26: Cell<26>,
    c27: Cell<27>,
    c28: Cell<28>,
    c29: Cell<29>
)]
pub struct Wide;
static WIDE: Mutex<Vec<u64>> = Mutex::new(Vec::new());
impl Component for Wide {
    type Config = ();
    type Output = ();
    fn new((): (), i: Self::Init<'_>) -> (Self, ()) {
        WIDE.lock().unwrap().push(i.c0 + i.c1 + i.c2 + i.c3 + i.c4 + i.c5 + i.c6 + i.c7 + i.c8 + i.c9 + i.c10 + i.c11 + i.c12 + i.c13 + i.c14 + i.c15 + i.c16 + i.c17 + i.c18 + i.c19 + i.c20 + i.c21 + i.c22 + i.c23 + i.c24 + i.c25 + i.c26 + i.c27 + i.c28 + i.c29);
        (Wide, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        WIDE.lock().unwrap().push(i.c0 + i.c1 + i.c2 + i.c3 + i.c4 + i.c5 + i.c6 + i.c7 + i.c8 + i.c9 + i.c10 + i.c11 + i.c12 + i.c13 + i.c14 + i.c15 + i.c16 + i.c17 + i.c18 + i.c19 + i.c20 + i.c21 + i.c22 + i.c23 + i.c24 + i.c25 + i.c26 + i.c27 + i.c28 + i.c29);
    }
}

#[test]
fn a_wide_subscriber_reads_views_across_blocks() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    WIDE.lock().unwrap().clear();
    let mut p = Pipeline::<Msg, (Wide,)>::build(Configs::new()).unwrap();
    assert_eq!(p.len(), 31);
    p.push(Msg::A(5));
    p.push(Msg::B(1));
    // Cell<K>: v starts at K, +val+K per push; view = v * 1000 + K (after a run) or + 1 (first).
    let first: u64 = (0..30).map(|k| k * 1000 + 1).sum();
    let after1: u64 = (0..30).map(|k| (k + 5 + k) * 1000 + k).sum();
    let after2: u64 = (0..30).map(|k| (k + 5 + k + 1 + k) * 1000 + k).sum();
    assert_eq!(*WIDE.lock().unwrap(), vec![first, after1, after2]);
}

// ---------------------------------------------------------------------------
// Generic user code
// ---------------------------------------------------------------------------

pub fn drive<R, S: Sinks<R>>(p: &mut Pipeline<R, S>, steps: impl IntoIterator<Item = R>) {
    for r in steps {
        p.push(r);
    }
}
pub fn make<R, S: Sinks<R>>() -> Pipeline<R, S> {
    Pipeline::build(Configs::new()).unwrap()
}

/// Generic over any by-value node with a `u64` output, written with `Node<Output = u64>` only:
/// the field is `&u64`, as before handles existed.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(x: X, l: Level)]
pub struct Generic<X: Node<Output = u64>>(std::marker::PhantomData<X>);
static GENERIC: Mutex<Vec<u64>> = Mutex::new(Vec::new());
impl<X: Node<Output = u64>> Component for Generic<X> {
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Generic(std::marker::PhantomData), ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        let x: &u64 = i.x;
        GENERIC.lock().unwrap().push(*x * 100 + i.l);
    }
}

#[test]
fn generic_code_needs_only_the_sinks_bound_and_node_output() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    GENERIC.lock().unwrap().clear();
    let mut p: Pipeline<Msg, (Generic<Twice>,)> = make();
    drive(&mut p, [Msg::A(1), Msg::B(2)]);
    assert_eq!(*GENERIC.lock().unwrap(), vec![2 * 100 + 8, 4 * 100 + 10]);
}
