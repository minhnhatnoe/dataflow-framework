//! Selectors on a handle-component source: a selector takes the view and returns a view of the matched
//! part (`SelectView`, written by `#[derive(Demux)]` on an enum with a lifetime parameter), so a
//! selected input stays a borrowed or owned view. Checks reference and owned variants, a switch
//! over several subscribers, nested chains (`Some(..)`), `new` and `on_update` of a selected
//! entry, a parent that did not fire, order and lifecycle, and a wide subscriber.
//! No `#![feature]` here.

use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, Configs, Demux, Pipeline, Root, UpdateHandler};

static SERIAL: Mutex<()> = Mutex::new(());
static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static NEW: AtomicUsize = AtomicUsize::new(0);
static DROP: AtomicUsize = AtomicUsize::new(0);

fn log(s: impl Into<String>) {
    LOG.lock().unwrap().push(s.into());
}
fn take() -> Vec<String> {
    std::mem::take(&mut *LOG.lock().unwrap())
}
fn reset() {
    take();
    NEW.store(0, SeqCst);
    DROP.store(0, SeqCst);
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

/// The root: `A` events feed the feed, `B` events feed the second one.
#[derive(Clone, Copy, Root, Demux)]
pub enum Msg {
    A(u64),
    B(u64),
}

#[derive(Debug)]
pub struct Book {
    px: u64,
}
#[derive(Debug)]
pub struct Trade {
    qty: u64,
}
enum Frame {
    Book(Book),
    Trade(Trade),
    Seq(u64),
}

/// The view of a frame: two variants borrow from the feed's state, one is owned.
#[derive(Demux)]
pub enum FrameView<'a> {
    Book(&'a Book),
    Trade(&'a Trade),
    Seq(u64),
}

/// A ring of frames; the handle is the slot of the last one. `A(v)` makes a book (`v % 3 == 0`),
/// a trade (`1`) or a sequence number (`2`).
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Msg::A(v): Msg)]
pub struct Feed {
    ring: [Frame; 4],
    n: usize,
    _life: Life,
}
impl Component for Feed {
    type Config = ();
    type Handle = usize;
    type View<'a> = FrameView<'a>;
    fn new((): (), _: Self::Init<'_>) -> (Self, usize) {
        log("new Feed");
        let ring = [Frame::Book(Book { px: 100 }), Frame::Seq(0), Frame::Seq(0), Frame::Seq(0)];
        (Feed { ring, n: 0, _life: Life::new() }, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> usize {
        log("update Feed");
        self.n += 1;
        let slot = self.n % 4;
        let v = *i.v;
        self.ring[slot] = match v % 3 {
            0 => Frame::Book(Book { px: v }),
            1 => Frame::Trade(Trade { qty: v }),
            _ => Frame::Seq(v),
        };
        slot
    }
    fn view<'a>(&'a self, h: &'a usize) -> FrameView<'a> {
        match &self.ring[*h] {
            Frame::Book(b) => FrameView::Book(b),
            Frame::Trade(t) => FrameView::Trade(t),
            Frame::Seq(n) => FrameView::Seq(*n),
        }
    }
}

// ---------------------------------------------------------------------------
// Subscribers of the three variants (a switch on the feed's view)
// ---------------------------------------------------------------------------

static NEW_SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// `b` is exactly the variant's field: `&Book`. `new` gets `Option` of it (the feed's first view
/// is a book).
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(FrameView::Book(b): Feed)]
pub struct Mid(Life);
impl Component for Mid {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) {
        log("new Mid");
        let b: Option<&Book> = i.b;
        NEW_SEEN.lock().unwrap().push(format!("Mid {:?}", b));
        (Mid(Life::new()), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let b: &Book = i.b;
        log(format!("update Mid {}", b.px));
        b.px
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(FrameView::Trade(t): Feed)]
pub struct Tr(Life);
impl Component for Tr {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) {
        log("new Tr");
        let t: Option<&Trade> = i.t;
        NEW_SEEN.lock().unwrap().push(format!("Tr {:?}", t));
        (Tr(Life::new()), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let t: &Trade = i.t;
        log(format!("update Tr {}", t.qty));
        t.qty
    }
}

/// An owned variant: `n` is a `u64`.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(FrameView::Seq(n): Feed)]
pub struct Sq(Life);
impl Component for Sq {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) {
        log("new Sq");
        let n: Option<u64> = i.n;
        NEW_SEEN.lock().unwrap().push(format!("Sq {:?}", n));
        (Sq(Life::new()), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let n: u64 = i.n;
        log(format!("update Sq {}", n));
        n
    }
}

type Three = (Mid, Tr, Sq);

#[test]
fn three_variants_run_only_when_they_match() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    NEW_SEEN.lock().unwrap().clear();
    let mut p = Pipeline::<Msg, Three>::build(Configs::new()).unwrap();
    // `new` got the selected part of the feed's first view, if it matches.
    assert_eq!(*NEW_SEEN.lock().unwrap(), ["Mid Some(Book { px: 100 })", "Tr None", "Sq None"]);
    assert_eq!(take(), ["new Feed", "new Mid", "new Tr", "new Sq"]);
    p.push(Msg::A(3)); // 3 % 3 == 0: a book
    assert_eq!(take(), ["update Feed", "update Mid 3"]);
    p.push(Msg::A(4)); // a trade
    assert_eq!(take(), ["update Feed", "update Tr 4"]);
    p.push(Msg::A(5)); // a sequence number
    assert_eq!(take(), ["update Feed", "update Sq 5"]);
    p.push(Msg::A(6));
    assert_eq!(take(), ["update Feed", "update Mid 6"]);
}

#[test]
fn a_parent_that_did_not_fire_runs_no_subscriber() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut p = Pipeline::<Msg, Three>::build(Configs::new()).unwrap();
    take();
    p.push(Msg::A(7)); // a trade, the feed's last frame
    assert_eq!(take(), ["update Feed", "update Tr 7"]);
    // `B` events do not reach the feed. Its last frame is still a trade, but its edge did not
    // fire: nobody runs.
    p.push(Msg::B(1));
    p.push(Msg::B(2));
    assert_eq!(take(), Vec::<String>::new());
    p.push(Msg::A(9)); // a book again
    assert_eq!(take(), ["update Feed", "update Mid 9"]);
}

