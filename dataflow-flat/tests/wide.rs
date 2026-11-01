//! Lifecycle and values for wide components (stored as a tree of tuples): a
//! root with 100 subscriptions, and a 22-subscription component whose entries
//! read each other's upstream across node boundaries, and components of 9 and 15
//! subscriptions (an inner node with empty children).

use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::Mutex;

use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline, Root};

#[derive(Root)]
pub struct Poll;

/// The output of the last component that ran (the sink, which runs last).
static RESULT: AtomicUsize = AtomicUsize::new(0);
fn result() -> u64 {
    RESULT.load(SeqCst) as u64
}

static SERIAL: Mutex<()> = Mutex::new(());
static NEW: AtomicUsize = AtomicUsize::new(0);
static UPDATE: AtomicUsize = AtomicUsize::new(0);
static DROP: AtomicUsize = AtomicUsize::new(0);

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

fn counts() -> (usize, usize, usize) {
    (NEW.load(SeqCst), UPDATE.load(SeqCst), DROP.load(SeqCst))
}
fn reset() {
    NEW.store(0, SeqCst);
    UPDATE.store(0, SeqCst);
    DROP.store(0, SeqCst);
}

/// `Name(n)`: output `n`, no subscriptions.
macro_rules! leaf {
    ($name:ident, $n:expr) => {
        #[derive(ComponentSubscription)]
        #[subscribe(p: Poll)]
        pub struct $name(Life);
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
                ($name(Life::new()), $n)
            }
            fn update(&mut self, _: Self::Input<'_>) -> u64 {
                UPDATE.fetch_add(1, SeqCst);
                $n
            }
        }
    };
}

/// Output: the sum of all inputs, weighted by position (so a swapped or wrong
/// input changes the result), plus `$base`.
macro_rules! summing {
    ($name:ident, $base:expr, [$($f:ident: $d:ty),*]) => {
        #[derive(ComponentSubscription)]
        #[subscribe($($f: $d),*)]
        pub struct $name(Life);
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
                ($name(Life::new()), 0)
            }
            fn update(&mut self, input: Self::Input<'_>) -> u64 {
                UPDATE.fetch_add(1, SeqCst);
                let mut acc: u64 = $base;
                let mut w = 1;
                $(acc += w * *input.$f; w += 1;)*
                let _ = w;
                RESULT.store(acc as usize, SeqCst);
                acc
            }
        }
    };
}

leaf!(L0, 1);
leaf!(L1, 2);
leaf!(L2, 3);
leaf!(L3, 4);
leaf!(L4, 5);
leaf!(L5, 6);
leaf!(L6, 7);
leaf!(L7, 8);
leaf!(L8, 9);
leaf!(L9, 10);
leaf!(L10, 11);
leaf!(L11, 12);
leaf!(L12, 13);
leaf!(L13, 14);
leaf!(L14, 15);
leaf!(L15, 16);
leaf!(L16, 17);
leaf!(L17, 18);
leaf!(L18, 19);
leaf!(L19, 20);
leaf!(L20, 21);
leaf!(L21, 22);
leaf!(L22, 23);
leaf!(L23, 24);
leaf!(L24, 25);
leaf!(L25, 26);
leaf!(L26, 27);
leaf!(L27, 28);
leaf!(L28, 29);
leaf!(L29, 30);
leaf!(L30, 31);
leaf!(L31, 32);
leaf!(L32, 33);
leaf!(L33, 34);
leaf!(L34, 35);
leaf!(L35, 36);
leaf!(L36, 37);
leaf!(L37, 38);
leaf!(L38, 39);
leaf!(L39, 40);
leaf!(L40, 41);
leaf!(L41, 42);
leaf!(L42, 43);
leaf!(L43, 44);
leaf!(L44, 45);
leaf!(L45, 46);
leaf!(L46, 47);
leaf!(L47, 48);
leaf!(L48, 49);
leaf!(L49, 50);
leaf!(L50, 51);
leaf!(L51, 52);
leaf!(L52, 53);
leaf!(L53, 54);
leaf!(L54, 55);
leaf!(L55, 56);
leaf!(L56, 57);
leaf!(L57, 58);
leaf!(L58, 59);
leaf!(L59, 60);
leaf!(L60, 61);
leaf!(L61, 62);
leaf!(L62, 63);
leaf!(L63, 64);
leaf!(L64, 65);
leaf!(L65, 66);
leaf!(L66, 67);
leaf!(L67, 68);
leaf!(L68, 69);
leaf!(L69, 70);
leaf!(L70, 71);
leaf!(L71, 72);
leaf!(L72, 73);
leaf!(L73, 74);
leaf!(L74, 75);
leaf!(L75, 76);
leaf!(L76, 77);
leaf!(L77, 78);
leaf!(L78, 79);
leaf!(L79, 80);
leaf!(L80, 81);
leaf!(L81, 82);
leaf!(L82, 83);
leaf!(L83, 84);
leaf!(L84, 85);
leaf!(L85, 86);
leaf!(L86, 87);
leaf!(L87, 88);
leaf!(L88, 89);
leaf!(L89, 90);
leaf!(L90, 91);
leaf!(L91, 92);
leaf!(L92, 93);
leaf!(L93, 94);
leaf!(L94, 95);
leaf!(L95, 96);
leaf!(L96, 97);
leaf!(L97, 98);
leaf!(L98, 99);
leaf!(L99, 100);

