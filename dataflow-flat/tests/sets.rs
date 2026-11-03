//! Set subscriptions, `name: for S in Catalog { Node<S> }`: one field of `Input` and `Init` that
//! reads every element (`get::<S>()`, `fold`, `for_each`). Same-typed and mixed-typed sets, a set
//! beside named and unnamed entries, conditional elements with stale values, handle parents,
//! `new`, handlers, order, lifecycle, 16 and 300 elements, aliases. No `#![feature]` here.

use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::Mutex;

use dataflow_flat::{
    catalog, Component, ComponentSubscription, Concat, Configs, Initial, Pipeline, Product, Root, Select,
    UpdateHandler, Zip, M,
};

static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SERIAL: Mutex<()> = Mutex::new(());
static NEW: AtomicUsize = AtomicUsize::new(0);
static DROP: AtomicUsize = AtomicUsize::new(0);
static OUT: Mutex<Vec<u64>> = Mutex::new(Vec::new());
fn take() -> Vec<String> {
    std::mem::take(&mut *LOG.lock().unwrap())
}
fn log(s: impl Into<String>) {
    LOG.lock().unwrap().push(s.into());
}
fn out() -> u64 {
    *OUT.lock().unwrap().last().unwrap()
}
fn reset() {
    take();
    OUT.lock().unwrap().clear();
    NEW.store(0, SeqCst);
    DROP.store(0, SeqCst);
}

#[derive(Root)]
pub struct Msg {
    pub key: u32,
    pub v: u64,
}

/// Symbol number `K`.
pub struct Sy<const K: usize>;
pub trait Num: 'static {
    const N: u64;
    const KEY: u32;
}
impl<const K: usize> Num for Sy<K> {
    const N: u64 = K as u64 + 1;
    const KEY: u32 = K as u32;
}

include!("for_form_cats.inc");

/// A feed of symbol `K`: its output is `v * (K + 1)` and it runs on every push.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct Feed<S: Num>(PhantomData<S>);
impl<S: Num> Component for Feed<S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Feed(PhantomData), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        i.m.v * S::N
    }
}

macro_rules! total {
    ($name:ident, $cat:ty) => {
        #[derive(ComponentSubscription, UpdateHandler)]
        #[subscribe(feeds: for S: Num in $cat { Feed<S> })]
        pub struct $name {
            first: u64,
        }
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): (), i: Self::Init<'_>) -> (Self, u64) {
                // The values at construction are the first outputs of the feeds: 0.
                let first = i.feeds.fold(0u64, |a, v: &u64| a + *v + 1);
                ($name { first }, first)
            }
            fn update(&mut self, i: Self::Input<'_>) -> u64 {
                let sum = i.feeds.fold(0u64, |a, v| a + *v);
                OUT.lock().unwrap().push(sum);
                sum
            }
        }
    };
}

total!(Total16, Cat16);
total!(Total300, Cat300);

#[test]
fn fold_16_and_300() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut p = Pipeline::<Msg, (Total16,)>::build(Configs::new()).unwrap();
    p.push(Msg { key: 0, v: 2 });
    assert_eq!(out(), 2 * (1..=16).sum::<u64>());
    let mut p = Pipeline::<Msg, (Total300,)>::build(Configs::new()).unwrap();
    p.push(Msg { key: 0, v: 3 });
    assert_eq!(out(), 3 * (1..=300).sum::<u64>());
    p.push(Msg { key: 0, v: 1 });
    assert_eq!(out(), (1..=300).sum::<u64>());
}

#[test]
fn new_sees_the_first_outputs() {
    // `new` folds `a + v + 1` over 16 feeds whose first outputs are 0: 16.
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let p = Pipeline::<Msg, (Total16,)>::build(Configs::new()).unwrap();
    assert_eq!(p.len(), 1 + 16);
}

// -- get, for_each, order ---------------------------------------------------------------------

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(feeds: for S: Num in Cat16 { Feed<S> })]
pub struct Picker;
impl Component for Picker {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) {
        assert_eq!(*i.feeds.get::<Sy<3>>(), 0);
        (Picker, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let f = i.feeds;
        let a = *f.get::<Sy<0>>();
        let b = *f.get::<Sy<3>>();
        let c: &u64 = f.get::<Sy<15>>();
        let d = *f.get_ref::<Sy<8>>();
        let mut seen = Vec::new();
        f.for_each(|v: &u64| seen.push(*v));
        log(format!("{seen:?}"));
        assert_eq!(f.len(), 16);
        a * 1_000_000_000 + b * 1_000_000 + c * 1_000 + d
    }
}

