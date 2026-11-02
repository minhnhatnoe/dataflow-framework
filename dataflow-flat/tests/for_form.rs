//! `for` entries in `#[subscribe(..)]`: one handled entry per element of a catalog
//! (`catalog![..]`, `Product`, `Concat`, `Zip`). Catalogs of 1, 16, 17 and 300
//! elements, order across plain and `for` entries, mapped pairs. No `#![feature]` here.

use std::marker::PhantomData;
use std::sync::Mutex;

use dataflow_flat::{
    catalog, Component, ComponentSubscription, Concat, Configs, Initial, Pipeline, Product, Root, Select,
    UpdateHandler, Zip,
};

static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SERIAL: Mutex<()> = Mutex::new(());
fn take() -> Vec<String> {
    std::mem::take(&mut *LOG.lock().unwrap())
}
fn log(s: impl Into<String>) {
    LOG.lock().unwrap().push(s.into());
}

#[derive(Root)]
pub struct Msg {
    pub key: Option<u32>,
    pub v: u64,
}
pub struct Frame {
    key: Option<u32>,
    v: u64,
}

/// A symbol: a type-level name with an integer key.
pub trait Symbol: 'static {
    const KEY: u32;
    const NAME: &'static str;
}

/// A hand-written selector for the symbol `S`: matches iff the frame has key `S::KEY`.
/// Different keys at one place are mutually exclusive.
pub struct Pick<S>(PhantomData<fn() -> S>);
impl<S: Symbol> Select<Frame> for Pick<S> {
    type Out = u64;
    const KEY: u32 = S::KEY;
    fn select(x: &Frame) -> Option<&u64> {
        (x.key == Some(S::KEY)).then_some(&x.v)
    }
    fn key_of(x: &Frame) -> u32 {
        x.key.unwrap_or(dataflow_flat::GUARD)
    }
}
impl<S: Symbol> Initial<Frame> for Pick<S> {
    fn initial() -> u64 {
        0
    }
}

#[derive(ComponentSubscription)]
#[subscribe(m: Msg)]
pub struct Ws;
impl Component for Ws {
    type Config = ();
    type Output = Option<Frame>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Option<Frame>) {
        (Ws, None)
    }
    fn update(&mut self, i: Self::Input<'_>) -> Option<Frame> {
        Some(Frame { key: i.m.key, v: i.m.v })
    }
}

/// Symbol number `K`.
pub struct Sy<const K: usize>;
impl<const K: usize> Symbol for Sy<K> {
    const KEY: u32 = K as u32;
    const NAME: &'static str = "Sy";
}

#[derive(ComponentSubscription)]
#[subscribe(Some(Pick::<S>(u)): Ws)]
pub struct Quoter<S: Symbol>(PhantomData<S>);
impl<S: Symbol> Component for Quoter<S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Quoter(PhantomData), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        *i.u * 2
    }
}

include!("for_form_cats.inc");

/// A handler over a catalog of symbols: `on_update` records (key, value), `update` logs and hands them out.
macro_rules! oms {
    ($name:ident, $cat:ty) => {
        #[derive(ComponentSubscription)]
        #[subscribe(for S: Symbol in $cat { Quoter<S> })]
        pub struct $name {
            got: Vec<(u32, u64)>,
        }
        impl Component for $name {
            type Config = ();
            type Output = Vec<(u32, u64)>;
            fn new((): (), _: Self::Init<'_>) -> (Self, Self::Output) {
                ($name { got: Vec::new() }, Vec::new())
            }
            fn update(&mut self, _: Self::Input<'_>) -> Self::Output {
                log(format!("{}{:?}", stringify!($name), self.got));
                std::mem::take(&mut self.got)
            }
        }
        impl<S: Symbol> UpdateHandler<Quoter<S>> for $name {
            fn on_update(&mut self, v: &u64) {
                self.got.push((S::KEY, *v));
            }
        }
    };
}

