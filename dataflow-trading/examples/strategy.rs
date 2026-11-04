//! Synthetic prices demonstrate wiring; this is not an exchange connection.
use dataflow_flat::{Component, ComponentSubscription, Root};
use dataflow_trading::*;
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

pub struct VenueA;
impl Exchange for VenueA {
    type Order = Order;
}
pub struct Btc;
impl Symbol for Btc {
    const NAME: &'static str = "BTC";
}
pub struct Eth;
impl Symbol for Eth {
    const NAME: &'static str = "ETH";
}

#[derive(Root)]
pub struct Tick {
    pub price: f64,
}

#[book]
#[source(event: Tick)]
pub struct Depth<E: Exchange, S: Symbol> {
    marker: PhantomData<(E, S)>,
}
pub struct DepthState {
    pub prices: Vec<f64>,
}
impl<E: Exchange, S: Symbol> Book for Depth<E, S> {
    type Config = ();
    type State = DepthState;
    fn new(_: ()) -> (Self, Self::State) {
        (
            Self {
                marker: PhantomData,
            },
            DepthState {
                prices: Vec::with_capacity(16),
            },
        )
    }
    fn update(&mut self, i: Self::Inputs<'_>, state: &mut Self::State) {
        state.prices.push(i.event.price);
    }
    fn ready(&self, state: &Self::State) -> bool {
        !state.prices.is_empty()
    }
}

#[book]
#[source(event: Tick)]
pub struct Positions<E: Exchange> {
    marker: PhantomData<E>,
}
impl<E: Exchange> Book for Positions<E> {
    type Config = ();
    type State = f64;
    fn new(_: ()) -> (Self, f64) {
        (
            Self {
                marker: PhantomData,
            },
            0.0,
        )
    }
    fn update(&mut self, _: Self::Inputs<'_>, _: &mut f64) {}
}

#[derive(SignalOutputs)]
pub struct FairOutput {
    pub price: f64,
    pub confidence: f64,
}
#[signal]
#[inputs(depth: Depth<E,S>)]
pub struct Fair<E: Exchange, S: Symbol> {
    marker: PhantomData<(E, S)>,
}
impl<E: Exchange, S: Symbol> Signal for Fair<E, S> {
    type Config = ();
    type Output = FairOutput;
    fn new(_: ()) -> Self {
        Self {
            marker: PhantomData,
        }
    }
    fn compute(&mut self, i: Self::Inputs<'_>) -> FairOutput {
        FairOutput {
            price: *i.depth.prices.last().unwrap(),
            confidence: 1.0,
        }
    }
}

#[quote_engine(exchange=E, symbol=S)]
#[inputs(fair: Fair<E,S> => price, positions: Positions<E>)]
pub struct Quoter<E: Exchange, S: Symbol> {
    marker: PhantomData<(E, S)>,
}
impl<E: Exchange, S: Symbol> QuoteEngine for Quoter<E, S> {
    type Config = ();
    fn new(_: ()) -> Self {
        Self {
            marker: PhantomData,
        }
    }
    fn compute(&mut self, i: Self::Inputs<'_>) -> Quote {
        (
            Some(i.fair - 1.0 - i.positions),
            Some(i.fair + 1.0 - i.positions),
        )
    }
}

#[oms(exchange=E)]
#[inputs(positions: Positions<E>)]
#[quotes(btc: Quoter<E,Btc>, eth: Quoter<E,Eth>)]
pub struct Manager<E: Exchange<Order = Order>> {
    marker: PhantomData<E>,
}
#[derive(Debug, PartialEq)]
pub struct Order {
    pub symbol: &'static str,
    pub quote: Quote,
}
impl<E: Exchange<Order = Order>> Oms for Manager<E> {
    type Config = ();
    fn new(_: ()) -> Self {
        Self {
            marker: PhantomData,
        }
    }
    fn compute(&mut self, _: Self::Inputs<'_>, quotes: Quotes<'_, E>) -> Vec<Order> {
        quotes
            .updated()
            .map(|q| Order {
                symbol: q.symbol.name,
                quote: q.quote,
            })
            .collect()
    }
}

#[derive(ComponentSubscription, dataflow_flat::UpdateHandler)]
#[subscribe(batch: Manager<VenueA>)]
pub struct Route {
    log: Rc<RefCell<Vec<Order>>>,
}
impl Component for Route {
    type Config = Rc<RefCell<Vec<Order>>>;
    type Output = ();
    fn new(log: Self::Config, _: Self::Init<'_>) -> (Self, ()) {
        (Self { log }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        self.log
            .borrow_mut()
            .extend(i.batch.orders.iter().map(|x| Order {
                symbol: x.symbol,
                quote: x.quote,
            }));
    }
}
fn main() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut p =
        Pipeline::<Tick, (Route,)>::build(Configs::new().with::<Route>(log.clone())).unwrap();
    p.push(Tick { price: 100.0 });
    println!("{:?}", log.borrow());
}
