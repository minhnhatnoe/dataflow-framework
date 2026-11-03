//! Set subscriptions together with guards (conditions asked again, see `guard.rs`): a set whose
//! elements fire only under a guard (`Pick::<S>(Odd(u))`: the key, then a decision on the value),
//! its reader, an active handler on a component downstream of such a guard, and a set reader
//! inside a guarded block with a guarded component after it. Reads see the last stored values,
//! components run in order and at most once per push; the log of many pushes is compared with a
//! model. No `#![feature]` here.

use std::marker::PhantomData;
use std::sync::Mutex;

use dataflow_flat::{
    catalog, Component, ComponentSubscription, Configs, Initial, Pipeline, Root, Select, UpdateHandler, GUARD,
};

static LOG: Mutex<Vec<String>> = Mutex::new(Vec::new());
static SERIAL: Mutex<()> = Mutex::new(());
fn log(s: impl Into<String>) {
    LOG.lock().unwrap().push(s.into());
}
fn take() -> Vec<String> {
    std::mem::take(&mut *LOG.lock().unwrap())
}

#[derive(Root)]
pub struct Mg {
    pub key: u32,
    pub v: u64,
}

pub struct Sy<const K: usize>;
pub trait Num: 'static {
    const KEY: u32;
}
impl<const K: usize> Num for Sy<K> {
    const KEY: u32 = K as u32;
}

/// The value of the message with key `S::KEY`.
pub struct Pick<S>(PhantomData<fn() -> S>);
impl<S: Num> Select<Mg> for Pick<S> {
    type Out = u64;
    const KEY: u32 = S::KEY;
    fn select(x: &Mg) -> Option<&u64> {
        (x.key == S::KEY).then_some(&x.v)
    }
    fn key_of(x: &Mg) -> u32 {
        x.key
    }
}
impl<S: Num> Initial<Mg> for Pick<S> {
    fn initial() -> u64 {
        0
    }
}
/// A guard on a value: odd.
pub struct Odd;
impl Select<u64> for Odd {
    type Out = u64;
    const KEY: u32 = GUARD;
    fn select(x: &u64) -> Option<&u64> {
        (x % 2 == 1).then_some(x)
    }
    fn key_of(_: &u64) -> u32 {
        GUARD
    }
}
/// A guard on a stored output: a multiple of 4.
pub struct Four;
impl Select<u64> for Four {
    type Out = u64;
    const KEY: u32 = GUARD;
    fn select(x: &u64) -> Option<&u64> {
        (x % 4 == 0).then_some(x)
    }
    fn key_of(_: &u64) -> u32 {
        GUARD
    }
}

/// Fires on its key and an odd value.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Pick::<S>(Odd(u)): Mg)]
pub struct Gd<S: Num>(PhantomData<S>);
impl<S: Num> Component for Gd<S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Gd(PhantomData), 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("Gd{}", S::KEY));
        *i.u * 10 + S::KEY as u64
    }
}

macro_rules! cat {
    () => {
        catalog![Sy<0>, Sy<1>, Sy<2>]
    };
}

/// Reads the set: every element's last stored value.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(g: for S: Num in cat!() { Gd<S> })]
pub struct Rd;
impl Component for Rd {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Rd, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let mut all = Vec::new();
        i.g.for_each(|v: &u64| all.push(*v));
        log(format!("Rd {all:?}"));
        all.iter().sum()
    }
}

/// Always runs; has an active handler on `Gd<Sy<1>>`, which sits behind a guard.
#[derive(ComponentSubscription)]
#[subscribe(m: Mg, x: Gd<Sy<1>>, r: Rd)]
pub struct Hd;
impl Component for Hd {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Hd, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("Hd x={} r={}", i.x, i.r));
        0
    }
}
impl UpdateHandler<Mg> for Hd {
    const ACTIVE: bool = false;
}
impl UpdateHandler<Rd> for Hd {
    const ACTIVE: bool = false;
}
impl UpdateHandler<Gd<Sy<1>>> for Hd {
    fn on_update(&mut self, _: Self::Input<'_>, v: &u64) {
        log(format!("Hd.on {v}"));
    }
}

/// A second reader of the same set, a block of its own: runs when some element fired.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(g: for S: Num in cat!() { Gd<S> })]
pub struct Dn;
impl Component for Dn {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Dn, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let s = i.g.fold(0u64, |a, v| a + *v);
        log(format!("Dn {s}"));
        s
    }
}

/// Behind a guard on `Dn`'s stored output: a stale `Dn` that matches must not run it.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Four(d): Dn)]
pub struct Tl;
impl Component for Tl {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {
        (Tl, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        log(format!("Tl {}", i.d));
        0
    }
}

#[test]
fn set_readers_and_handlers_behind_guards() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let mut p = Pipeline::<Mg, (Hd, Tl)>::build(Configs::new()).unwrap();
    // The three elements, `Rd`, `Hd`, `Dn`, `Tl`.
    assert_eq!(p.len(), 3 + 4);
    take();
    let mut out = [0u64; 3];
    let mut rd = 0u64;
    let mut dn = 0u64;
    let mut seq: Vec<(u32, u64)> = Vec::new();
    for key in 0..4 {
        for v in [1, 2, 3, 4, 5, 6] {
            seq.push((key, v));
        }
    }
    for k in 0..500u32 {
        let h = k.wrapping_mul(2654435761) >> 9;
        seq.push((h % 4, ((h >> 3) % 40) as u64));
    }
    for (key, v) in seq {
        let mut want: Vec<String> = Vec::new();
        let ran = key < 3 && v % 2 == 1;
        // the listing order: Gd1 (named by `Hd` first), Gd0, Gd2, Rd, Hd, Dn, Tl
        let mut fire = |k: usize, out: &mut [u64; 3], want: &mut Vec<String>| {
            if ran && key as usize == k {
                out[k] = v * 10 + k as u64;
                want.push(format!("Gd{k}"));
            }
        };
        fire(1, &mut out, &mut want);
        fire(0, &mut out, &mut want);
        fire(2, &mut out, &mut want);
        if ran {
            want.push(format!("Rd {out:?}"));
            rd = out.iter().sum();
            if key == 1 {
                want.push(format!("Hd.on {}", out[1]));
            }
        }
        want.push(format!("Hd x={} r={}", out[1], rd));
        if ran {
            dn = out.iter().sum();
            want.push(format!("Dn {dn}"));
            if dn % 4 == 0 {
                want.push(format!("Tl {dn}"));
            }
        }
        p.push(Mg { key, v });
        assert_eq!(take(), want, "key = {key}, v = {v}");
    }
}
