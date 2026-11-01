//! Lifecycle and runtime guarantees: every component is constructed once (in
//! execution order, with `new` receiving its subscriptions' first outputs),
//! updated once per push in execution order, its output stored between pushes,
//! and dropped once.

use std::marker::PhantomData;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline, Root};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ev {
    New,
    Update,
    Drop,
}
type Log = Mutex<Vec<(Ev, String)>>;

fn take(log: &Log) -> Vec<(Ev, String)> {
    std::mem::take(&mut *log.lock().unwrap())
}
fn names(log: &[(Ev, String)], ev: Ev) -> Vec<&str> {
    log.iter().filter(|(e, _)| *e == ev).map(|(_, n)| n.as_str()).collect()
}

/// A root carrying a number (the "event").
#[derive(Root)]
pub struct Tick(pub u64);

/// Logs its own drop; one per component.
struct Life(&'static Log, &'static str);
impl Drop for Life {
    fn drop(&mut self) {
        self.0.lock().unwrap().push((Ev::Drop, self.1.to_string()));
    }
}

/// Define a component that logs into `$log`. Its output is `(value, runs)`:
/// the sum of its inputs' values (and the tick, if it reads it) plus its own
/// base, and how many times it has run. `new` logs and returns `(base, 0)`.
macro_rules! comp {
    ($log:ident, $name:ident, $base:expr, [$($f:ident: $d:ty),*]) => {
        #[derive(ComponentSubscription)]
        #[subscribe($($f: $d),*)]
        pub struct $name {
            _life: Life,
            runs: u64,
        }
        impl Component for $name {
            type Config = ();
            type Output = (u64, u64);
            fn new((): (), _init: Self::Init<'_>) -> (Self, (u64, u64)) {
                $log.lock().unwrap().push((Ev::New, stringify!($name).to_string()));
                ($name { _life: Life(&$log, stringify!($name)), runs: 0 }, ($base, 0))
            }
            fn update(&mut self, input: Self::Input<'_>) -> (u64, u64) {
                $log.lock().unwrap().push((Ev::Update, stringify!($name).to_string()));
                self.runs += 1;
                #[allow(unused_mut)]
                let mut v: u64 = $base;
                $(v += lifecycle_value(input.$f);)*
                (v, self.runs)
            }
        }
    };
}

/// Inputs are either the tick or a component's `(value, runs)`.
trait Value {
    fn value(&self) -> u64;
}
impl Value for Tick {
    fn value(&self) -> u64 {
        self.0
    }
}
impl Value for (u64, u64) {
    fn value(&self) -> u64 {
        self.0
    }
}
fn lifecycle_value<T: Value>(t: &T) -> u64 {
    t.value()
}

// ---------------------------------------------------------------------------
// Diamond: Tick -> Src -> {Double, Square} -> Sum; sinks (Sum, Double).
// ---------------------------------------------------------------------------
mod diamond {
    use super::*;
    pub static LOG: Log = Mutex::new(Vec::new());
    comp!(LOG, Src, 10, [t: Tick]);
    comp!(LOG, Double, 0, [x: Src]);
    comp!(LOG, Square, 1, [x: Src]);
    comp!(LOG, Sum, 0, [d: Double, s: Square, again: Double]);
}

#[test]
fn diamond_lifecycle() {
    use diamond::*;
    take(&LOG);
    {
        let mut p = Pipeline::<Tick, (Sum, Double)>::build(Configs::new()).unwrap();
        // Construction: every component once, in execution order.
        let log = take(&LOG);
        assert_eq!(names(&log, Ev::New), ["Src", "Double", "Square", "Sum"]);
        assert_eq!(log.len(), 4, "nothing runs or drops at build");
        assert_eq!(p.len(), 4);
        assert_eq!(p.order().iter().map(|n| n.rsplit("::").next().unwrap()).collect::<Vec<_>>(), [
            "Src", "Double", "Square", "Sum"
        ]);

        for step in 1..=3u64 {
            p.push(Tick(step));
            let log = take(&LOG);
            // Each component exactly once, in the documented order; no new, no drop.
            assert_eq!(names(&log, Ev::Update), ["Src", "Double", "Square", "Sum"]);
            assert_eq!(log.len(), 4);
        }
    }
    // Dropped once each when the pipeline goes away (and not before).
    let log = take(&LOG);
    let mut dropped = names(&log, Ev::Drop);
    assert_eq!(log.len(), 4);
    dropped.sort_unstable();
    assert_eq!(dropped, ["Double", "Square", "Src", "Sum"]);
}

// ---------------------------------------------------------------------------
// `new` is given the subscriptions' first outputs; `update` reads the fresh
// ones. A root with a payload is read by value of the current push.
// ---------------------------------------------------------------------------
mod values {
    use super::*;