#[test]
fn get_and_for_each_in_listing_order() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut p = Pipeline::<Msg, (Picker,)>::build(Configs::new()).unwrap();
    p.push(Msg { key: 0, v: 10 });
    let want: Vec<u64> = (1..=16).map(|n| n * 10).collect();
    assert_eq!(take(), [format!("{want:?}")]);
}

// -- mixed output types ---------------------------------------------------------------------

pub struct A;
pub struct B;
pub struct C;
pub trait Kind: 'static {
    type T: 'static + Clone + std::fmt::Debug;
    fn make(v: u64) -> Self::T;
}
impl Kind for A {
    type T = u64;
    fn make(v: u64) -> u64 {
        v + 1
    }
}
impl Kind for B {
    type T = String;
    fn make(v: u64) -> String {
        format!("b{v}")
    }
}
impl Kind for C {
    type T = (u8, f64);
    fn make(v: u64) -> (u8, f64) {
        (v as u8, v as f64 / 2.0)
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct Cell<S: Kind>(PhantomData<S>);
impl<S: Kind> Component for Cell<S> {
    type Config = ();
    type Output = S::T;
    fn new((): (), _: Self::Init<'_>) -> (Self, S::T) {
        (Cell(PhantomData), S::make(0))
    }
    fn update(&mut self, i: Self::Input<'_>) -> S::T {
        S::make(i.m.v)
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(cells: for S: Kind in catalog![A, B, C] { Cell<S> })]
pub struct Mixed;
impl Component for Mixed {
    type Config = ();
    type Output = String;
    fn new((): (), i: Self::Init<'_>) -> (Self, String) {
        assert_eq!(i.cells.get::<B>(), "b0");
        (Mixed, String::new())
    }
    fn update(&mut self, i: Self::Input<'_>) -> String {
        let a: &u64 = i.cells.get::<A>();
        let b: &String = i.cells.get::<B>();
        let c: &(u8, f64) = i.cells.get::<C>();
        format!("{a} {b} {c:?}")
    }
}

#[test]
fn mixed_types_use_get() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Msg, (Mixed,)>::build(Configs::new()).unwrap();
    p.push(Msg { key: 0, v: 4 });
    let _ = &mut p;
}

// -- beside named and unnamed entries ---------------------------------------------------------

/// Counts the unnamed `for` entries' calls.
#[derive(ComponentSubscription)]
#[subscribe(m: Msg, feeds: for S: Num in catalog![Sy<1>, Sy<2>] { Feed<S> }, for S: Num in catalog![Sy<3>] { Feed<S> }, last: Feed<Sy<4>>)]
pub struct Beside {
    calls: Vec<u32>,
}
impl Component for Beside {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) {
        assert_eq!(*i.last, 0);
        (Beside { calls: Vec::new() }, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let calls = std::mem::take(&mut self.calls);
        log(format!("calls {calls:?}"));
        i.feeds.fold(0, |a, v| a + *v) * 1000 + *i.last * 10 + i.m.v
    }
}
impl UpdateHandler<Msg> for Beside {}
impl<S: Num> UpdateHandler<Feed<S>> for Beside {
    fn on_update(&mut self, _: Self::Input<'_>, _v: &u64) {
        self.calls.push(S::KEY);
    }
}

#[test]
fn beside_named_and_unnamed_entries() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut p = Pipeline::<Msg, (Beside,)>::build(Configs::new()).unwrap();
    p.push(Msg { key: 0, v: 1 });
    // The set has Feed<Sy<1>> and Feed<Sy<2>> (outputs 2, 3); `Sy<3>` is the unnamed `for` and is handled.
    assert_eq!(take(), ["calls [1, 2, 3, 4]"]);
    let _ = &mut p;
}

// -- selected elements: they run on their key only, the others keep stale values ----------------------

pub struct Pick<S>(PhantomData<fn() -> S>);
impl<S: Num> Select<Msg> for Pick<S> {
    type Out = u64;
    const KEY: u32 = S::KEY;
    fn select(x: &Msg) -> Option<&u64> {
        (x.key == S::KEY).then_some(&x.v)
    }
    fn key_of(x: &Msg) -> u32 {
        x.key
    }
}
impl<S: Num> Initial<Msg> for Pick<S> {
    fn initial() -> u64 {
        0
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Pick::<S>(u): Msg)]
pub struct Gated<S: Num>(PhantomData<S>);
impl<S: Num> Component for Gated<S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Gated(PhantomData), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("gated {}", S::KEY));
        *i.u * 100 + S::KEY as u64
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(g: for S: Num in catalog![Sy<0>, Sy<1>, Sy<2>] { Gated<S> })]
pub struct Latest;
impl Component for Latest {
    type Config = ();
    type Output = Vec<u64>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Vec<u64>) {
        (Latest, Vec::new())
    }
    fn update(&mut self, i: Self::Input<'_>) -> Vec<u64> {
        let mut all = Vec::new();
        i.g.for_each(|v: &u64| all.push(*v));
        log(format!("latest {all:?}"));
        all
    }
}

