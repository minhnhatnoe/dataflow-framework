//! `Book<X, S>`: one order book per exchange and symbol.
//!
//! The non-generic-core pattern: the logic lives in [`BookCore`], compiled once.
//! The generic `Book<X, S>` is only a wrapper that carries the types (and
//! therefore the subscription `Ev::Btc(b): X::Ws`-style pattern), so the 2 x N instances add
//! almost nothing to compile time.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use dataflow_flat::{Component, ComponentSubscription, UpdateHandler, Select};

use crate::market::*;

/// The book's state and logic, independent of exchange and symbol.
#[derive(Default)]
pub struct BookCore {
    pub updates: u64,
}
impl BookCore {
    pub fn apply(&mut self, t: &Tob) -> Tob {
        self.updates += 1;
        *t
    }
}

thread_local! {
    /// How many times each book ran (the example checks it against the wire).
    pub static BOOK_RUNS: RefCell<BTreeMap<(&'static str, &'static str), u64>> = RefCell::new(BTreeMap::new());
}

/// Runs only for packets of exchange `X` with symbol `S`.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(S(t)): <X as Exchange>::Ws)]
pub struct Book<X: Exchange, S: Market + Select<X::Frame, Out = Tob>> {
    core: BookCore,
    _p: PhantomData<(X, S)>,
}
impl<X: Exchange, S: Market + Select<X::Frame, Out = Tob>> Component for Book<X, S> {
    type Config = ();
    type Output = Tob;
    fn new((): (), _: Self::Init<'_>) -> (Self, Tob) {
        (Book { core: BookCore::default(), _p: PhantomData }, Tob::default())
    }
    fn update(&mut self, i: Self::Input<'_>) -> Tob {
        BOOK_RUNS.with(|m| *m.borrow_mut().entry((X::NAME, <S as Market>::NAME)).or_default() += 1);
        self.core.apply(i.t)
    }
}