    pub static SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());

    #[derive(ComponentSubscription)]
    #[subscribe(t: Tick)]
    pub struct Counter {
        n: u64,
    }
    impl Component for Counter {
        type Config = u64;
        type Output = u64;
        fn new(start: u64, init: Self::Init<'_>) -> (Self, u64) {
            // The root has no value before the first push: `()`.
            let (): () = init.t;
            (Counter { n: start }, start)
        }
        fn update(&mut self, input: Self::Input<'_>) -> u64 {
            self.n += input.t.0;
            self.n
        }
    }

    #[derive(ComponentSubscription)]
    #[subscribe(c: Counter, t: Tick)]
    pub struct Report;
    impl Component for Report {
        type Config = ();
        type Output = ();
        fn new((): (), init: Self::Init<'_>) -> (Self, ()) {
            // A component's first output exists: it is the config value here.
            SEEN.lock().unwrap().push(format!("new sees counter={}", init.c));
            (Report, ())
        }
        fn update(&mut self, input: Self::Input<'_>) {
            SEEN.lock().unwrap().push(format!("tick={} counter={}", input.t.0, input.c));
        }
    }
}

#[test]
fn new_receives_first_outputs_and_update_reads_fresh_ones() {
    use values::*;
    let mut p = Pipeline::<Tick, (Report,)>::build(Configs::new().with::<Counter>(100)).unwrap();
    p.push(Tick(1));
    p.push(Tick(5));
    p.push(Tick(2));
    assert_eq!(
        *SEEN.lock().unwrap(),
        ["new sees counter=100", "tick=1 counter=101", "tick=5 counter=106", "tick=2 counter=108"]
    );
}

// ---------------------------------------------------------------------------
// Outputs are stored: created by `new`, replaced (and the old one dropped) by
// each update, the last one dropped with the pipeline.
// ---------------------------------------------------------------------------
mod stored {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};

    pub static MADE: AtomicUsize = AtomicUsize::new(0);
    pub static DROPPED: AtomicUsize = AtomicUsize::new(0);

    pub struct Out(pub usize);
    impl Out {
        fn make() -> Out {
            Out(MADE.fetch_add(1, SeqCst))
        }
    }
    impl Drop for Out {
        fn drop(&mut self) {
            DROPPED.fetch_add(1, SeqCst);
        }
    }

    #[derive(ComponentSubscription)]
    #[subscribe(t: Tick)]
    pub struct Producer;
    impl Component for Producer {
        type Config = ();
        type Output = Out;
        fn new((): (), _: Self::Init<'_>) -> (Self, Out) {
            (Producer, Out::make())
        }
        fn update(&mut self, _: Self::Input<'_>) -> Out {
            Out::make()
        }
    }

    #[derive(ComponentSubscription)]
    #[subscribe(p: Producer)]
    pub struct Consumer;
    impl Component for Consumer {
        type Config = ();
        type Output = usize;
        fn new((): (), init: Self::Init<'_>) -> (Self, usize) {
            (Consumer, init.p.0)
        }
        fn update(&mut self, input: Self::Input<'_>) -> usize {
            input.p.0
        }
    }

    #[test]
    fn outputs_are_stored_and_replaced() {
        let mut p = Pipeline::<Tick, (Consumer,)>::build(Configs::new()).unwrap();
        assert_eq!((MADE.load(SeqCst), DROPPED.load(SeqCst)), (1, 0), "new made the first output");
        for n in 1..=4 {
            p.push(Tick(0));
            // One new output per push; the replaced one is dropped, the new one stays.
            assert_eq!((MADE.load(SeqCst), DROPPED.load(SeqCst)), (1 + n, n));
        }
        drop(p);
        assert_eq!((MADE.load(SeqCst), DROPPED.load(SeqCst)), (5, 5), "the stored output is dropped with the pipeline");
    }
}

// ---------------------------------------------------------------------------
// Movability and Send
// ---------------------------------------------------------------------------
fn assert_send<T: Send>() {}

#[test]
fn pipeline_is_send_and_movable() {
    use diamond::*;
    assert_send::<Pipeline<Tick, (Sum, Double)>>();
    take(&LOG);
    let p = Pipeline::<Tick, (Sum,)>::build(Configs::new()).unwrap();
    let mut p = std::thread::spawn(move || {
        let mut p = p;
        p.push(Tick(1));
        p
    })
    .join()
    .unwrap();
    p.push(Tick(2));
    let log = take(&LOG);
    assert_eq!(names(&log, Ev::Update).len(), 8, "four components, two pushes");
    drop(p);
    // `Pipeline` reports a lifecycle log of its own: nothing is dropped twice.
    assert_eq!(names(&take(&LOG), Ev::Drop).len(), 4);
}