#[test]
fn order_and_lifecycle() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    {
        let mut p = Pipeline::<Msg, Three>::build(Configs::new()).unwrap();
        assert_eq!(p.order().len(), 4);
        assert_eq!((NEW.load(SeqCst), DROP.load(SeqCst)), (4, 0));
        for k in 0..6 {
            p.push(Msg::A(k));
        }
    }
    assert_eq!((NEW.load(SeqCst), DROP.load(SeqCst)), (4, 4));
}

// ---------------------------------------------------------------------------
// A nested chain: `Some(FrameView::Book(b))` on a feed whose view is an `Option`
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Msg::B(v): Msg)]
pub struct Feed2 {
    last: Frame,
    _life: Life,
}
impl Component for Feed2 {
    type Config = ();
    type Handle = ();
    type View<'a> = Option<FrameView<'a>>;
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Feed2 { last: Frame::Seq(0), _life: Life::new() }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        let v = *i.v;
        self.last = if v % 2 == 0 { Frame::Book(Book { px: v }) } else { Frame::Seq(v) };
    }
    fn view<'a>(&'a self, (): &'a ()) -> Option<FrameView<'a>> {
        match &self.last {
            Frame::Seq(0) => None,
            Frame::Book(b) => Some(FrameView::Book(b)),
            Frame::Trade(t) => Some(FrameView::Trade(t)),
            Frame::Seq(n) => Some(FrameView::Seq(*n)),
        }
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(FrameView::Book(b)): Feed2)]
pub struct Nested(Life);
impl Component for Nested {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) {
        let b: Option<&Book> = i.b;
        assert!(b.is_none());
        (Nested(Life::new()), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let b: &Book = i.b;
        log(format!("update Nested {}", b.px));
        b.px
    }
}

#[test]
fn a_nested_chain() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut p = Pipeline::<Msg, (Nested,)>::build(Configs::new()).unwrap();
    p.push(Msg::B(3)); // odd: a sequence number, `Some(Seq)` does not match `Book`
    p.push(Msg::A(1)); // not for this feed
    assert_eq!(take(), Vec::<String>::new());
    p.push(Msg::B(4)); // even: a book
    assert_eq!(take(), ["update Nested 4"]);
}

