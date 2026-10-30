//! Lifecycle tests: every component logs new / update / drop into a per-graph
//! log; each graph has one root `App` component whose output `run` returns.

#![allow(clippy::type_complexity, dead_code)]

use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ev {
    New,
    Update,
    Drop,
}

type Log = Mutex<Vec<(Ev, String)>>;

/// Logs its own drop. One per component.
struct Probe {
    name: String,
    log: &'static Log,
}
impl Probe {
    fn new(log: &'static Log, name: &str) -> Probe {
        log.lock().unwrap().push((Ev::New, name.to_string()));
        Probe { name: name.to_string(), log }
    }
    fn update(&self) {
        self.log.lock().unwrap().push((Ev::Update, self.name.clone()));
    }
}
impl Drop for Probe {
    fn drop(&mut self) {
        self.log.lock().unwrap().push((Ev::Drop, self.name.clone()));
    }
}

fn take(log: &Log) -> Vec<(Ev, String)> {
    std::mem::take(&mut *log.lock().unwrap())
}

fn names(log: &[(Ev, String)], ev: Ev) -> Vec<&str> {
    log.iter().filter(|(e, _)| *e == ev).map(|(_, n)| n.as_str()).collect()
}

fn pos(v: &[&str], n: &str) -> usize {
    v.iter().position(|x| *x == n).unwrap_or_else(|| panic!("{n} missing from {v:?}"))
}

