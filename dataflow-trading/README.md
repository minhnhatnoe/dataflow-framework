# Trading interfaces (initial implementation)

`dataflow-trading` implements stateful Books, signals, quote engines and OMSs on
`dataflow-flat`. Add both crates as dependencies. User crates need no feature
attributes; use the workspace's pinned nightly.

Run the complete synthetic two-symbol example:

```sh
cargo run -p dataflow-trading --example strategy
cargo test -p dataflow-trading
python3 scripts/check_trading_errors.py
```

## Component contracts

| Kind | Declaration | Implement | Published value |
|---|---|---|---|
| Book | `#[book]` | `Book` | Arbitrary `State`, updated through `&mut State` |
| Signal | `#[signal]` | `Signal` | Named public `f64` fields, `#[derive(SignalOutputs)]` |
| Quote engine | `#[quote_engine(exchange=E, symbol=S)]` | `QuoteEngine` | `(Option<f64>, Option<f64>)` |
| OMS | `#[oms(exchange=E)]`, `#[quotes(name: Engine, ...)]` | `Oms` | `Vec<E::Order>` |

Each implementation owns its configuration and private fields. Configuration
implements `Default`, following `Configs` in the underlying framework. Use
`Configs::with::<ConcreteType>(config)` to override it. A concrete type is one
instance; generic parameters express exchange, symbol or other axes. Constructors
run once; subsequent calls receive `&mut self` and generated `Self::Inputs<'_>`.

Framework owners implement `Exchange` (including its order command type),
`Symbol`, Books, event sources and routing sinks. Strategy authors implement
signals, quote engines and OMSs. The example's `Route` is the framework boundary,
not another strategy component that every strategy author should write.

## Named inputs

```rust,ignore
#[inputs(
    depth: Depth<E, S>,
    account: Positions<E>,
    features: Fair<E, S>,
    fair_price: Fair<E, S> => price,
)]
```

Inputs are immutable references to the dependency's state/output. `=> price`
selects a named float field using ordinary Rust field access; no string lookup
occurs. Whole-output and selected-field subscriptions share the source instance.
The underlying `#[subscribe]` syntax is unchanged.

Books can read Books; signals and quote engines can read Books and signals;
OMS inputs can read Books. Macros enforce these categories through trait bounds.
Books expose arbitrary typed state, with no `Clone` or `Default` requirement.
Their `new` returns `(Self, State)` and `update` mutates that state in place.
Only Books may declare `#[source(event: RootType)]`: this first adapter accepts
one current root event, rather than a retained source output. It never replays
a previous feed event. A Book may also have Book inputs.

## Readiness and execution

A Book's `ready(&state)` is explicit, defaulting to true. Empty state is not
implicitly unavailable. Signals and quote engines have no value until their
first computation. They compute on an upstream event only when all their inputs
are ready; loss of readiness propagates, and later recovery resumes computation.
Constructors do not invoke strategy computations. A ready Book's initial value
can be read on another dependency's first event.

`Published<T>` carries availability and a monotonically increasing revision.
An equal numeric result is still an update; no float-equality policy is imposed.
The existing graph determines execution order and runs a component at most once
per push. Shared dependencies are built once. Dependency discovery remains
transitive from the final routing sinks.

The storage adapter uses `Component::take_output` / `update_recycled`: it takes
the previous output before gathering inputs and returns it after mutation.
Acyclic dependencies make the temporary placeholder unobservable during that
update. Existing components default to the original replacement behavior.
Collection allocations inside a Book survive pushes; large inline state can
still move, so box large inline arrays. If an update panics, discard the pipeline;
the adapter does not promise recovery of the taken output.

## OMS quote view and orders

`#[quotes(btc: Quoter<E,Btc>, eth: Quoter<E,Eth>)]` supplies a statically wired
quote set. The framework creates a borrowed `Quotes<'_, E>` view with:

- `iter()`: every current quote, in declaration order;
- `get(SymbolId::of::<S>())`: the quote for a symbol (linear scan);
- `updated()`: quotes recomputed since the last successful OMS computation.

All quote engines must belong to the OMS exchange (compile-time check). Duplicate
symbols, including aliases, panic during pipeline construction. Identity uses
`TypeId`, not display names. An OMS waits until its Book inputs and every quote
are ready. Therefore an update accumulated while the OMS was unavailable is
reported on recovery. Its output batch is cleared on every invocation, including
unavailable ones, to prevent stale orders from being replayed. A routing sink
must consume that batch only when its OMS fires, not on unrelated triggers.
Order types belong to exchange adapters (`Exchange::Order`), not individual OMSs.

## Deliberate first-version limits

- The `strategy!` catalog shorthand is not implemented. Quote sets are explicit
  on OMS declarations; this is the main assembly API to evolve next.
- Sources currently subscribe to the whole root event. Selected feed sources,
  multiple independently firing event inputs and listener request integration
  need a dedicated source adapter. All root-subscribing Books run on each push.
- The underlying exclusive-join rule still applies. This layer does not expand
  the class of graphs accepted by the engine.
- Uniqueness is checked within each OMS at construction, not globally across
  multiple OMS sinks. Use one OMS per exchange in the initial assembly.
- `updated()` is since the last successful OMS call, which can span several
  pushes during unavailability. It is not an event timestamp.
- An OMS returns a new order vector; allocation-free order buffers, large-graph
  performance and strategy catalog generation are not claimed here.
- Dependency crate names must remain `dataflow_flat` and `dataflow_trading` in
  consumer manifests, as in the existing macros.

The integration tests cover generic instances, shared Books, non-Clone Book
allocation reuse, readiness loss/recovery, order batches and duplicate symbols.
Compile-fail checks cover the schema, input categories, identities and wiring.