// ---------------------------------------------------------------------------
// `on_update` of a selected entry gets exactly the selected view
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription)]
#[subscribe(FrameView::Book(b): Feed)]
pub struct Hdl(Life);
impl Component for Hdl {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Hdl(Life::new()), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let b: &Book = i.b;
        log(format!("update Hdl {}", b.px));
        b.px
    }
}
impl UpdateHandler<hdl_edges::B> for Hdl {
    fn on_update(&mut self, i: Self::Input<'_>, v: &Book) {
        log(format!("on_update Book {} (input {})", v.px, i.b.px));
    }
}

/// An unnamed selected entry: only `on_update` sees it.
#[derive(ComponentSubscription)]
#[subscribe(FrameView::Trade(_): Feed)]
pub struct Cnt(Life);
impl Component for Cnt {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Cnt(Life::new()), 0)
    }
    fn update(&mut self, _: Self::Input<'_>) -> u64 {
        log("update Cnt");
        0
    }
}
impl UpdateHandler<cnt_edges::Trade> for Cnt {
    fn on_update(&mut self, _: Self::Input<'_>, v: &Trade) {
        log(format!("on_update Trade {}", v.qty));
    }
}

#[test]
fn on_update_gets_the_selected_view() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut p = Pipeline::<Msg, (Hdl, Cnt)>::build(Configs::new()).unwrap();
    take();
    p.push(Msg::A(3));
    assert_eq!(take(), ["update Feed", "on_update Book 3 (input 3)", "update Hdl 3"]);
    p.push(Msg::A(4));
    assert_eq!(take(), ["update Feed", "on_update Trade 4", "update Cnt"]);
    p.push(Msg::A(5));
    assert_eq!(take(), ["update Feed"]);
}

// ---------------------------------------------------------------------------
// A wide subscriber (24 entries: 23 components that run on a trade, and the selected view)
// ---------------------------------------------------------------------------

macro_rules! wleaf {
    ($name:ident, $n:expr) => {
        #[derive(ComponentSubscription, UpdateHandler)]
        #[subscribe(FrameView::Trade(t): Feed)]
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
wleaf!(W0, 0);
wleaf!(W1, 1);
wleaf!(W2, 2);
wleaf!(W3, 3);
wleaf!(W4, 4);
wleaf!(W5, 5);
wleaf!(W6, 6);
wleaf!(W7, 7);
wleaf!(W8, 8);
wleaf!(W9, 9);
wleaf!(W10, 10);
wleaf!(W11, 11);
wleaf!(W12, 12);
wleaf!(W13, 13);
wleaf!(W14, 14);
wleaf!(W15, 15);
wleaf!(W16, 16);
wleaf!(W17, 17);
wleaf!(W18, 18);
wleaf!(W19, 19);
wleaf!(W20, 20);
wleaf!(W21, 21);
wleaf!(W22, 22);

static WIDE_OUT: AtomicUsize = AtomicUsize::new(0);

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(w0: W0, w1: W1, w2: W2, w3: W3, w4: W4, w5: W5, w6: W6, w7: W7, w8: W8, w9: W9, w10: W10, w11: W11, w12: W12, w13: W13, w14: W14, w15: W15, w16: W16, w17: W17, w18: W18, w19: W19, w20: W20, w21: W21, w22: W22, FrameView::Trade(t): Feed)]
pub struct Wide;
impl Component for Wide {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Wide, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let out = *i.w0 + *i.w1 + *i.w2 + *i.w3 + *i.w4 + *i.w5 + *i.w6 + *i.w7 + *i.w8 + *i.w9 + *i.w10 + *i.w11 + *i.w12 + *i.w13 + *i.w14 + *i.w15 + *i.w16 + *i.w17 + *i.w18 + *i.w19 + *i.w20 + *i.w21 + *i.w22 + i.t.qty;
        WIDE_OUT.store(out as usize, SeqCst);
        out
    }
}

#[test]
fn a_wide_subscriber_with_a_selected_view() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut p = Pipeline::<Msg, (Wide,)>::build(Configs::new()).unwrap();
    let base: u64 = (0..23).sum();
    p.push(Msg::A(3)); // a book: the selected entry does not match
    assert_eq!(WIDE_OUT.load(SeqCst), 0);
    p.push(Msg::A(4)); // a trade with qty 4
    assert_eq!(WIDE_OUT.load(SeqCst) as u64, base + 4);
    p.push(Msg::B(1));
    assert_eq!(WIDE_OUT.load(SeqCst) as u64, base + 4);
    p.push(Msg::A(7));
    assert_eq!(WIDE_OUT.load(SeqCst) as u64, base + 7);
}