/// Define a component that logs into `$log`. `ext` holds config-derived data.
macro_rules! comp {
    ($log:ident, $name:ident, [$($n:ident: $d:ty),*], $cfg:ty, $out:ty, $ext:ty,
     |$cfgv:ident| $new:expr, |$self_:ident, $inp:ident| $body:expr) => {
        #[derive(ComponentSubscription)]
        #[subscribe($($n: $d),*)]
        pub struct $name {
            probe: Probe,
            state: u64,
            ext: $ext,
        }
        impl Component for $name {
            type Config = $cfg;
            type Output = $out;
            fn new($cfgv: $cfg) -> Self {
                $name { probe: Probe::new(&$log, stringify!($name)), state: 0, ext: $new }
            }
            fn update(&mut $self_, $inp: Self::Input<'_>) -> $out {
                $self_.probe.update();
                $self_.state += 1;
                $body
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Graph 1: diamond   Src -> {Double, Square} -> Sum;   App -> Sum, Double
// ---------------------------------------------------------------------------
mod diamond {
    use super::*;
    pub static LOG: Log = Mutex::new(Vec::new());

    comp!(LOG, Src, [], u64, u64, u64, |start| start, |self, _i| 10 + self.state + self.ext);
    comp!(LOG, Double, [x: Src], (), u64, (), |_c| (), |self, i| *i.x * 2);
    comp!(LOG, Square, [x: Src], (), u64, (), |_c| (), |self, i| *i.x * *i.x);
    comp!(LOG, Sum, [d: Double, q: Square, again: Double], (), u64, (), |_c| (),
          |self, i| { assert!(std::ptr::eq(i.d, i.again)); *i.d + *i.q });
    // The root: its output is what `run` returns.
    comp!(LOG, App, [sum: Sum, double: Double], (), (u64, u64), (), |_c| (),
          |self, i| (*i.sum, *i.double));
}

#[test]
fn diamond_lifecycle() {
    use diamond::*;
    let mut p = Pipeline::<App>::build(Configs::new().with::<Src>(0)).unwrap();
    let after_build = take(&LOG);
    assert!(after_build.iter().all(|(e, _)| *e == Ev::New));
    let built = names(&after_build, Ev::New);
    assert_eq!(built.len(), 5, "each component constructed once: {built:?}");
    // constructed after everything it depends on
    assert!(pos(&built, "Src") < pos(&built, "Double"));
    assert!(pos(&built, "Src") < pos(&built, "Square"));
    assert!(pos(&built, "Double") < pos(&built, "Sum"));
    assert!(pos(&built, "Square") < pos(&built, "Sum"));
    assert!(pos(&built, "Sum") < pos(&built, "App"));
    assert!(pos(&built, "Double") < pos(&built, "App"));
    assert_eq!(*built.last().unwrap(), "App");

    for run in 1..=3u64 {
        let (sum, double) = p.run();
        let log = take(&LOG);
        let upd = names(&log, Ev::Update);
        assert_eq!(upd.len(), 5, "run {run}: each component updated once: {upd:?}");
        assert_eq!(log.len(), 5, "run {run}: only updates, no drops: {log:?}");
        assert_eq!(*upd.last().unwrap(), "App");
        assert!(pos(&upd, "Src") < pos(&upd, "Double"));
        assert!(pos(&upd, "Src") < pos(&upd, "Square"));
        assert!(pos(&upd, "Double") < pos(&upd, "Sum"));
        assert!(pos(&upd, "Square") < pos(&upd, "Sum"));
        let src = 10 + run; // Src.state counts its own updates
        assert_eq!(double, src * 2);
        assert_eq!(sum, src * 2 + src * src);
    }

    drop(p);
    let log = take(&LOG);
    let mut dropped = names(&log, Ev::Drop);
    println!("diamond drop order: {dropped:?}");
    assert_eq!(log.len(), 5, "only drops after the pipeline is dropped: {log:?}");
    dropped.sort();
    assert_eq!(dropped, ["App", "Double", "Square", "Src", "Sum"], "each dropped exactly once");
}

// ---------------------------------------------------------------------------
// Graph 2: the example graph. App -> Report, Len<Tokenize>
// ---------------------------------------------------------------------------
mod words {
    use super::*;
    pub static LOG: Log = Mutex::new(Vec::new());

    comp!(LOG, Source, [], Vec<String>, String, Vec<String>, |lines| lines,
          |self, _i| self.ext[(self.state as usize - 1) % self.ext.len()].clone());
    comp!(LOG, Tokenize, [line: Source], bool, Vec<String>, bool, |lower| lower,
          |self, i| i.line.split_whitespace().map(|w| if self.ext { w.to_lowercase() } else { w.to_string() }).collect());
    comp!(LOG, WordCount, [tokens: Tokenize], (), HashMap<String, u64>, (), |_c| (),
          |self, i| { let mut m = HashMap::new(); for t in i.tokens { *m.entry(t.clone()).or_insert(0) += 1; } m });
    comp!(LOG, TopWords, [counts: WordCount], usize, Vec<(String, u64)>, usize, |n| n,
          |self, i| { let mut v: Vec<_> = i.counts.iter().map(|(k, v)| (k.clone(), *v)).collect();
                v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
                v.truncate(self.ext); v });
    comp!(LOG, Longest, [tokens: Tokenize], (), Option<String>, (), |_c| (),
          |self, i| i.tokens.iter().max_by(|a, b| a.len().cmp(&b.len()).then(b.cmp(a))).cloned());
    comp!(LOG, Report, [top: TopWords, longest: Longest, line: Source], (), String, (), |_c| (),
          |self, i| format!("line={:?} top={:?} longest={:?}", i.line, i.top, i.longest));

    // Generic component subscribing to its own type parameter.
    #[derive(ComponentSubscription)]
    #[subscribe(items: U)]
    pub struct Len<U: Component<Output = Vec<String>>> {
        probe: Probe,
        _u: PhantomData<U>,
    }
    impl<U: Component<Output = Vec<String>>> Component for Len<U> {
        type Config = ();
        type Output = usize;
        fn new((): ()) -> Self {
            Len { probe: Probe::new(&LOG, "Len<Tokenize>"), _u: PhantomData }
        }
        fn update(&mut self, input: Self::Input<'_>) -> usize {
            self.probe.update();
            input.items.len()
        }
    }
    comp!(LOG, App, [report: Report, count: Len<Tokenize>], (), (String, usize), (), |_c| (),
          |self, i| (i.report.clone(), *i.count));
}

#[test]
fn words_lifecycle() {
    use words::*;
    let configs = Configs::new()
        .with::<Source>(vec!["the quick brown fox jumps over the lazy dog".into(), "The dog sleeps and the fox runs".into()])
        .with::<Tokenize>(true)
        .with::<TopWords>(2);
    let mut p = Pipeline::<App>::build(configs).unwrap();
    let built_log = take(&LOG);
    let built = names(&built_log, Ev::New);
    assert_eq!(built.len(), 8, "{built:?}");
    assert_eq!(built_log.len(), 8);
    let deps_of: &[(&str, &[&str])] = &[
        ("Tokenize", &["Source"]), ("WordCount", &["Tokenize"]), ("TopWords", &["WordCount"]),
        ("Longest", &["Tokenize"]), ("Report", &["TopWords", "Longest", "Source"]), ("Len<Tokenize>", &["Tokenize"]),
        ("App", &["Report", "Len<Tokenize>"]),
    ];
    for (c, ds) in deps_of {
        for d in *ds {
            assert!(pos(&built, d) < pos(&built, c), "construct {d} before {c}");
        }
    }

    for run in 1..=2 {
        let (report, count) = p.run();
        let log = take(&LOG);
        let upd = names(&log, Ev::Update);
        assert_eq!(log.len(), 8, "run {run}: only updates, eight of them: {log:?}");
        for (c, ds) in deps_of {
            for d in *ds {
                assert!(pos(&upd, d) < pos(&upd, c), "run {run}: update {d} before {c}");
            }
        }
        if run == 1 {
            assert_eq!(report, r#"line="the quick brown fox jumps over the lazy dog" top=[("the", 2), ("brown", 1)] longest=Some("brown")"#);
            assert_eq!(count, 9);
        } else {
            assert_eq!(report, r#"line="The dog sleeps and the fox runs" top=[("the", 2), ("and", 1)] longest=Some("sleeps")"#);
            assert_eq!(count, 7);
        }
    }

    drop(p);
    let log = take(&LOG);
    assert_eq!(log.len(), 8);
    let mut dropped = names(&log, Ev::Drop);
    dropped.sort();
    assert_eq!(dropped, ["App", "Len<Tokenize>", "Longest", "Report", "Source", "Tokenize", "TopWords", "WordCount"]);
}

// ---------------------------------------------------------------------------
// Movability and Send
// ---------------------------------------------------------------------------
fn assert_send<T: Send>() {}

mod mv {
    use super::*;
    #[derive(ComponentSubscription)]
    pub struct A(u64);
    impl Component for A {
        type Config = u64;
        type Output = u64;
        fn new(c: u64) -> Self { A(c) }
        fn update(&mut self, _: Self::Input<'_>) -> u64 { self.0 += 1; self.0 }
    }
    #[derive(ComponentSubscription)]
    #[subscribe(a: A)]
    pub struct B(String);
    impl Component for B {
        type Config = ();
        type Output = String;
        fn new((): ()) -> Self { B(String::new()) }
        fn update(&mut self, i: Self::Input<'_>) -> String { self.0.push('x'); format!("{}{}", i.a, self.0) }
    }
}

#[test]
fn pipeline_is_send_and_movable() {
    use mv::*;
    assert_send::<Pipeline<diamond::App>>();
    assert_send::<Pipeline<words::App>>();

    let mut p = Pipeline::<B>::build(Configs::new().with::<A>(100)).unwrap();
    assert_eq!(p.run(), "101x");
    // move after a run (state must travel with it), box it, send it to another thread, run there
    let boxed = Box::new(p);
    let mut p = *boxed;
    let h = std::thread::spawn(move || p.run());
    assert_eq!(h.join().unwrap(), "102xx");
}

// ---------------------------------------------------------------------------
// Panicking constructor: every already-built component is dropped exactly
// once (by the compiler's unwinding of the partly built tree).
// ---------------------------------------------------------------------------
mod boom {
    use super::*;
    pub static LOG: Log = Mutex::new(Vec::new());
    comp!(LOG, First, [], (), u64, (), |_c| (), |self, _i| 1);
    comp!(LOG, Second, [a: First], (), u64, (), |_c| (), |self, _i| 2);
    #[derive(ComponentSubscription)]
    #[subscribe(a: Second)]
    pub struct Bad;
    impl Component for Bad {
        type Config = ();
        type Output = u64;
        fn new((): ()) -> Self { panic!("constructor failed") }
        fn update(&mut self, _: Self::Input<'_>) -> u64 { 3 }
    }
}

#[test]
fn constructor_panic_never_double_drops() {
    use boom::*;
    let r = std::panic::catch_unwind(|| Pipeline::<Bad>::build(Configs::new()).map(|_| ()));
    assert!(r.is_err());
    let log = take(&LOG);
    let news = names(&log, Ev::New);
    let drops = names(&log, Ev::Drop);
    assert_eq!(news, ["First", "Second"]);
    // What was built before the panic is dropped by unwinding, each exactly once.
    println!("constructor panic: built {news:?}, dropped {drops:?}");
    let mut d = drops.clone();
    d.sort();
    assert_eq!(d, ["First", "Second"], "built components dropped exactly once");
}

// ---------------------------------------------------------------------------
// Over-aligned components: the storage honours their alignment.
// ---------------------------------------------------------------------------
mod aligned {
    use super::*;
    #[repr(align(64))]
    pub struct Big([u8; 64]);
    #[derive(ComponentSubscription)]
    pub struct Wide(Big);
    impl Component for Wide {
        type Config = ();
        type Output = usize;
        fn new((): ()) -> Self { Wide(Big([7; 64])) }
        fn update(&mut self, _: Self::Input<'_>) -> usize {
            assert_eq!(self as *const Wide as usize % 64, 0, "component misaligned");
            self.0 .0[0] as usize
        }
    }
    #[derive(ComponentSubscription)]
    #[subscribe(w: Wide)]
    pub struct After(u8);
    impl Component for After {
        type Config = ();
        type Output = usize;
        fn new((): ()) -> Self { After(1) }
        fn update(&mut self, i: Self::Input<'_>) -> usize { *i.w + self.0 as usize }
    }
}

#[test]
fn over_aligned_components() {
    use aligned::*;
    for _ in 0..8 {
        // Move the pipeline around on the stack / heap between runs.
        let mut p = Box::new(Pipeline::<After>::build(Configs::new()).unwrap());
        assert_eq!(p.run(), 8);
        let mut q = *p;
        q.run();
    }
}