summing!(Mid, 1000, [a: L3, b: L40, c: L99]);
summing!(Mid2, 5000, [a: L60, m: Mid, b: L1]);
summing!(Root100, 0, [r0: L0, r1: L1, r2: L2, r3: L3, r4: L4, r5: L5, r6: L6, r7: L7, r8: L8, r9: L9, r10: L10, r11: L11, r12: L12, r13: L13, r14: L14, r15: L15, r16: L16, r17: L17, r18: L18, r19: L19, r20: L20, r21: L21, r22: L22, r23: L23, r24: L24, r25: L25, r26: L26, r27: L27, r28: L28, r29: L29, r30: L30, r31: L31, r32: L32, r33: L33, r34: L34, r35: L35, r36: L36, r37: L37, r38: L38, r39: L39, r40: L40, r41: L41, r42: L42, r43: L43, r44: L44, r45: L45, r46: L46, r47: L47, r48: L48, r49: L49, r50: L50, r51: L51, r52: L52, r53: L53, r54: L54, r55: L55, r56: L56, r57: L57, r58: L58, r59: L59, r60: L60, r61: L61, r62: L62, r63: L63, r64: L64, r65: L65, r66: L66, r67: L67, r68: L68, r69: L69, r70: L70, r71: L71, r72: L72, r73: L73, r74: L74, r75: L75, r76: L76, r77: L77, r78: L78, r79: L79, r80: L80, r81: L81, r82: L82, r83: L83, r84: L84, r85: L85, r86: L86, r87: L87, r88: L88, r89: L89, r90: L90, r91: L91, r92: L92, r93: L93, r94: L94, r95: L95, r96: L96, r97: L97, r98: L98, r99: L99]);
// 22 unique subscriptions, fill-first: leaf 0 = indices 0..7, leaf 1 = 7..14,
// leaf 2 = 14..21, leaf 3 = index 21. `m: Mid2` (leaf 0) reaches L1 and L3 first;
// `r1: L1` is read in leaf 2 and `r3: L3` in leaf 3, so those inputs are found
// in an earlier node of the same list.
summing!(Wide22, 7, [a: L0, m: Mid2, r2: L2, r4: L4, r5: L5, r6: L6, r7: L7, r8: L8, r9: L9, r10: L10, r11: L11, r12: L12, r13: L13, r14: L14, r15: L15, r16: L16, r17: L17, r18: L18, r19: L19, r20: L20, r1: L1, r3: L3]);
// 9 subscriptions: ((L0..L6), (L7, L8), (), (), (), (), (), ()).
summing!(Nine, 0, [n0: L0, n1: L1, n2: L2, n3: L3, n4: L4, n5: L5, n6: L6, n7: L7, n8: L8]);
// 15 subscriptions: ((L0..L6), (L7..L13), (L14), (), (), (), (), ()).
summing!(Fifteen, 0, [f0: L0, f1: L1, f2: L2, f3: L3, f4: L4, f5: L5, f6: L6, f7: L7, f8: L8, f9: L9, f10: L10, f11: L11, f12: L12, f13: L13, f14: L14]);
summing!(App, 0, [w: Wide22, x: Root100, m: Mid]);

