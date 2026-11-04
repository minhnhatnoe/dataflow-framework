#[allow(dead_code)]
#[path = "../examples/strategy.rs"]
mod strategy;
use dataflow_flat::{Component, ComponentSubscription};
use dataflow_trading::*;
use std::{cell::RefCell, rc::Rc};
use strategy::*;

#[test]
fn generic_strategy_discovers_shared_books_and_routes_once() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut p =
        Pipeline::<Tick, (Route,)>::build(Configs::new().with::<Route>(log.clone())).unwrap();
    assert_eq!(
        p.order()
            .iter()
            .filter(|n| n.contains("Positions<"))
            .count(),
        1
    );
    assert!(log.borrow().is_empty());
    p.push(Tick { price: 100.0 });
    p.push(Tick { price: 110.0 });
    assert_eq!(
        *log.borrow(),
        vec![
            Order {
                symbol: "BTC",
                quote: (Some(99.0), Some(101.0))
            },
            Order {
                symbol: "ETH",
                quote: (Some(99.0), Some(101.0))
            },
            Order {
                symbol: "BTC",
                quote: (Some(109.0), Some(111.0))
            },
            Order {
                symbol: "ETH",
                quote: (Some(109.0), Some(111.0))
            },
        ]
    );
    assert_eq!(FairOutput::LABELS, &["price", "confidence"]);
}

#[derive(ComponentSubscription, dataflow_flat::UpdateHandler)]
#[subscribe(depth: Depth<VenueA,Btc>)]
struct AllocationProbe {
    log: Rc<RefCell<Vec<usize>>>,
}
impl Component for AllocationProbe {
    type Config = Rc<RefCell<Vec<usize>>>;
    type Output = ();
    fn new(log: Self::Config, _: Self::Init<'_>) -> (Self, ()) {
        (Self { log }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        self.log
            .borrow_mut()
            .push(i.depth.get().unwrap().prices.as_ptr() as usize);
    }
}
#[test]
fn non_clone_book_keeps_its_allocation() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut p = Pipeline::<Tick, (AllocationProbe,)>::build(
        Configs::new().with::<AllocationProbe>(log.clone()),
    )
    .unwrap();
    for price in 0..8 {
        p.push(Tick {
            price: price as f64,
        });
    }
    let log = log.borrow();
    assert_eq!(log.len(), 8);
    assert!(log.iter().all(|p| *p == log[0]));
}

struct TestExchange;
impl Exchange for TestExchange {
    type Order = f64;
}
#[book]
#[source(event: Tick)]
struct Gated;
impl Book for Gated {
    type Config = ();
    type State = f64;
    fn new(_: ()) -> (Self, f64) {
        (Self, -1.0)
    }
    fn update(&mut self, i: Self::Inputs<'_>, s: &mut f64) {
        *s = i.event.price;
    }
    fn ready(&self, s: &f64) -> bool {
        *s >= 0.0
    }
}
#[signal]
#[inputs(value: Gated)]
struct Count {
    calls: u32,
}
#[derive(SignalOutputs)]
pub struct CountOutput {
    pub calls: f64,
}
impl Signal for Count {
    type Config = ();
    type Output = CountOutput;
    fn new(_: ()) -> Self {
        Self { calls: 0 }
    }
    fn compute(&mut self, _: Self::Inputs<'_>) -> CountOutput {
        self.calls += 1;
        CountOutput {
            calls: self.calls as f64,
        }
    }
}
#[quote_engine(exchange=TestExchange,symbol=Btc)]
#[inputs(count: Count => calls)]
struct CountQuote;
impl QuoteEngine for CountQuote {
    type Config = ();
    fn new(_: ()) -> Self {
        Self
    }
    fn compute(&mut self, i: Self::Inputs<'_>) -> Quote {
        (Some(*i.count), None)
    }
}
#[oms(exchange=TestExchange)]
#[quotes(count: CountQuote)]
struct CountOms;
impl Oms for CountOms {
    type Config = ();
    fn new(_: ()) -> Self {
        Self
    }
    fn compute(&mut self, _: Self::Inputs<'_>, q: Quotes<'_, TestExchange>) -> Vec<f64> {
        assert_eq!(q.updated().count(), 1);
        vec![q.get(SymbolId::of::<Btc>()).unwrap().0.unwrap()]
    }
}
#[derive(ComponentSubscription, dataflow_flat::UpdateHandler)]
#[subscribe(batch: CountOms)]
struct Capture {
    log: Rc<RefCell<Vec<Vec<f64>>>>,
}
impl Component for Capture {
    type Config = Rc<RefCell<Vec<Vec<f64>>>>;
    type Output = ();
    fn new(log: Self::Config, _: Self::Init<'_>) -> (Self, ()) {
        (Self { log }, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        self.log.borrow_mut().push(i.batch.orders.clone());
    }
}
#[test]
fn unavailable_inputs_do_not_compute_or_replay_orders_and_recover() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut p =
        Pipeline::<Tick, (Capture,)>::build(Configs::new().with::<Capture>(log.clone())).unwrap();
    for price in [-1.0, 10.0, -1.0, 20.0] {
        p.push(Tick { price });
    }
    assert_eq!(*log.borrow(), vec![vec![], vec![1.0], vec![], vec![2.0]]);
}

#[oms(exchange=TestExchange)]
#[quotes(first: CountQuote, second: CountQuote)]
struct Duplicate;
impl Oms for Duplicate {
    type Config = ();
    fn new(_: ()) -> Self {
        Self
    }
    fn compute(&mut self, _: Self::Inputs<'_>, _: Quotes<'_, TestExchange>) -> Vec<f64> {
        vec![]
    }
}
#[test]
#[should_panic(expected = "duplicate quote engine for exchange/symbol")]
fn duplicate_symbols_rejected_at_build() {
    let _ = Pipeline::<Tick, (Duplicate,)>::build(Configs::new());
}