/// Pushes each key: the handler runs once with exactly that quoter; a key outside the catalog runs nothing.
macro_rules! probe_test {
    ($test:ident, $name:ident, $cat:ty, $keys:expr, $miss:expr) => {
        oms!($name, $cat);
        #[test]
        fn $test() {
            let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
            let mut p = Pipeline::<Msg, ($name,)>::build(Configs::new()).unwrap();
            take();
            for k in $keys {
                p.push(Msg { key: Some(k as u32), v: 5 });
                assert_eq!(take(), [format!("{}[({}, 10)]", stringify!($name), k)], "key {k}");
            }
            p.push(Msg { key: Some($miss), v: 1 });
            assert_eq!(take(), Vec::<String>::new());
        }
    };
}

probe_test!(catalog_of_one, Oms1, Cat1, [0usize], 77);
probe_test!(catalog_of_sixteen, Oms16, Cat16, [0usize, 7, 15], 77);
probe_test!(catalog_of_seventeen, Oms17, Cat17, [0usize, 15, 16], 77);
probe_test!(catalog_of_300, Oms300, Cat300, [0usize, 1, 15, 16, 17, 255, 256, 299], 1000);
probe_test!(
    concat_of_catalogs,
    OmsCat,
    Concat<catalog![Sy<0>, Sy<1>], Concat<catalog![Sy<2>], catalog![Sy<3>, Sy<4>]>>,
    [0usize, 2, 4],
    9
);
probe_test!(concat_of_big_catalogs, OmsCat2, Concat<Cat300, CatHi>, [0usize, 15, 16, 299, 300, 316], 1000);

// -- order: plain and `for` entries, in listing order ---------------------------------------------

macro_rules! ticks {
    ($($name:ident),*) => { $(
        #[derive(ComponentSubscription)]
        #[subscribe(m: Msg)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
                ($name, 0)
            }
            fn update(&mut self, _: Self::Input<'_>) -> u64 {
                log(concat!("run ", stringify!($name)));
                1
            }
        }
    )* };
}
ticks!(First, T0, T1, T2, Last, U0, U1);

/// Listing order: `First`, T0..T2, `Last`, then U0, U1, with a named input in between; all fire on every push.
#[derive(ComponentSubscription)]
#[subscribe(First, for T in catalog![T0, T1, T2] { T }, Last, limits: Limits, for U in catalog![U0, U1] { U })]
pub struct Mix;
#[derive(ComponentSubscription)]
#[subscribe(m: Msg)]
pub struct Limits;
impl Component for Limits {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Limits, 7)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u64 {
        log("run Limits");
        7
    }
}
impl Component for Mix {
    type Config = ();
    type Output = ();
    fn new((): (), i: Self::Init<'_>) -> (Self, ()) {
        log(format!("Mix.new limits={}", i.limits));
        (Mix, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        log(format!("Mix.update limits={}", i.limits));
    }
}
macro_rules! handles_mix {
    ($($t:ident),*) => { $(impl UpdateHandler<$t> for Mix {
        fn on_update(&mut self, _: &u64) {
            log(concat!("on ", stringify!($t)));
        }
    })* };
}
handles_mix!(First, T0, T1, T2, Last, U0, U1);

#[test]
fn for_and_plain_entries_keep_listing_order() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Msg, (Mix,)>::build(Configs::new()).unwrap();
    // `new` is called with the named input only; no `on_update` at build.
    let l = take();
    assert!(l.contains(&"Mix.new limits=7".to_string()), "{l:?}");
    assert!(!l.iter().any(|x| x.starts_with("on ")), "{l:?}");
    p.push(Msg { key: None, v: 0 });
    let got = take();
    // Execution is depth-first post-order in listing order; `on_update` follows it, then `update`.
    let on: Vec<&str> = got.iter().filter(|x| x.starts_with("on ")).map(|x| x.as_str()).collect();
    assert_eq!(on, ["on First", "on T0", "on T1", "on T2", "on Last", "on U0", "on U1"]);
    assert_eq!(got.last().unwrap(), "Mix.update limits=7");
    let run: Vec<&str> = got.iter().filter(|x| x.starts_with("run ")).map(|x| x.as_str()).collect();
    assert_eq!(run, ["run First", "run T0", "run T1", "run T2", "run Last", "run Limits", "run U0", "run U1"]);
}