#[test]
fn hundred_subscriptions_run_once_each_with_the_right_inputs() {
    let _g = SERIAL.lock().unwrap();
    reset();
    let mut p = Pipeline::<Poll, (Root100,)>::build(Configs::new()).unwrap();
    assert_eq!(counts(), (101, 0, 0));
    for run in 1..=3 {
        // sum over i of (i + 1) * (i + 1)
        let want: u64 = (1..=100u64).map(|k| k * k).sum();
        p.push(Poll);
        assert_eq!(result(), want);
        assert_eq!(counts(), (101, 101 * run, 0));
    }
    drop(p);
    assert_eq!(counts(), (101, 303, 101));
}

#[test]
fn entries_read_upstream_found_in_earlier_nodes() {
    let _g = SERIAL.lock().unwrap();
    reset();
    let mut p = Pipeline::<Poll, (App,)>::build(Configs::new()).unwrap();
    // L0..L99 (100) + Mid + Mid2 + Wide22 + Root100 + App
    assert_eq!(p.len(), 105);
    assert_eq!(counts().0, 105);
    let mid: u64 = 1000 + 1 * 4 + 2 * 41 + 3 * 100;
    let mid2: u64 = 5000 + 61 + 2 * mid + 3 * 2;
    // The values in field order of `Wide22`: L0, Mid2, then L2, L4..L20, L1, L3.
    let mut vals = vec![1, mid2];
    vals.extend([2u64, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 1, 3].iter().map(|i| i + 1));
    let wide22: u64 = 7 + vals.iter().enumerate().map(|(k, v)| (k as u64 + 1) * v).sum::<u64>();
    let root100: u64 = (1..=100u64).map(|k| k * k).sum();
    let want = wide22 + 2 * root100 + 3 * mid;
    for run in 1..=2 {
        p.push(Poll);
        assert_eq!(result(), want);
        assert_eq!(counts(), (105, 105 * run, 0));
    }
    drop(p);
    assert_eq!(counts(), (105, 210, 105));
}

/// Inner nodes whose later children are empty (`()`): every component built,
/// updated and dropped once, with the right inputs in the right fields.
#[test]
fn nine_and_fifteen_subscriptions() {
    let _g = SERIAL.lock().unwrap();
    reset();
    let mut p = Pipeline::<Poll, (Nine,)>::build(Configs::new()).unwrap();
    assert_eq!(counts(), (10, 0, 0));
    for run in 1..=2 {
        p.push(Poll);
        assert_eq!(result(), (1..=9u64).map(|k| k * k).sum::<u64>());
        assert_eq!(counts(), (10, 10 * run, 0));
    }
    drop(p);
    assert_eq!(counts(), (10, 20, 10));

    reset();
    let mut p = Pipeline::<Poll, (Fifteen,)>::build(Configs::new()).unwrap();
    assert_eq!(counts(), (16, 0, 0));
    for run in 1..=2 {
        p.push(Poll);
        assert_eq!(result(), (1..=15u64).map(|k| k * k).sum::<u64>());
        assert_eq!(counts(), (16, 16 * run, 0));
    }
    drop(p);
    assert_eq!(counts(), (16, 32, 16));
}
