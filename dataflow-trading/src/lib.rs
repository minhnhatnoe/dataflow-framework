//! Typed strategy interfaces over dataflow-flat. See the crate README.
#![forbid(unsafe_code)]
pub use dataflow_flat::{Configs, Pipeline};
pub use dataflow_trading_macros::{book, oms, quote_engine, signal, SignalOutputs};
use std::{any::TypeId, marker::PhantomData};

pub trait Inputs: 'static {
    type Inputs<'a>;
}
pub trait Exchange: 'static {
    type Order: 'static;
}
pub trait Symbol: 'static {
    const NAME: &'static str;
}
pub type Quote = (Option<f64>, Option<f64>);
/// Implemented by the derive; named f64 fields form the output schema.
pub trait SignalOutputs: 'static {
    const LABELS: &'static [&'static str];
}
pub trait Book: Inputs + Sized {
    type Config: Default + 'static;
    type State: 'static;
    fn new(config: Self::Config) -> (Self, Self::State);
    fn update(&mut self, input: Self::Inputs<'_>, state: &mut Self::State);
    /// Readiness is independent of whether the state is empty.
    fn ready(&self, _state: &Self::State) -> bool {
        true
    }
}
pub trait Signal: Inputs + Sized {
    type Config: Default + 'static;
    type Output: SignalOutputs;
    fn new(config: Self::Config) -> Self;
    fn compute(&mut self, input: Self::Inputs<'_>) -> Self::Output;
}
pub trait Market: 'static {
    type Exchange: Exchange;
    type Symbol: Symbol;
}
pub trait QuoteEngine: Inputs + Market + Sized {
    type Config: Default + 'static;
    fn new(config: Self::Config) -> Self;
    fn compute(&mut self, input: Self::Inputs<'_>) -> Quote;
}
pub trait Venue: 'static {
    type Exchange: Exchange;
}
pub trait Oms: Inputs + Venue + Sized {
    type Config: Default + 'static;
    fn new(config: Self::Config) -> Self;
    fn compute(
        &mut self,
        input: Self::Inputs<'_>,
        quotes: Quotes<'_, Self::Exchange>,
    ) -> Vec<<Self::Exchange as Exchange>::Order>;
}
/// Persistent published state. An unavailable value is never replaced by a fake zero.
pub struct Published<T> {
    pub value: Option<T>,
    pub ready: bool,
    pub revision: u64,
}
impl<T> Default for Published<T> {
    fn default() -> Self {
        Self {
            value: None,
            ready: false,
            revision: 0,
        }
    }
}
impl<T> Published<T> {
    pub fn get(&self) -> Option<&T> {
        if self.ready {
            self.value.as_ref()
        } else {
            None
        }
    }
    #[doc(hidden)]
    pub fn publish(&mut self, value: T) {
        self.value = Some(value);
        self.ready = true;
        self.revision = self.revision.checked_add(1).expect("revision overflow");
    }
}
/// Marks the published value type of a trading node.
pub trait ValueNode: dataflow_flat::Component<Output = Published<Self::Value>> {
    type Value: 'static;
}
pub trait BookNode: ValueNode {}
pub trait FeatureNode: ValueNode {}
impl<T: BookNode> FeatureNode for T {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SymbolId {
    id: TypeId,
    pub name: &'static str,
}
impl SymbolId {
    pub fn of<S: Symbol>() -> Self {
        Self {
            id: TypeId::of::<S>(),
            name: S::NAME,
        }
    }
}
pub struct QuoteEntry {
    pub symbol: SymbolId,
    pub quote: Quote,
    pub updated: bool,
}
pub struct Quotes<'a, E: Exchange> {
    entries: &'a [QuoteEntry],
    marker: PhantomData<E>,
}
impl<'a, E: Exchange> Quotes<'a, E> {
    #[doc(hidden)]
    pub fn new(entries: &'a [QuoteEntry]) -> Self {
        Self {
            entries,
            marker: PhantomData,
        }
    }
    pub fn get(&self, symbol: SymbolId) -> Option<Quote> {
        self.entries
            .iter()
            .find(|x| x.symbol == symbol)
            .map(|x| x.quote)
    }
    pub fn iter(&self) -> impl Iterator<Item = &QuoteEntry> {
        self.entries.iter()
    }
    pub fn updated(&self) -> impl Iterator<Item = &QuoteEntry> {
        self.entries.iter().filter(|x| x.updated)
    }
}
/// Only the current invocation's commands; routing sinks must consume once per push.
pub struct Orders<T> {
    pub orders: Vec<T>,
    #[doc(hidden)]
    pub revisions: Vec<u64>,
}
impl<T> Default for Orders<T> {
    fn default() -> Self {
        Self {
            orders: Vec::new(),
            revisions: Vec::new(),
        }
    }
}
#[doc(hidden)]
pub fn quote_symbol<Q: QuoteEngine<Exchange = E>, E: Exchange>() -> SymbolId {
    SymbolId::of::<Q::Symbol>()
}
#[doc(hidden)]
pub fn require_book<T: BookNode>() {}
#[doc(hidden)]
pub fn require_feature<T: FeatureNode>() {}