// -- Product and Zip: pairs --------------------------------------------------------------------------

pub trait Tag: 'static {
    const TAG: u32;
}
pub struct V0;
pub struct V1;
pub struct V2;
impl Tag for V0 {
    const TAG: u32 = 0;
}
impl Tag for V1 {
    const TAG: u32 = 1;
}
impl Tag for V2 {
    const TAG: u32 = 2;
}
/// The symbol of the pair `(venue, coin)`: key `100 * venue + coin`.
pub struct Pair<X, S>(PhantomData<(X, S)>);
impl<X: Tag, S: Symbol> Symbol for Pair<X, S> {
    const KEY: u32 = X::TAG * 100 + S::KEY;
    const NAME: &'static str = "Pair";
}

#[derive(ComponentSubscription)]
#[subscribe(Some(Pick::<Pair<X, S>>(u)): Ws)]
pub struct Book<X: Tag, S: Symbol>(PhantomData<(X, S)>);
impl<X: Tag, S: Symbol> Component for Book<X, S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Book(PhantomData), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        *i.u + 1
    }
}

macro_rules! books {
    ($name:ident, $cat:ty) => {
        #[derive(ComponentSubscription)]
        #[subscribe(for (X: Tag, S: Symbol) in $cat { Book<X, S> })]
        pub struct $name {
            got: Vec<u32>,
        }
        impl Component for $name {
            type Config = ();
            type Output = ();
            fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
                ($name { got: Vec::new() }, ())
            }
            fn update(&mut self, _: Self::Input<'_>) {
                log(format!("{}{:?}", stringify!($name), self.got));
                self.got.clear();
            }
        }
        impl<X: Tag, S: Symbol> UpdateHandler<Book<X, S>> for $name {
            fn on_update(&mut self, _: &u64) {
                self.got.push(<Pair<X, S> as Symbol>::KEY);
            }
        }
    };
}

type Venues = catalog![V0, V1, V2];
type Coins = catalog![Sy<0>, Sy<1>, Sy<2>, Sy<3>];
books!(Prod, Product<Venues, Coins>);
books!(Zipped, Zip<catalog![V0, V1, V2], catalog![Sy<0>, Sy<1>, Sy<2>]>);
// A product with a big side: 17 venues' worth of rows would be heavy in debug; 3 x 17 keeps both sides past one leaf.
books!(ProdBig, Product<Venues, Cat17>);

#[test]
fn product_covers_every_pair_zip_the_diagonal() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Msg, (Prod,)>::build(Configs::new()).unwrap();
    take();
    for v in 0..3u32 {
        for c in 0..4u32 {
            p.push(Msg { key: Some(v * 100 + c), v: 1 });
            assert_eq!(take(), [format!("Prod[{}]", v * 100 + c)]);
        }
    }
    p.push(Msg { key: Some(4), v: 1 });
    assert_eq!(take(), Vec::<String>::new());

    let mut z = Pipeline::<Msg, (Zipped,)>::build(Configs::new()).unwrap();
    take();
    for d in 0..3u32 {
        z.push(Msg { key: Some(d * 100 + d), v: 1 });
        assert_eq!(take(), [format!("Zipped[{}]", d * 101)]);
    }
    // Off the diagonal there is no entry.
    z.push(Msg { key: Some(1), v: 1 });
    assert_eq!(take(), Vec::<String>::new());

    let mut b = Pipeline::<Msg, (ProdBig,)>::build(Configs::new()).unwrap();
    take();
    for (v, c) in [(0u32, 0u32), (0, 16), (1, 15), (2, 16)] {
        b.push(Msg { key: Some(v * 100 + c), v: 1 });
        assert_eq!(take(), [format!("ProdBig[{}]", v * 100 + c)]);
    }
}