#[test]
fn conditional_elements_keep_stale_values() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut p = Pipeline::<Msg, (Latest,)>::build(Configs::new()).unwrap();
    take();
    p.push(Msg { key: 1, v: 7 });
    assert_eq!(take(), ["gated 1", "latest [0, 701, 0]"]);
    p.push(Msg { key: 2, v: 5 });
    assert_eq!(take(), ["gated 2", "latest [0, 701, 502]"]);
    // A key outside the set: nothing fired, `Latest` does not run.
    p.push(Msg { key: 9, v: 5 });
    assert_eq!(take(), Vec::<String>::new());
    p.push(Msg { key: 0, v: 1 });
    assert_eq!(take(), ["gated 0", "latest [100, 701, 502]"]);
}

// -- order and lifecycle ------------------------------------------------------------------------

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct Life<S: Num>(PhantomData<S>);
impl<S: Num> Component for Life<S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        NEW.fetch_add(1, SeqCst);
        log(format!("new {}", S::KEY));
        (Life(PhantomData), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("run {}", S::KEY));
        i.m.v
    }
}
impl<S: Num> Drop for Life<S> {
    fn drop(&mut self) {
        DROP.fetch_add(1, SeqCst);
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(first: Life<Sy<9>>, set: for S: Num in catalog![Sy<5>, Sy<3>, Sy<7>] { Life<S> }, last: Life<Sy<1>>)]
pub struct Order;
impl Component for Order {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        log("new Order");
        (Order, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log("run Order");
        *i.first + *i.last + i.set.fold(0, |a, v| a + *v)
    }
}

#[test]
fn order_and_lifecycle() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    {
        let mut p = Pipeline::<Msg, (Order,)>::build(Configs::new()).unwrap();
        assert_eq!(take(), ["new 9", "new 5", "new 3", "new 7", "new 1", "new Order"]);
        p.push(Msg { key: 0, v: 2 });
        assert_eq!(take(), ["run 9", "run 5", "run 3", "run 7", "run 1", "run Order"]);
        assert_eq!(NEW.load(SeqCst), 5);
        assert_eq!(DROP.load(SeqCst), 0);
    }
    assert_eq!(DROP.load(SeqCst), 5);
}

// -- aliases and duplicates -----------------------------------------------------------------------

type FeedOne = Feed<Sy<1>>;

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(plain: Feed<Sy<1>>, alias: FeedOne, set: for S: Num in catalog![Sy<1>, Sy<2>, Sy<1>] { Feed<S> })]
pub struct Dups;
impl Component for Dups {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Dups, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        // One component for all three; the set lists Sy<1> twice (two entries, the same value).
        assert!(std::ptr::eq(i.plain, i.alias));
        assert!(std::ptr::eq(i.plain, i.set.get::<Sy<1>>()));
        let n = i.set.fold(0u32, |n, _: &u64| n + 1);
        assert_eq!(n, 3);
        let s = i.set.fold(0u64, |a, v| a + *v);
        let out = *i.plain * 100 + s;
        OUT.lock().unwrap().push(out);
        out
    }
}

#[test]
fn aliases_and_duplicates_are_one_component() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut p = Pipeline::<Msg, (Dups,)>::build(Configs::new()).unwrap();
    assert_eq!(p.len(), 1 + 2 + 1 - 1); // Feed<1>, Feed<2>, Dups (+ the root counted by len)
    p.push(Msg { key: 0, v: 1 });
    // Feed<1> = 2, Feed<2> = 3: plain 2, set 2 + 3 + 2 = 7
    assert_eq!(out(), 2 * 100 + 7);
}