// ---------------------------------------------------------------------------
// Panics: every already-built component is dropped exactly once.
// ---------------------------------------------------------------------------
mod boom {
    use super::*;
    pub static LOG: Log = Mutex::new(Vec::new());
    pub static BLOW_UP_NEW: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    pub static BLOW_UP_UPDATE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

    comp!(LOG, First, 1, [t: Tick]);
    comp!(LOG, Second, 1, [f: First]);

    #[derive(ComponentSubscription)]
    #[subscribe(s: Second)]
    pub struct Third(#[allow(dead_code)] Option<Life>);
    impl Component for Third {
        type Config = ();
        type Output = u64;
        fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
            if BLOW_UP_NEW.load(std::sync::atomic::Ordering::SeqCst) {
                panic!("boom in new");
            }
            LOG.lock().unwrap().push((Ev::New, "Third".into()));
            (Third(Some(Life(&LOG, "Third"))), 0)
        }
        fn update(&mut self, _: Self::Input<'_>) -> u64 {
            if BLOW_UP_UPDATE.load(std::sync::atomic::Ordering::SeqCst) {
                panic!("boom in update");
            }
            LOG.lock().unwrap().push((Ev::Update, "Third".into()));
            0
        }
    }
}

#[test]
fn panics_never_double_drop() {
    use boom::*;
    use std::sync::atomic::Ordering::SeqCst;
    take(&LOG);

    // A constructor panics: First and Second were built and are dropped once.
    BLOW_UP_NEW.store(true, SeqCst);
    let r = catch_unwind(|| {
        let _ = Pipeline::<Tick, (Third,)>::build(Configs::new());
    });
    assert!(r.is_err());
    BLOW_UP_NEW.store(false, SeqCst);
    let log = take(&LOG);
    assert_eq!(names(&log, Ev::New), ["First", "Second"]);
    let mut dropped = names(&log, Ev::Drop);
    dropped.sort_unstable();
    assert_eq!(dropped, ["First", "Second"]);

    // An update panics: the pipeline is still dropped cleanly, each component once.
    let mut p = Pipeline::<Tick, (Third,)>::build(Configs::new()).unwrap();
    take(&LOG);
    BLOW_UP_UPDATE.store(true, SeqCst);
    let r = catch_unwind(AssertUnwindSafe(|| p.push(Tick(1))));
    assert!(r.is_err());
    BLOW_UP_UPDATE.store(false, SeqCst);
    assert_eq!(names(&take(&LOG), Ev::Update), ["First", "Second"], "the ones before the panic ran");
    p.push(Tick(2));
    take(&LOG);
    drop(p);
    let mut dropped = names(&take(&LOG), Ev::Drop).into_iter().map(String::from).collect::<Vec<_>>();
    dropped.sort_unstable();
    assert_eq!(dropped, ["First", "Second", "Third"]);
}

// ---------------------------------------------------------------------------
// Over-aligned components: the storage honours their alignment.
// ---------------------------------------------------------------------------
mod aligned {
    use super::*;

    #[derive(ComponentSubscription)]
    #[subscribe(t: Tick)]
    #[repr(align(64))]
    pub struct Wide64 {
        pad: [u8; 3],
    }
    impl Component for Wide64 {
        type Config = ();
        type Output = usize;
        fn new((): (), _: Self::Init<'_>) -> (Self, usize) {
            (Wide64 { pad: [1, 2, 3] }, 0)
        }
        fn update(&mut self, _: Self::Input<'_>) -> usize {
            assert_eq!(self.pad, [1, 2, 3]);
            let addr = self as *const Self as usize;
            assert_eq!(addr % 64, 0, "Wide64 is 64-aligned");
            addr
        }
    }

    #[derive(ComponentSubscription)]
    #[subscribe(w: Wide64, t: Tick)]
    #[repr(align(256))]
    pub struct Wide256(PhantomData<u8>);
    impl Component for Wide256 {
        type Config = ();
        type Output = (usize, usize);
        fn new((): (), _: Self::Init<'_>) -> (Self, (usize, usize)) {
            (Wide256(PhantomData), (0, 0))
        }
        fn update(&mut self, input: Self::Input<'_>) -> (usize, usize) {
            let addr = self as *const Self as usize;
            assert_eq!(addr % 256, 0, "Wide256 is 256-aligned");
            assert_eq!(*input.w % 64, 0, "Wide64 is 64-aligned");
            (*input.w, addr)
        }
    }
}

#[test]
fn over_aligned_components() {
    use aligned::*;
    let mut p = Pipeline::<Tick, (Wide256,)>::build(Configs::new()).unwrap();
    p.push(Tick(0));
    p.push(Tick(1));
    assert_eq!(std::mem::align_of::<Wide256>(), 256);
}