// ---------------------------------------------------------------------------
// A handle source that itself reads a handle parent (its run is not fused with the switch: the key
// is taken from the stored handle)
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(l: Level)]
pub struct Feed3 {
    book: Book,
    last: u64,
}
impl Component for Feed3 {
    type Config = ();
    type Handle = ();
    type View<'a> = FrameView<'a>;
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Feed3 { book: Book { px: 0 }, last: 0 }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        self.last = i.l;
        self.book.px = i.l * 10;
    }
    fn view<'a>(&'a self, (): &'a ()) -> FrameView<'a> {
        if self.last % 2 == 0 {
            FrameView::Book(&self.book)
        } else {
            FrameView::Seq(self.last)
        }
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(FrameView::Book(b): Feed3)]
pub struct Bk3;
impl Component for Bk3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Bk3, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("update Bk3 {}", i.b.px));
        i.b.px
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(FrameView::Seq(n): Feed3)]
pub struct Sq3;
impl Component for Sq3 {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Sq3, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("update Sq3 {}", i.n));
        i.n
    }
}

#[test]
fn a_handle_source_below_a_handle_source() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut p = Pipeline::<Msg, (Bk3, Sq3)>::build(Configs::new()).unwrap();
    p.push(Msg::B(3));
    p.push(Msg::A(1)); // not for this chain
    p.push(Msg::B(4));
    assert_eq!(take(), ["update Sq3 3", "update Bk3 40"]);
}

// ---------------------------------------------------------------------------
// Two fields spelled alike: two entries, each gets its own value in `new` and `update`
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(FrameView::Book(first): Feed, FrameView::Book(second): Feed)]
pub struct Twin;
impl Component for Twin {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) {
        let (a, b): (Option<&Book>, Option<&Book>) = (i.first, i.second);
        assert_eq!((a.map(|b| b.px), b.map(|b| b.px)), (Some(100), Some(100)));
        (Twin, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("update Twin {} {}", i.first.px, i.second.px));
        i.first.px + i.second.px
    }
}

#[test]
fn two_fields_spelled_alike() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut p = Pipeline::<Msg, (Twin,)>::build(Configs::new()).unwrap();
    p.push(Msg::A(4));
    p.push(Msg::A(6));
    assert_eq!(take().into_iter().filter(|l| l.contains("Twin")).collect::<Vec<_>>(), ["update Twin 6 6"]);
}

// ---------------------------------------------------------------------------
// A hand-written view selector (a guard) on an owned view
// ---------------------------------------------------------------------------

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Msg::B(v): Msg)]
pub struct Level {
    v: u64,
}
impl Component for Level {
    type Config = ();
    type Handle = ();
    type View<'a> = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {
        (Level { v: 0 }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        self.v = *i.v;
    }
    fn view<'a>(&'a self, (): &'a ()) -> u64 {
        self.v
    }
}

/// Matches an odd level; the part is the level.
pub struct Odd;
impl dataflow_flat::SelectView<u64> for Odd {
    type Out = u64;
    const KEY: u32 = dataflow_flat::GUARD;
    fn select(x: u64) -> Option<u64> {
        if x % 2 == 1 {
            Some(x)
        } else {
            None
        }
    }
    fn key_of(_: &u64) -> u32 {
        dataflow_flat::GUARD
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Odd(n): Level)]
pub struct OnOdd;
impl Component for OnOdd {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) {
        assert_eq!(i.n, None);
        (OnOdd, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("update OnOdd {}", i.n));
        i.n
    }
}

#[test]
fn a_hand_written_view_selector() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    reset();
    let mut p = Pipeline::<Msg, (OnOdd,)>::build(Configs::new()).unwrap();
    p.push(Msg::B(2));
    p.push(Msg::B(3));
    p.push(Msg::A(5));
    p.push(Msg::B(8));
    p.push(Msg::B(9));
    assert_eq!(take(), ["update OnOdd 3", "update OnOdd 9"]);
}