// -- other catalog combinators --------------------------------------------------------------------

pub struct Venue<const K: usize>;
pub trait Mul: 'static {
    const M: u64;
}
impl<const K: usize> Mul for Venue<K> {
    const M: u64 = 10u64.pow(K as u32);
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct Book<X: Mul, S: Num>(PhantomData<(X, S)>);
impl<X: Mul, S: Num> Component for Book<X, S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Book(PhantomData), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        i.m.v * X::M * S::N
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(books: for (X: Mul, S: Num) in Product<catalog![Venue<0>, Venue<1>], catalog![Sy<0>, Sy<1>, Sy<2>]> { Book<X, S> })]
pub struct Books;
impl Component for Books {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Books, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        // venue 1, symbol 2 is the fifth element
        let one = *i.books.get::<M<Venue<1>, Sy<1>>>();
        let all = i.books.fold(0, |a, v| a + *v);
        one * 10_000 + all
    }
}

#[test]
fn product_catalog_get_and_fold() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Msg, (Books,)>::build(Configs::new()).unwrap();
    p.push(Msg { key: 0, v: 1 });
    let _ = &mut p;
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(z: for (X: Mul, S: Num) in Zip<catalog![Venue<0>, Venue<1>], catalog![Sy<4>, Sy<5>]> { Book<X, S> })]
pub struct Zipped;
impl Component for Zipped {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Zipped, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        *i.z.get::<M<Venue<1>, Sy<5>>>() + i.z.fold(0, |a, v| a + *v)
    }
}

#[test]
fn zip_catalog() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Msg, (Zipped,)>::build(Configs::new()).unwrap();
    p.push(Msg { key: 0, v: 1 });
}

// -- handle parents: each element's `View` as it is -----------------------------------------------

/// A handle component: the state is a counter, the handle `()`, the view a reference into the state.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct Ring<S: Num> {
    n: u64,
    _s: PhantomData<S>,
}
impl<S: Num> Component for Ring<S> {
    type Config = u64;
    type Handle = ();
    type View<'a> = &'a u64;
    fn new(start: u64, _: Self::Init<'_>) -> (Self, ()) {
        (Ring { n: start, _s: PhantomData }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        self.n += i.m.v * S::N;
    }
    fn view<'a>(&'a self, (): &'a ()) -> &'a u64 {
        &self.n
    }
}
/// A handle component with an owned view.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m: Msg)]
pub struct Own<S: Num> {
    n: u64,
    _s: PhantomData<S>,
}
impl<S: Num> Component for Own<S> {
    type Config = ();
    type Handle = ();
    type View<'a> = (u64, String);
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Own { n: 0, _s: PhantomData }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        self.n += i.m.v + S::N;
    }
    fn view<'a>(&'a self, (): &'a ()) -> (u64, String) {
        (self.n, format!("o{}", S::KEY))
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(rings: for S: Num in catalog![Sy<0>, Sy<1>, Sy<2>] { Ring<S> }, owned: for S: Num in catalog![Sy<4>, Sy<5>] { Own<S> }, m: Msg)]
pub struct Views;
impl Component for Views {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) {
        let start: u64 = i.rings.fold(0, |a, v: &u64| a + *v);
        assert_eq!(start, 10 + 20 + 30);
        (Views, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let ring1: &u64 = i.rings.get::<Sy<1>>();
        let sum = i.rings.fold(0, |a, v: &u64| a + *v);
        let (n, name) = i.owned.get_ref::<Sy<5>>().clone();
        let mut names = String::new();
        i.owned.for_each(|(_, s): (u64, String)| names.push_str(&s));
        assert_eq!(names, "o4o5");
        let out = sum * 1000 + *ring1 * 10 + n + i.m.v;
        assert_eq!(name, "o5");
        OUT.lock().unwrap().push(out);
        out
    }
}

#[test]
fn handle_parents_give_their_views() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let cfg = Configs::new().with::<Ring<Sy<0>>>(10).with::<Ring<Sy<1>>>(20).with::<Ring<Sy<2>>>(30);
    let mut p = Pipeline::<Msg, (Views,)>::build(cfg).unwrap();
    p.push(Msg { key: 0, v: 1 });
    // rings: 10 + 1, 20 + 2, 30 + 3 = 66; ring1 = 22; Own<5>: 0 + 1 + 6 = 7
    assert_eq!(out(), 66 * 1000 + 22 * 10 + 7 + 1);
}
