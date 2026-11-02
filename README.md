# dataflow: compile-time-wired, event-driven component graphs in Rust

A framework for building programs out of many small components (designed for hundreds, tested into the thousands) that form a directed acyclic graph and react to events. You feed one *root* value into a `Pipeline` with `push`. Each component is constructed from an owned config, subscribes to the outputs of other components (or to the part of an output that matches a pattern), and has one `update` method that produces a new output. Everything upstream of the *sinks* you name is discovered by the compiler, built exactly once per concrete type, and updated only when something it reads changed, in a documented order. At run time there is no dynamic dispatch, no lookup and no `unsafe`: a `push` compiles to straight-line code with a jump table where the graph branches, comparable to a hand-written nested `match` (about 1% to 8% slower on the benchmark graph, see [Performance](#performance-and-compile-time)).

**Status:** working prototype; the focus is the design rather than polish. Everything here was built and checked on the pinned nightly (see [Toolchain](#toolchain)). Code blocks in this file were compiled as a scratch crate against this tree; the output shown is what they printed.

## Contents

1. [Requirements that shaped the design](#requirements-that-shaped-the-design)
2. [Quick start](#quick-start)
3. [Workspace layout](#workspace-layout)
4. [The model](#the-model)
5. [Subscriptions are patterns](#subscriptions-are-patterns)
6. [Demux, markers and generic selectors](#demux-markers-and-generic-selectors)
7. [Stored selector values](#stored-selector-values)
8. [Handlers](#handlers)
9. [Listeners](#listeners)
10. [Handle components](#handle-components)
11. [How it works, start to finish](#how-it-works-start-to-finish)
12. [Guarantees and where each error shows up](#guarantees-and-where-each-error-shows-up)
13. [Performance and compile time](#performance-and-compile-time)
14. [Design history and rejected alternatives](#design-history-and-rejected-alternatives)
15. [Known limitations and rough edges](#known-limitations-and-rough-edges)
16. [Inspecting compiler output](#inspecting-compiler-output)
17. [Toolchain](#toolchain)

## Requirements that shaped the design

These came from the person who asked for the framework, roughly in the order they arrived:

| # | Requirement |
|---|---|
| R1 | Components depend on other components. If two components depend on the same component, only one instance exists. Each concrete type, including generic instantiations such as `Book<Binance, Btc>` vs `Book<Binance, Eth>`, is a singleton. |
| R2 | Dependencies are resolved at compile time. |
| R3 | With hundreds of components, the user names only the minimal set (the sinks). Nothing else is listed anywhere. |
| R4 | Each component takes an owned config in its constructor, returns its first output from it, and has a single `update` that takes what it subscribes to and produces its next output. |
| R5 | Components run in topological order, and the order is a documented guarantee, not an implementation detail. |
| R6 | No dynamic dispatch. |
| R7 | Nightly features are acceptable, but must not leak into user crates: user code needs no feature gate. |
| R8 | Declaration syntax: the user names subscriptions, and `update` receives a struct with named fields (`input.sub1`), not a tuple. |
| R9 | Event-driven: one root, fed by `push`. A step runs only the components whose inputs changed. Outputs persist between steps. |
| R10 | `#![forbid(unsafe_code)]` in the framework. |
| R11 | Demuxing lives on the subscription (a pattern on the parent's output), not in extra graph nodes. A component joining branches that can fire together is a compile error. |
| R12 | Run time equal to a hand-written program, within a few percent. |
| R13 | Scale: one NIC, a few exchanges, thousands of markets, about ten components per market. Release build of the typical app (823 components) in well under a minute; much larger graphs must at least build. |

## Quick start

```rust
use dataflow_flat::{Component, ComponentSubscription, UpdateHandler, Configs, Pipeline, Root};

/// The root: the value passed to `push`. It exists only during the push.
#[derive(Root)]
pub struct Tick(pub u64);

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(tick: Tick)]
pub struct Total {
    sum: u64,
}
impl Component for Total {
    type Config = u64; // the starting sum
    type Output = u64;
    fn new(start: u64, _: Self::Init<'_>) -> (Self, u64) {
        (Total { sum: start }, start) // the component and its first output
    }
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        self.sum += i.tick.0;
        self.sum
    }
}

/// A sink: an ordinary component nobody subscribes to.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(total: Total)]
pub struct Printer;
impl Component for Printer {
    type Config = ();
    type Output = ();
    fn new((): (), i: Self::Init<'_>) -> (Self, ()) {
        println!("initial total {}", i.total);
        (Printer, ())
    }
    fn update(&mut self, i: Self::Input<'_>) {
        println!("total {}", i.total);
    }
}

fn main() {
    let mut p = Pipeline::<Tick, (Printer,)>::build(Configs::new().with::<Total>(10)).unwrap();
    p.push(Tick(1));
    p.push(Tick(5));
    println!("{:?}", p.order());
}
```

```text
initial total 10
total 11
total 16
["docs_check::quick::Total", "docs_check::quick::Printer"]
```

(The last line is the construction order, which is also the order of updates in a step; the crate name comes from the scratch crate.) Run the bigger examples with `cargo run -p dataflow-flat --example basic` and `--example exchanges`; the second is the one to read: a NIC, two exchanges, three markets, listeners, an exclusive join and a handler sink.

## Workspace layout

| Path | What |
|---|---|
| `dataflow-flat/` | The library: plan, storage, construction and push (`src/`), tests (`tests/`), examples |
| `dataflow-macros/` | `#[derive(ComponentSubscription)]` with `#[subscribe(..)]`, `#[derive(UpdateHandler)]`, `#[derive(Root)]`, `#[derive(Demux)]`, the `catalog!` macro and the callback macros that generate the per-shape impls |
| `bench/` | Generators for benchmark apps (`gen_app.py` is the main one; see [Performance](#performance-and-compile-time)) |
| `scripts/` | `check_errors.sh` (one tiny crate per class of user mistake), `measure.py`, `bench.sh`, `gen_selbench.py` |
| `handoff/` | Design notes and session logs (`event-spec.md`, `README.md`, `session-2.md`), small profiling tools |
| `.cargo/config.toml` | Build flags for this tree (see [Build flags](#build-flags-and-files)) |

## The model

**Nodes.** Everything you can subscribe to is a node. There are two kinds you write:

* The **root**, `#[derive(Root)]` on a struct or enum, is the value of one `push`. It has no state and no subscriptions, fires on every push, and exists only during that push. A pipeline has exactly one root type `R`.
* **Components**: `#[derive(ComponentSubscription, UpdateHandler)]` with `#[subscribe(..)]`, plus `impl Component`. A component's latest output is *stored* in the pipeline and persists between pushes. (`UpdateHandler` is the derive for a component with no handlers; see [Handlers](#handlers).)

A **sink** is a component whose output is usually `()`. `Pipeline::<R, (Sink1, Sink2)>::build(configs)` takes a tuple of 1 to 15 sinks; the graph is everything upstream of them. There is no application component.

**The component trait.**

```rust
trait Component {
    type Config: Default + 'static;   // owned; components without an entry in `Configs` get Default
    type Output: 'static;             // subscribers receive &Output
    fn new(config: Self::Config, init: Self::Init<'_>) -> (Self, Self::Output);
    fn update(&mut self, input: Self::Input<'_>) -> Self::Output;
}
```

A component whose subscribers should read a view of its state instead of a stored value is a [handle component](#handle-components). The derive generates `Input<'a>` (one `&'a` field per named subscription, in listing order) and `Init<'a>` (the same names, holding what each subscription's output was when it was constructed). `new` runs once, in execution order, so every subscription already has a first output. `Pipeline::build` returns `Err(BuildError)` if a config was supplied for a component outside the graph.

**One step, `push(root)`:**

1. **OR rule.** A component runs iff at least one of its edges *fired*. The root fires on every push. A component fires iff it ran. An edge with a pattern (below) fires iff its parent fired *and* the pattern matched. Anything that did not run keeps its stored output, and subscribers read that.
2. **Once, in post-order.** A component runs at most once per step, in depth-first post-order following the subscription listing order. For `C` subscribing to `[D1, .., Dn]`, the runs during `C`'s walk come in `n` blocks followed by `C` itself. Block `i` is `Di` plus everything upstream of `Di` that has not run yet in this step, in this same order. A later-listed dependency that is upstream of an earlier one therefore runs inside the earlier block. This is documented on `Component` and `Pipeline`, and `tests/order.rs` enforces it.
3. **Exclusive joins.** A component can read branches that never fire in the same step. After the plan has absorbed redundant terms (a term implied by another is dropped), the remaining terms of its condition must be pairwise *exclusive*: they choose different alternatives of the same enum at the same place. Otherwise the build fails with "`X` joins branches that can fire in the same step". In the exchanges example `BestBid<S>` reads `Book<Binance, S>` and `Book<Coinbase, S>`; one packet is from one exchange, so the join is accepted, and in a Binance step the Coinbase book supplies its stored value.
4. **Reachability.** Every component must be reachable from the root, and the root must be `R`; a second root type in the graph is an error.

`push` returns nothing. Outputs are observed by sinks (side effects in `update`); there is no accessor for stored outputs. This is deliberate: an accessor would force every output to live somewhere inspectable and would reintroduce a place for run-time bookkeeping.

The framework never invents an `Option` for an input. A component that wants "no value yet" sets `Output = Option<T>` itself (the exchange listeners do, with `None` as the first output).

## Subscriptions are patterns

The `#[subscribe(..)]` list is a list of entries. A subscription is a Rust-looking pattern on the parent's output, written the way a `match` arm would be, followed by `:` and the parent node. The pattern says which part of the parent's output the component reads, and the edge fires only when the pattern matches.

```rust
use dataflow_flat::{Component, ComponentSubscription, UpdateHandler, Configs, Demux, Pipeline, Root};

#[derive(Root)]
pub struct Poll;

#[derive(Clone, Copy, Debug, Default)]
pub struct BinanceMsg { pub bid: i64, pub ask: i64 }
#[derive(Clone, Copy, Debug, Default)]
pub struct CoinbaseMsg { pub mid: i64 }

/// An ordinary enum. `#[derive(Demux)]` makes one selector per variant.
#[derive(Clone, Copy, Debug, Demux)]
pub enum Packet {
    Binance(BinanceMsg),
    Coinbase(CoinbaseMsg),
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(_poll: Poll)]
pub struct Nic { n: u64 }
impl Component for Nic {
    type Config = ();
    type Output = Option<Packet>;   // None: the poll found nothing
    fn new((): (), _: Self::Init<'_>) -> (Self, Option<Packet>) { (Nic { n: 0 }, None) }
    fn update(&mut self, _: Self::Input<'_>) -> Option<Packet> {
        self.n += 1;
        match self.n % 3 {
            0 => None,
            1 => Some(Packet::Binance(BinanceMsg { bid: 100 + self.n as i64, ask: 102 })),
            _ => Some(Packet::Coinbase(CoinbaseMsg { mid: 101 })),
        }
    }
}

/// Runs only for Binance packets. `m` is `&BinanceMsg`.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(Packet::Binance(m)): Nic)]
pub struct BinanceBook;
impl Component for BinanceBook {
    type Config = ();
    type Output = i64;
    fn new((): (), _: Self::Init<'_>) -> (Self, i64) { (BinanceBook, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> i64 {
        println!("BinanceBook runs");
        i.m.bid
    }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(Packet::Coinbase(m)): Nic)]
pub struct CoinbaseBook;
impl Component for CoinbaseBook {
    type Config = ();
    type Output = i64;
    fn new((): (), _: Self::Init<'_>) -> (Self, i64) { (CoinbaseBook, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> i64 {
        println!("CoinbaseBook runs");
        i.m.mid
    }
}

/// An exclusive join: a packet is from one exchange.
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(b: BinanceBook, c: CoinbaseBook)]
pub struct Best;
impl Component for Best {
    type Config = ();
    type Output = i64;
    fn new((): (), _: Self::Init<'_>) -> (Self, i64) { (Best, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> i64 {
        println!("Best: binance {} coinbase {}", i.b, i.c);
        *i.b.max(i.c)
    }
}
// Pipeline::<Poll, (Best,)>::build(Configs::new()), pushed four times:
```

```text
-- push
BinanceBook runs
Best: binance 101 coinbase 0
-- push
CoinbaseBook runs
Best: binance 101 coinbase 101
-- push
-- push
BinanceBook runs
Best: binance 104 coinbase 101
```

The third push found nothing (`None`): `Nic` ran and fired, but neither pattern matched, so nothing downstream ran.

**What a subscription entry can be.**

| Entry | Meaning |
|---|---|
| `limits: Limits` | A plain subscription: the whole output, always readable. The field is `&Output`. The name starts with a lowercase letter or `_`. |
| `Some(Packet::Binance(m)): Nic` | A chain of selectors, outermost first: `Some`, then the `Binance` variant. `m` is a reference to the innermost part. |
| `Btc(t): Ws` | A single selector `Btc` in scope (a marker type, see [Demux](#demux-markers-and-generic-selectors)). |
| `S(t): Src` | `S` is a generic parameter of the component that implements `Select<..>`. |
| `Ok(v): Parse`, `Err(e): Parse` | `Result` selectors. `Some(x)` and `None` are the `Option` selectors. These four names are the library's, not the prelude's variants. |
| `Disconnected: Ws` | A *unit selector*: no field to bind. The edge fires on the variant; it has no field in `Input`. Needs `Disconnected` in scope as a type (`use event::Disconnected;`), or write the path `Event::Disconnected: Ws`. |
| `Quoter<Btc>` (no colon) | An *unnamed* entry: not a field of `Input`, only its `UpdateHandler` impl sees it (see [Handlers](#handlers)). |
| `Btc(_): Ws` | A selected unnamed entry: `_` as the leaf. |
| `for S in Catalog { Quoter<S> }` | One unnamed entry per element of a catalog. |

**The uppercase-is-selector rule.** Before the `:`, an identifier that starts with a lowercase letter or `_` is the *name* of a plain input. An identifier that starts with an uppercase letter is a selector. That is why `limits: Limits` and `Disconnected: Ws` parse differently, and why a named input cannot start with an uppercase letter. Inside a pattern, the last, lowercase identifier is the binding that becomes a field of the input. A path whose second-to-last segment starts with an uppercase letter, `Packet::Binance`, means the marker the `Demux` derive generated for that variant, `packet::Binance` (the enum name in snake case); `Option::Some` and `Result::Ok` mean the library's; any other path (`markets::Btc`) is a type as written.

**What is rejected, with the message the derive prints** (copied from `scripts/check_errors.sh` runs):

| You wrote | Error |
|---|---|
| `x: Src => Sel1, Sel2` | ``error: `=>` is gone: write the selectors as a pattern, `Sel1(Sel2(x)): Src` instead of `x: Src => Sel1, Sel2` (a selector with nothing to bind is `Sel: Src`)`` |
| a struct pattern | ``subscription pattern: struct patterns are not supported; only tuple-variant patterns `Selector(name)` `` |
| `V(a, b)` | `subscription pattern: multi-field variants are not supported (a selector selects exactly one field)` |
| `V(1)` or `V(-1)` | ``subscription pattern: literal patterns are not supported; use a hand-written guard selector `S(x): Source` `` |
| `V(x) if ..` | ``subscription pattern: guards are not supported; use a hand-written guard selector `S(x): Source` `` |
| `A(x) \| B(x)` | `subscription pattern: or-patterns are not supported; use a hand-written selector` |
| `&V(x)` | `subscription pattern: references are not supported (a pattern matches the value, which is borrowed already)` |
| `V(1..=2)` | `subscription pattern: range patterns are not supported` |
| `_` alone | ``subscription pattern: a bare `_` is not a pattern; a handled entry is a type (`Average`) or `Selector(_): Source` `` |
| `Disconnected: Src` without the selector in scope | `error[E0425]: cannot find type `Disconnected` in this scope` |

Patterns are enum-only on purpose: they mirror a `match` arm on an enum variant with at most one field, and anything else (a literal, a range, a guard, an or-pattern) is a hand-written selector, below. This keeps the plan's idea of "alternative at a place" simple: a selector has an integer `KEY`, two selectors with different keys at the same place are mutually exclusive, and a guard (`KEY = GUARD`) is exclusive with nothing.

A unit selector on a data variant, `Data: E` where `Data(u64)`, compiles and ignores the payload. Several `#[subscribe(..)]` attributes on one struct are allowed; their entries are concatenated in order.

### Hand-written selectors

```rust
trait Select<In>: 'static {
    type Out: 'static;                 // the matched part
    const KEY: u32;                    // the alternative at this place; GUARD (u32::MAX) = no exclusivity
    const REQUEST: bool = false;       // does a listener get told (see Listeners)?
    fn select(x: &In) -> Option<&Self::Out>;
    fn key_of(x: &In) -> u32;          // the KEY of the alternative x is, GUARD if none
}
```

`key_of` lets the push evaluate one function and branch on a jump table instead of testing each selector in turn. All keyed selectors applied at one place must belong to one family (give the same `key_of`). A guard returns `GUARD` from `key_of`. A hand-written selector that is read in steps where it does not match also implements `Initial<In>` (below). The source of the subscription decides which trait it needs: a stored output takes `Select`; a `#[handle]` component takes `SelectView<View>` (`fn select(x: View) -> Option<Out>`, by value, see [Selectors on a handle component](#selectors-on-a-handle-component)).

## Demux, markers and generic selectors

`#[derive(Demux)]` on an enum generates a module named after the enum in snake case with one zero-sized selector per variant, and implements `Select<Enum>` for each:

* variants are unit variants (the part is `()`) or tuple variants with exactly one field (the part is that field);
* the `KEY` is the variant index, and one shared `key_of` function serves all variants, so the generated code is linear in the number of variants;
* the enum cannot be generic.

Reading `Packet::Binance(m)` is `packet::Binance(m)`. For a market catalog you usually want *one* marker per market shared by many enums (each exchange's frame has a `Btc` variant). `#[demux(markers = path)]` makes the derive implement `Select` for the existing types `path::Variant` instead of generating new ones, and re-exports them in the enum's module so `Frame::Btc` still names the same type:

```rust
use dataflow_flat::{Component, ComponentSubscription, UpdateHandler, Configs, Demux, Pipeline, Root, Select};
use std::marker::PhantomData;

/// One marker per market, shared by every exchange's frame enum.
pub mod markets {
    pub struct Btc;
    pub struct Eth;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Tob { pub bid: i64 }

#[derive(Clone, Copy, Debug, Demux)]
#[demux(markers = markets)]
pub enum BinanceFrame { Btc(Tob), Eth(Tob) }
#[derive(Clone, Copy, Debug, Demux)]
#[demux(markers = markets)]
pub enum CoinbaseFrame { Btc(Tob), Eth(Tob) }

// `BinanceWs` (not shown) has Output = Option<BinanceFrame>.

/// `S` is any marker that selects a `Tob` out of a `BinanceFrame`: Btc, Eth, ...
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(S(t)): BinanceWs)]
pub struct Book<S: Select<BinanceFrame, Out = Tob>>(PhantomData<S>);
// impl Component for Book<S> { .. update returns i.t.bid .. }

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(btc: Book<markets::Btc>, eth: Book<markets::Eth>)]
pub struct Sink;
```

```text
Btc runs
btc 100 eth 0
Eth runs
btc 100 eth 101
Btc runs
btc 102 eth 101
```

`Book<Btc>` and `Book<Eth>` are two components; each runs only when its variant arrives, and `Sink` runs on either and reads the other's stored output. Because `Book<S>` is generic, the same body serves every market. The recommended shape for a large family is a thin generic wrapper over a non-generic core compiled once (`dataflow-flat/examples/exchanges/book.rs`).

The marker may select from several enums only if each enum implements `Select` for it; the bound `Select<BinanceFrame, Out = Tob>` is what the component needs. The exchanges example combines two with one trait (`Sym`).

## Stored selector values

A component can subscribe to several alternatives of one source and run for any of them:

```rust
#[derive(Root, Demux, Clone, Debug)]
pub enum Wire { Btc(f64), Eth(f64), Disconnected }

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Wire::Btc(a): Wire, Wire::Eth(b): Wire)]
pub struct Spread;
// update: a - b, printing both

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Wire::Disconnected: Wire)]
pub struct Alarm;
```

```text
Spread::new sees btc 0 eth 0
-- push Btc(10.0)
Spread: btc 10 eth 0
Out: spread 10 alarm 0
-- push Eth(3.0)
Spread: btc 10 eth 3
Out: spread 7 alarm 0
-- push Btc(11.0)
Spread: btc 11 eth 3
Out: spread 8 alarm 0
-- push Disconnected
Alarm
Out: spread 8 alarm 1
```

(`Out` subscribes to `Spread` and `Alarm`.) `Spread` runs on a `Btc` or an `Eth` push; on a `Btc` push `b` is not matched, so it reads the **last value** that `Eth(b)` matched, and before the first match, the selector's **initial value**. `Alarm` runs only on `Disconnected`, and `Spread` does not run for it, so `Out` reads `Spread`'s stored output.

Rules:

* **Last value.** For each (source, selector chain) that some subscriber reads in steps where the chain does not match, the plan keeps one slot, shared by all such subscribers, written right after the source runs (at the start of the push for the root) when the chain matches. An input that is always matched when its subscriber runs has no slot and is read straight from the source.
* **Initial value, and `new`.** Every selected input has an initial value that `new` is given (`init.a`, which dereferences to the part): `Default::default()` of the payload for `#[derive(Demux)]` selectors, `Some`, `Ok` and `Err`, and `Initial::initial()` for a hand-written selector (`impl Initial<In> for Sel`). `None` and unit selectors have the part `()`. The derive fills the field in the user's crate, where the chain and payload are concrete types: it uses the chain's `Initial` if there is one, else the field is a `SelInit` that panics when dereferenced, naming the component, the input and the payload (`SelInit::get` gives `None` instead). This cannot be a compile error, since `new` reads the field at run time. Limit: a component that is generic over its selector or its payload sees only type parameters there, so no `Initial` is found, and `new` panics if it reads such an input unless the edge has a slot (the slot's value wins).
* **`Default` and `Clone`.** `Clone` is required only for an input that is stored (the value is cloned out of the source's output); a payload that is read only when matched needs neither. A missing `Clone` is: ``error[E0277]: `NoClone` is read through selectors in steps where they do not match, so it is stored, and must implement `Clone` ``. A missing initial value: ``error[E0277]: the trait bound `NoDef: Default` is not satisfied``.
* Unnamed entries are never read in `update` and never stored. (A named entry with an active handler is read in `update` as always, and also given to `on_update`.)

## Handlers

Every entry of a component's `#[subscribe(..)]`, named or not, has an `UpdateHandler<E>` impl, where `E` is the entry's key (below). The impl says whether the component wants a call when that edge fires: `on_update(&mut self, input, v)`, once per push in which the edge fired, in the execution order of the parents, **before `update`**, in the same step. `input` is the same `Self::Input<'_>` that `update` is about to get (every named entry current, including the ones that did not fire in this push or whose selector did not match); `v` is the value of the edge. A component with no handlers at all, the common case, derives one blanket impl:

```rust
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(limits: Limits, feed: Feed)]
pub struct Quoter { .. }
```

`#[derive(UpdateHandler)]` is `impl<E: HandledEdge> UpdateHandler<E> for Quoter { const ACTIVE: bool = false; }`: no entry calls anything, and the component is an ordinary one in the push program (nothing is generated for it). A component with handlers writes them by hand, one impl per entry:

```rust
use dataflow_flat::{catalog, UpdateHandler};

#[derive(ComponentSubscription)]               // no `UpdateHandler` derive
#[subscribe(limits: Limits, Quoter<markets::Btc>, Quoter<markets::Eth>)]
pub struct Oms {
    pending: Vec<(&'static str, i64)>,   // recycled between pushes
}
impl Component for Oms {
    type Config = ();
    type Output = usize;                 // how many orders this push produced
    fn new((): (), _: Self::Init<'_>) -> (Self, usize) {
        (Oms { pending: Vec::with_capacity(16) }, 0)
    }
    fn update(&mut self, i: Self::Input<'_>) -> usize {
        let n = self.pending.len();
        for (name, px) in self.pending.drain(..) {
            println!("  order {name} {px} (limit {})", i.limits);
        }
        n
    }
}
impl UpdateHandler<Limits> for Oms {}    // a named entry needs an impl too; the empty one does nothing
impl<S: Symbol + Select<Tick, Out = Quote>> UpdateHandler<Quoter<S>> for Oms {
    fn on_update(&mut self, input: Self::Input<'_>, px: &i64) {
        if *px < input.limits.max_px {
            self.pending.push((S::NAME, *px));
        }
    }
}
```

* **The key `E` of an entry.** A plain entry, named (`limits: Limits`) or not (`Quoter<Btc>`), is keyed by its node type. A selected entry, named (`Some(Sol(sol)): Ws`) or not (`Btc(_): Ws`), is keyed by a marker the derive generates in a module named after the struct: for `struct Tape` it is `tape_edges::Btc`. The marker's name is the binding's name in camel case (`sol` gives `tape_edges::Sol`; `_` takes the innermost selector's name, and so does a unit selector; for a generic selector `Tag::<Btc>` it is the type argument's name). Two unit selectors that would get one name are told apart by their position (`Name2`).
* **The value `v`.** It follows the source. A by-value parent gives `&Output` (`&i64` above). A handle parent (`#[handle]`, below) gives exactly its `View<'a>`: `u64` for an owned view, `&'a u64` for a borrowed one. A selected entry gives the selected part, by reference, and `E`'s `Arg<'a>` (`HandledEdge::Arg`) is that type.
* **`ACTIVE`.** `UpdateHandler` has `const ACTIVE: bool = true` and `on_update` has an empty default body. `impl UpdateHandler<E> for C {}` therefore means a call that does nothing; `const ACTIVE: bool = false` in the impl says the plan need not make it. The plan reads `ACTIVE` once per edge: a component with no active edge is an ordinary one in the push program. Unnamed entries are not fields of `Input`; a named entry stays one whether it is active or not, so `input.limits` and `v` can both be read in a call for it.
* **A missing impl** is ``error[E0277]: `Oms` subscribes to `Limits` without an `UpdateHandler<Limits>` impl``, pointing at the struct (`cargo check` already shows it), with a note that says to derive `UpdateHandler` or to write the impl. **`#[derive(UpdateHandler)]` with a hand-written impl** is Rust's own ``error[E0119]: conflicting implementations of trait `UpdateHandler<Limits>` for type `Oms` ``: the blanket impl the derive wrote overlaps every hand-written one. Remove the derive from a component that has handlers (the derive is for components that have none); there is no way to combine them, a hand-written impl for every entry with `const ACTIVE: bool = false` is the same thing.
* **An unnamed entry whose impl is `ACTIVE = false`** is a trigger only: the component runs when the edge fires, and nothing reads the value.

When a handler runs (the OR rule over *all* its entries, named and unnamed), each active edge that fired in this push gets `on_update` in the execution order of the parents, then `update` runs once. Nothing is called at build: `new` returns the initial output. Edges that did not fire are never called; an unnamed edge is not read at all, so a handler may list branches that are not implied by its condition. The exclusive-join rule still applies to its entries. Inside each branch of the push the set of fired active edges is known statically, so the code is `on_update` for exactly those and then `update`: no flags and no loop over the entries.

* **Selected entries.** `Btc(_): Ws` (the leaf is `_`) or `Some(Sol(sol)): Ws` (named). `on_update` gets the selected part:

    ```rust
    #[derive(ComponentSubscription)]
    #[subscribe(markets::Btc(_): Tick, markets::Sol(_): Tick)]
    pub struct Tape { seen: Vec<i64> }
    // impl UpdateHandler<tape_edges::Btc> for Tape { fn on_update(&mut self, _: Self::Input<'_>, q: &Quote) { .. } }
    // impl UpdateHandler<tape_edges::Sol> for Tape { .. }
    ```

    Two unnamed selected entries ending in the same selector name are an error (``two handled entries end in the selector `K0` ``).
* **`for` entries over a catalog.** Writing a hundred `Quoter<..>` entries by hand is not an option, so `for S in Catalog { Node<S> }` expands to one unnamed entry per element, in catalog order (which is listing order):

    ```rust
    pub type AllSymbols = catalog![markets::Btc, markets::Eth, markets::Sol];

    #[derive(ComponentSubscription)]
    #[subscribe(limits: Limits, for S: Symbol + Select<Tick, Out = Quote> in AllSymbols { Quoter<S> })]
    pub struct Oms2 { pending: Vec<(&'static str, i64)> }
    // the same `impl<S: ..> UpdateHandler<Quoter<S>> for Oms2` as above, and `impl UpdateHandler<Limits> for Oms2 {}`
    ```

    A catalog is a nested tuple built by `catalog![A, B, C]`, or one of the combinators, which take catalogs: `Concat<A, B>` (the elements of `A`, then `B`), `Product<A, B>` (all pairs, row by row), `Zip<A, B>` (position-wise pairs; `A` and `B` must have the same shape). Pairs are `M<X, S>`, destructured in the `for`: `for (X: Tag, S: Symbol) in Product<Venues, Coins> { Book<X, S> }`, and `((X, S), Y)` for nesting. The derive cannot see what a catalog contains, so it generates a mapper from the body; there is no filtering. An enum with `#[derive(Demux)]` is a catalog of its markers (`mkt::All`, below). The body must be one node type (`a `for` body is one node type`), and a pattern with more than a pair is rejected (``a `for` pattern destructures a pair: `(X, S)` ``).
* **Handle components** take part like any other: a handle parent's `View<'a>` is the value `v`, and a `#[handle]` component can have handlers and `for` entries of its own. The component is then updated through the split of the states (`SplitAt`) and `on_update` gets `input` read from the same context as `update` does.
* **Cost.** The edge's flag is tested only where the plan cannot know statically that the edge fired. Inside a `Switch` arm the fired edge is known, so there is no test; `on_update`'s `input` is gathered at the call, and an empty body leaves nothing behind after inlining. An `impl UpdateHandler<E> for C {}` (default `ACTIVE = true`) makes the plan treat the component as a handler (it is `PG_HRUN` instead of `PG_RUN`, with one item per edge that can fire), even though each call does nothing; set `ACTIVE = false` on entries that never need a call.

### Handler buffer guidance

The buffer a handler collects into lives on `self`, is allocated once, and is emptied in `update`. `on_update` gets `input` as a separate argument, so reading `input.limits` while pushing into `self.pending` borrows nothing of `self`:

* allocate it in `new`: `Vec::with_capacity(n)`;
* empty it in `update` with `drain(..)` (to consume the items) or `clear()`;
* **do not** use `std::mem::take(&mut self.pending)` on a `Vec`: it leaves an empty `Vec` with no capacity behind, so the next push allocates again;
* **do not** return a `Vec` as the component's `Output`: the output is stored and replaced on every run, so each push allocates and frees a `Vec`. Return a count, a small `Copy` summary, or keep the data on `self` and have sinks read it by side effect.

Some tests do use `mem::take` and a `Vec` output for brevity; they are not performance code. A buffer a handler fills from `on_update` can also be handed to a downstream component without allocation by recycling: `update` swaps in an emptied buffer, e.g. `std::mem::replace(&mut self.pending, std::mem::take(&mut self.spare))`, where `spare` is the `Vec` the downstream component gave back (it keeps its capacity).

### How the `for` form feels

Checked by building `Oms2` above. It works and the expansion is as cheap as the written-out form (`bench/gen_handler.py N OUT for` generates the same plan and code). Three rough edges were found; this is where each stands.

* **The element bounds are written twice, and that stays.** The `for` header and the `UpdateHandler` impl both carry `S: Symbol + Select<Tick, Out = Quote>`. The header cannot be derived from `Quoter<S>`: Rust checks that `Quoter<S>` is well formed wherever it is written, and a macro cannot read the bounds of another type, so the generated `impl<S> Map<S>` has to state them (the impl is the same: Rust never infers bounds). What was fixed is the error when they are missing. It used to point at the struct with a help that suggested `pub struct Oms2 + handlers::Symbol`; it is now rustc's own error spanned at the `for` entry, with a help that edits the entry:

    ```text
    error[E0277]: the trait bound `S: Sym` is not satisfied
      |  #[subscribe(for S in dataflow_flat::catalog![A] { Q<S> })]
      |                                                    ^^^^ the trait `Sym` is not implemented for `S`
    note: required by a bound in `Q`
    help: consider further restricting type parameter `S` with trait `Sym`
      |  #[subscribe(for S + Sym in dataflow_flat::catalog![A] { Q<S> })]
    ```

    (rustc writes `S + Sym` where the entry wants `S: Sym`; that is rustc's rendering of "add a bound to a parameter that already has `'static`", but it is at the right place.) `check_errors.sh for_missing_bound` keeps this.
* **An enum is a catalog of its markers.** `#[derive(Demux)]` generates `<enum_snake>::All`, `catalog![Variant, ..]` in declaration order, next to the markers (with `#[demux(markers = path)]` the shared markers are listed). It is skipped if a variant is itself named `All`. So no list is written:

    ```rust
    #[derive(Demux)]
    pub enum Mkt { A(u64), B(u64), C(u64) }
    // `MktWs` has Output = Option<Mkt>.

    #[derive(ComponentSubscription, UpdateHandler)]
    #[subscribe(Some(S(u)): MktWs)]
    pub struct Leg<S: Select<Mkt, Out = u64>>(PhantomData<S>);
    // Component for Leg<S>: update returns *u * 10

    #[derive(ComponentSubscription)]
    #[subscribe(for S: Select<Mkt, Out = u64> in mkt::All { Leg<S> })]
    pub struct Fan;
    impl<S: Select<Mkt, Out = u64>> UpdateHandler<Leg<S>> for Fan { /* S::KEY is the variant index */ }
    ```

    (compiled in `tests/for_form.rs`, `enum_is_a_catalog_of_its_markers`.) A catalog of another enum's markers works the same through `markers = path`; the catalog still has to be the one whose elements the body accepts.
* **The body is one node type, and that stays.** ``for S in All { Some(S(_)): Ws }`` is rejected: ``a `for` body is one node type, ..: a named entry (`x: Ws`) or a selected one (`Some(S(_)): Ws`) is not supported in it yet``. A selected handled entry is keyed by a generated `<struct>_edges::<Name>` marker; one per element would be a generic marker `Name<S>` that `Map` must also return (with the chain, instead of the fixed `Hd<Node, ()>` of `MapDeps`), that the derive declares and the user's impl names (`UpdateHandler<oms_edges::Sel<S>>`), plus a way to give each element's entry an `Initial` for an unread input. The workaround is the one in the example above: put the selector in a component generic over the marker (`Leg<S>`), which is also what the other examples do.

Nobody outside the tests has used it yet.

## Listeners

A **listener** is a component whose subscribers read only some alternatives of its output (one feed, many markets), and which wants to know which ones, once, before it is constructed: so one websocket can carry exactly the symbols the graph uses. Mark it with `#[listener]` on the derive, implement `Listener` with a `Requests` type, and implement `Request<S>` for the selectors it accepts:

```rust
#[derive(Default)]
pub struct Streams(pub Vec<&'static str>);
impl<S: Stream> Request<S> for Streams {
    fn add(&mut self) { self.0.push(S::STREAM); }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Poll)]
#[listener]
pub struct Ws { streams: Vec<&'static str> }
impl Listener for Ws { type Requests = Streams; }
impl Component for Ws {
    // ...
    fn new((): (), init: Self::Init<'_>) -> (Self, Option<Frame>) {
        println!("subscribing to {:?}", init.requests.0); // `init.requests` is the Requests value
        (Ws { streams: init.requests.0 }, None)
    }
}
// Sink subscribes to Book<Sol> and Book<Btc>; Book<S> reads `Some(S(px)): Ws`.
```

```text
subscribing to ["solusdt@bookTicker", "btcusdt@bookTicker"]
```

`Eth` is a variant of `Frame` but nobody reads it, so it is never requested. The rules:

* **Exactly the read selectors.** `Request<S>::add` is called once per distinct *requested* selector `S` that a subscriber reads through this listener. The requested selectors are those of `#[derive(Demux)]` (`Select::REQUEST` is true); `Some`, `Ok`, `Err`, guards and hand-written selectors are not requests.
* **Order.** Components in execution order; within a component its subscriptions in listing order; within a chain, left to right. A selector already added for that listener is skipped, so the order is that of the first reader (above: `Sink` lists `Sol` first).
* **Before anything is constructed.** A first pass over the component positions runs in `Pipeline::build`; it constructs nothing. The collected value reaches the listener's `new` as `init.requests`. A listener nobody reads gets `Requests::default()`.
* **Missing impl.** A selector read downstream of a listener whose `Requests` has no `Request<S>` impl is ``error[E0277]: the listener's requests type `Reqs` cannot take the request of the selector `Eth` ``.
* The same marker through different enums is the same request. Two listeners on different enums each get their own list.

## Handle components

A by-value component stores its output and its subscribers read `&Output`. A **handle component** stores a small *handle* instead, and its subscribers read a *view* the component computes from its own state and that handle, each time a subscriber runs. The output never has to be built as a value and moved into the pipeline: `update` changes the state in place and returns the handle. Mark the struct with `#[handle]` next to the derive and implement `HandleComponent` instead of `Component`:

```rust
use dataflow_flat::{ComponentSubscription, UpdateHandler, HandleComponent};

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(tick: Tick)]
#[handle]
pub struct Ring { arr: [u64; 4], n: usize }
impl HandleComponent for Ring {
    type Config = ();
    type Handle = usize;                 // the slot of the last write; stored between pushes
    type View<'a> = &'a u64;           // what a subscriber's field is, exactly
    fn new((): (), _: Self::Init<'_>) -> (Self, usize) { (Ring { arr: [0; 4], n: 0 }, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> usize {
        self.n += 1;
        self.arr[self.n % 4] = i.tick.0;
        self.n % 4
    }
    fn view<'a>(&'a self, h: &'a usize) -> &'a u64 { &self.arr[*h] }
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(r: Ring, tick: Tick)]
pub struct Reader;                       // an ordinary component
impl Component for Reader {
    // ...
    fn update(&mut self, i: Self::Input<'_>) -> u64 {
        let last: &u64 = i.r;            // `View<'a>` of `Ring`, with no `&` added
        *last + i.tick.0
    }
}
```

* **The field is `View<'a>`.** Owned (`type View<'a> = u64;`) or a reference or a struct that borrows (`&'a Level`, `Depth<'a>`): the field of a subscriber is that type and nothing is added to it. A by-value parent is still `&Output`. `new` of a subscriber gets the view computed from the parent as it was constructed (its first handle).
* **`output` is evaluated at each use and not cached**: once per subscriber run and once per subscriber `new`. It should be cheap (an index, a reference into the state, a copy of a few words). Anything expensive belongs in `update`, whose result is the handle.
* **Persistence.** The stored thing is the handle, and the state stays where it is. A subscriber that runs in a step where the handle component did not run reads the view of the state and handle as they were (`tests/handle.rs`, `a_parent_that_did_not_run_gives_the_same_view`). The execution order, the OR rule, the one-instance-per-type rule and the lifecycle (constructed, run at most once per push, dropped once) are those of any component.
* **`Handle` needs no `Copy`**, only `'static`: the framework stores it and lends `&Handle` to `output`. `()` and indices are typical.
* **Config.** `HandleComponent::Config` is given with `Configs::new().with::<Ring>(cfg)` (`with` takes both kinds of component).
* **Generic code.** `X: Node<Output = T>` still means "a node whose subscribers read `&T`", so existing generic components compile unchanged (their field is `&T`); a handle component is not a `Node` and is rejected there with ``error[E0277]: `Ring` is not a by-value Node``. The one bound for generic *pipeline* code is still `S: Sinks<R>`.
* **Selectors work on a handle component** (next subsection). **Not supported** (each is a clear compile error, `scripts/check_errors.sh handle_*`): `#[handle]` on a `#[listener]`, and a selected value from a handle source that would have to be stored.

### Selectors on a handle component

A selector on a handle source takes the **view** and returns a view of the matched part, so a selected input stays a borrowed (or owned) view and nothing is copied or stored. `#[derive(Demux)]` on an enum **with a lifetime parameter** writes the selectors:

```rust
#[derive(Demux)]
pub enum FrameView<'a> { Book(&'a Book), Trade(&'a Trade), Seq(u64) }   // reference and owned variants

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Msg::A(v): Msg)]
#[handle]
pub struct Feed { /* a ring of frames */ }
impl HandleComponent for Feed {
    type View<'a> = FrameView<'a>;
    /* .. fn view<'a>(&'a self, h: &'a usize) -> FrameView<'a> { match &self.ring[*h] { Frame::Book(b) => FrameView::Book(b), .. } } */
}

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(FrameView::Book(b): Feed)]      // `b` is exactly the variant's field: `&Book`
pub struct Mid { .. }
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(FrameView::Seq(n): Feed)]       // `n` is a `u64`
pub struct Sq { .. }
```

It is the same enum-style syntax as for a by-value source (invariant 11 is untouched). A variant's field is the selected part as it is (`&'a Book`, `u64`); a unit variant's part is `()`.

* **One concept, two traits.** `Select<In>` (a stored output: `fn select(&In) -> Option<&Out>`) is unchanged, so hand-written by-value selectors and `#[derive(Demux)]` on enums without a lifetime need no edit. A view selector implements `SelectView<In>`, with `In` the view *with its lifetime* (`impl<'a> SelectView<FrameView<'a>> for frame_view::Book { type Out = &'a Book; const KEY: u32 = 0; fn select(x: FrameView<'a>) -> Option<&'a Book>; fn key_of(x: &FrameView<'a>) -> u32; }`): no GAT is spelled by the user, so the E0195 trap of the prototype does not exist, and `option::Some`/`None` and `result::Ok`/`Err` (which have `SelectView` impls too) work on any view that is an `Option` or `Result`: `Some(FrameView::Book(b)): Feed2` when `View<'a> = Option<FrameView<'a>>`. The source decides which trait a chain needs, all the way down: `Select` for the root and by-value components, `SelectView` for a handle component.
* **How the source decides.** The derive wraps every non-empty chain in `<N::Kind as ChainWrap<N, C>>::Out`: `C` itself for the root and by-value components (a blanket impl over `ValueKind`, which `Node` pins, so generic `X: Node<Output = T>` code compiles as before), a `ViewC<N, C>` for a handle component. `ViewC` is a `SelectChain<N::Stored>`, so the subscription tree and every bound that carries a chain are unchanged; what needs the parent's state (matching, the switch key, reading inside the split) has its own impls in `gather.rs` (`ChainTest`, `ChainTestCx`, `ChainKeyAt`, `ChainReadCx`, `ChainSeed`); `Ready` and the switch got the states as an argument.
* **Run time.** A switch on a handle source takes its key from the view, like one on a by-value source: when the source's run is the node just before it, the two are fused (`PG_RUNSW`) and the key is computed from the view of the source's state and the handle `update` just returned, before the handle is stored (`ChainKeyPre`); otherwise `ChainKeyAt` computes the view from the stored handle. The leaf then reads `select` of a fresh view. A switch arm that is a view run (a run of a component with a handle parent, alone or followed by one plain run) is inlined into the switch instead of outlined (`Plan::prog_view_leaf`): outlined, the arm waits for the handle to come back from memory and a handle source was 55% slower than the same graph with a by-value source (7 feeds, below). For a by-value source nothing changed (measured below).
* **Multiple subscribers, wide subscribers, a parent that did not fire** follow the by-value rules: the OR rule and exclusivity are decided from the selectors' `KEY`s as before; a selected edge fires iff its parent fired and the view matches, and the parent's flag is tested first, so a parent that did not run in this step runs no subscriber even if its last view would match (`tests/handle_selectors.rs`).
* **`on_update` of a selected entry** gets exactly the selected view, as `E::Arg<'_>`: `&Book` for `FrameView::Book(b): Feed` (the marker `<struct>_edges::B` is keyed as for by-value sources).
* **`new` gets `Option` of the selected view**, not a `SelInit`: `Some(part)` if the chain matches the parent's view as it was constructed (its first handle), else `None`. There is no `Initial` for a view (a `&Book` has no default and nothing is stored to start from), so a payload without `Default` costs nothing.
* **A selected value from a handle is never stored.** An entry whose subscriber also runs when the selectors do not match (the case that gives a by-value input a slot) is a compile error that names the input: ``X: the input `n` selects from a handle component and is read in steps where its selectors do not match, so its value would have to be stored, and a view is not stored. Make the source a by-value component (`Component`), or move this subscriber below a component that keeps the value, or make the subscriber's other conditions imply the match``. Direct reads (the condition implies the match) need no storage.
* **Mistakes.** A by-value selector (`Select<T>`, a `Demux` enum without a lifetime) on a handle source, or a view selector of another enum, is ``error[E0277]: the selector chain `..` cannot be applied to the view `..` `` with a note saying which selectors a handle source takes; a view selector on a by-value source is the old ``cannot be applied to a value of type ..`` (`handle_selector_*` in `check_errors.sh`).
* **Two fields spelled alike** (`a: V::X(p): Feed, b: V::X(q): Feed`) are two entries of the plan (the derive no longer merges selected fields), because `new` gets each one's value by move.

**How it works.** `Node` is split in two. The internal, sealed `Vertex` is what the plan, the storage and the drivers use; its `Stored` type is the output of a by-value component and the handle of a handle component. `Edge::Read<'a>` says what a subscriber's field is for a parent, and the public `Node` is a `Vertex` whose `Read` is pinned (`for<'a> Self: Edge<Read<'a> = &'a Self::Output>`) so that the field of a by-value parent is `&Output` to generic code. A handle component implements `Edge` with `Read<'a> = Output<'a>` and is not a `Node`.

A component with a handle parent needs its parent's *state* while its own state is `&mut`: both sit in the same tree of tuples. `store::AtPos::at_mut` borrows the whole tree, so such a component runs through the safe split descent of `src/split.rs`: at each of the (at most four) levels of the constant path to its position the block's tuple is split by pattern, the own element becomes `&mut` and the elements before it become `&`; a parent (always at a smaller position) is picked from the context of the level where the two paths first differ and read down by shared steps. All indices are constants, so after inlining it is the same field offsets as `at_mut`, with no `unsafe`. A component with only by-value parents does not use it: the plan picks the program node kind (`PG_RUN_VIEW` and `PG_RUN_IF_VIEW` for a component with a handle parent, set while the walk records the edge; `PG_RUN` and `PG_RUN_IF` otherwise), so the by-value path has the bounds and the code that existed before handles (`.text` of an app made of by-value components is identical, see below).

**Measured** (`bench/gen_handle_chain.py`, a chain of 64 components, each rewriting a whole output of `K` words from the previous one; framework by value against framework `#[handle]` (state holds the buffer, `update` writes it in place, handle `()`, subscriber reads `&[u64; K]`) against a hand-written `o[i] = step(&o[i-1])` and a hand-written in-place version; release, `taskset -c 9`, min of 15 alternating rounds, best of 3 processes, ns per push; every variant computes the same checksum):

| Output | framework by value | framework handle | hand-written by value | hand-written in place |
|---|---|---|---|---|
| 64 B (K = 8) | 103 | 99 | 102 | 98 |
| 512 B (K = 64) | 274 | 272 | 200 to 277 (two modes between runs) | 274 |
| 4 KB (K = 512) | 5,480 | 2,513 | 5,463 | 2,548 |

Handles cost nothing at 64 B (3% faster), tie at 512 B, and are 2.2 times faster at 4 KB, the same as the hand-written in-place code: the by-value output is built and then copied into the pipeline, which costs about as much as computing it once it no longer fits in a few registers. The crossover is around 512 B (not measured between 512 B and 4 KB). This is the best case for a handle (every word is rewritten each step); a component that updates a little of a big state and returns it by value pays the copy, and one that returns a small value loses nothing. A chain of 300 components (three levels of the split path) at 64 B gives 731 ns by value and 698 ns with handles, same checksum. The prototype's slowdown for small outputs (about 2.5 times, attributed to the indexed read of just-written state) does not appear here with `Handle = ()`; it needs an indexed handle read, which this chain does not exercise.

**Cost of having handles in the library** (cold builds of a generated app that has no handle component, `--mode enum`, `cargo check -p app` and `cargo build --release -p runner`, `taskset -c 0-7,12-19`, 4 jobs, 3 runs each, master `bf9e974` against this branch, peak RSS of the largest process): typical (823): check 7.43 s to 7.76 s (+4%), 409 to 413 MB; release 20.45 s to 21.02 s (+3%), 886 to 898 MB. `m30-300-70` (4,079): check 31.5 s to 34.8 s (+11%), 1,323 to 1,334 MB; release 91.5 s to 93.5 s (+2%), 2,878 to 2,992 MB (+4%). The `.text` of the release binary is byte-identical to master's in all twelve builds (hashes of `objcopy -O binary --only-section=.text`, same directory names, same flags), so the run time of an app without handles is unchanged. That was the first version, which chose between the paths in the trait solver (`drive::UpdVia`, one more impl selection per position and site, about 3 s of the check at 4,079) and called `update` and `new` through a derived wrapper (`Comp::__update`/`__new`: two more items per component for the compiler to collect, which is where the 4% of release memory came from). Now the plan picks the path (a program node kind) and the drivers call `Component::update`/`new` directly (a handle component gets its `Component` impl from the derive, with the handle as `Output`). Measured the same way, 3 alternating runs against master: 4,079 check 51.2 s against 50.9 s (+1.5%), 1,579 MB against 1,572 MB; typical check 12.1 s against 11.9 s (+2%); release 4,079 94.0 and 100.2 s for master against 95.7 and 99.4 s (two runs each, the machine was loaded), peak 2,956 MB against 2,947 MB (+0.3%); typical release 20.9 s for master against 22.1 to 22.5 s (the second master run was 25.5 s, so noise), peak 904 MB for master against 883 MB; the release `.text` is still byte-identical at both sizes. What is left of the check cost is the `Node`/`Vertex`/`Edge` split itself (about 0.5 s of 27.5 s in a `-Ztime-passes` bisect).


## How it works, start to finish

1. **Macro expansion** (`dataflow-macros`). The derive turns `#[subscribe(..)]` into: the `Input`/`Init` structs, `ComponentSubscription::Deps` (the subscription list as a type-level tree: leaves of up to 15 entries, inner nodes of exactly 16 subtrees, `ARITY = 16` defined once in `dataflow-macros`, every per-shape impl generated from it by callback macros), and the node impls (`Edge`, `Vertex`, `Node`; for `#[handle]`, `Edge`, `Vertex`, `HandleView` and a forwarding `Component`). Each entry becomes `Sub<Node, SelectorChain, position>`; the position tells apart two entries naming one type through an alias. The generated tokens are re-spanned to the struct's identifier so that moving a declaration does not turn unrelated incremental queries red.
2. **Type checking** (`cargo check`). Every subscription is a node, `update`'s inputs match the upstream outputs, the selector chains apply to the output types. `cargo check` does not monomorphise, so the plan's errors only appear with `cargo build`.
3. **Plan** (`plan.rs`, constant evaluation). The depth-first walk from the sinks runs once in a `const fn` (`const_trait_impl`), recording every node's `TypeId`, first parent, execution position, subtree end and upstream edges; each selector as a *decision* `(parent, position in the chain, key)`; every component's *condition* (terms of decisions, absorbed, checked pairwise exclusive); which selected inputs need a stored slot; the listener requests' structure; and the **push program**: `Seq` (children in execution order), `Run`, `RunIf` (for conditions a trie cannot express, such as guards), `Switch` (one `key_of` call at a place, then a decision tree over the sorted constant keys, which LLVM lowers to jump tables) and `PG_HRUN` (a handler's run). The plan is integer arrays only. A failed plan produces an empty program and one error, because a const panic inside the plan once caused a 13 GB hang.
4. **Flat storage** (`flat.rs`, `store.rs`). The states of the components and their stored outputs are two balanced trees of blocks (16-ary) with one value per component in execution order. The type at a position is found by a halving descent of the first-parent tree, so the solver depth is about log2 of the graph depth. A position is read or written by a constant descent that is one field offset after inlining: one function per position walks a path of at most four digits (`store::path`) through projection steps that are shared by every position under the same child. Stored selector values take positions after the components.
5. **Construction** (`drive.rs`, `gather.rs`, `build.rs`). `Pipeline::build` visits positions in execution order; each `new` is given the outputs built so far. This is the only place with runtime lookups (a `TypeId` map of configs and of listener requests, `dyn Any` boxes).
6. **Push** (`drive.rs`). The same driver walks the program. Every step is a separately monomorphized function, `#[inline(always)]` in release builds, so a push is straight-line code with jump tables at the branches.

The price of being `unsafe`-free is `generic_const_exprs`, which is confined to this crate: user crates need no feature gate, and generic user code needs one bound, `S: Sinks<R>` (see `tests/generic_user.rs`).

## Guarantees and where each error shows up

| Guarantee | Enforced by |
|---|---|
| Each concrete component type is constructed once, run at most once per push, dropped once. Generic instantiations are different types; the same type subscribed twice, also through an alias, is one dependency. | Typed storage; `tests/lifecycle.rs`, `tests/duplicates.rs`, `tests/same_type_name.rs` |
| Execution order: depth-first post-order in listing order, restricted to what runs | `tests/order.rs` |
| Outputs persist between pushes; the root's value does not | Storage; `tests/lifecycle.rs` |
| `#![forbid(unsafe_code)]`; no `dyn` and no lookup in a push | Compiler; code review |
| User crates need no `#![feature]` | `tests/*.rs` are user crates |

| Mistake | Where | What you see |
|---|---|---|
| Typo in an input field | `cargo check` | `E0609` |
| Subscribing to a non-node | `cargo check` | `E0277` mentioning `Node` |
| Wrong input type, duplicate name, bad pattern, `=>` | `cargo check` | See above |
| Selector chain that does not apply to the output | `cargo check` | ``the selector chain `..` cannot be applied to a value of type `..` `` |
| Handler without an `UpdateHandler` impl, missing `Request<S>`, missing `Clone`/`Default` on a stored payload | `cargo build` (mono) | The `on_unimplemented` texts quoted in the sections above |
| Dependency cycle | `cargo check` | ``error[E0275]: overflow evaluating the requirement `B: dataflow_flat::Vertex` ``, naming a component of the cycle |
| Component not reachable from the root | `cargo build` | ``error[E0080]: evaluation panicked: ..::Orphan is not reachable from the root`` |
| A second root, or a root that is not `R` | `cargo build` | `E0080` ``.. is a root, but the pipeline's root is a different type`` |
| Joining branches that can fire together | `cargo build` | ``error[E0080]: evaluation panicked: ..::J joins branches that can fire in the same step`` |
| More than `MAX_COMPONENTS` (32,768) components | `cargo build` | ``evaluation panicked: too many components: raise dataflow_flat::MAX_COMPONENTS`` |
| A config for a component outside the graph | `Pipeline::build` returns | `BuildError { unused: [..] }` |
| Generic code without the `Sinks<R>` bound | `cargo check` | `E0277` ``not valid sinks`` / `Sinks<R>` |
| `#[handle]` without `impl HandleComponent` | `cargo check` | `E0277` ``the trait bound `X: HandleComponent` is not satisfied`` |
| A `Node<Output = ..>` bound on a handle component | `cargo check` | `E0277` ``is not a by-value Node: its subscribers read a view, not `&Output` `` |
| A by-value selector (or one for another view) on a handle component | `cargo check` | `E0277` ``the selector chain `..` cannot be applied to the view `..` `` |
| A selected input of a handle component that would have to be stored | `cargo build` | ``the input `n` selects from a handle component and is read in steps where its selectors do not match, so its value would have to be stored, and a view is not stored`` |
| `#[handle]` with `#[listener]` | `cargo check` | ``cannot be combined with `#[listener]` yet`` |
| An entry without an `UpdateHandler` impl | `cargo check` | `E0277` ``subscribes to `Limits` without an `UpdateHandler<Limits>` impl`` |
| `#[derive(UpdateHandler)]` and a hand-written impl | `cargo check` | `E0119` conflicting implementations of `UpdateHandler<..>` |

`MAX_EDGES` (131,072 subscription entries), `MAX_DECISIONS` (32,768) and `MAX_TERMS` (8,192 distinct conditions per component) fail the same way; each message names the constant to raise. `scripts/check_errors.sh` runs 56 cases, each a tiny crate that must fail (or, for the `_ok` cases, succeed) with the expected code or message.

## Performance and compile time

User priorities, in order: release run time; release build of the typical app (one NIC, several exchanges by symbols, hundreds of components, mostly at most 7 subscriptions, a few wide ones) under a minute; dev incremental rebuild after editing one big component in its own file plus a few small ones.

### Build flags and files

* **Release means every compiler optimization.** The `[profile.release]` in `Cargo.toml` (`opt-level = 3`, `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`, no debug assertions) and `-Ctarget-cpu=native`. Numbers from a build without either are not release numbers. Because `-Ctarget-cpu=native` is a rustflag in `.cargo/config.toml`, it applies to dev builds too and the binaries are not portable.
* **`.cargo/config.toml`** sets `rustflags = ["-Zshare-generics=n", "-Ctarget-cpu=native"]`. `-Zshare-generics=n` exists for the dev edit loop: with sharing on, `exported_generic_symbols` re-runs on any edit that changes a line's length (a dev rebuild on 823 components took 4.9 s instead of 0.8 s), through the normalization and constant evaluation it triggers. Release time, binary size and run time did not change with it. `bench/gen_app.py` writes the same file into generated apps. Apps of your own should copy it.
* **`MALLOC_CONF` in `[env]`.** `.cargo/config.toml` has `[env] MALLOC_CONF = "dirty_decay_ms:-1,muzzy_decay_ms:-1"`, so the jemalloc inside rustc never returns memory to the OS. This nightly's rustc reads `MALLOC_CONF`; `_RJEM_MALLOC_CONF` is ignored. (An earlier version of this tree used a `rustc-wrapper` script exporting `_RJEM_MALLOC_CONF`; it did nothing.) Without it, large checks spend most of their time in kernel page faults: cold `cargo check -p app`, `--mode enum`, 4,079 components, incremental on, measured here with `perf stat -e page-faults` on 2026-11-02: 13.4M faults and 52 s with jemalloc's defaults, 0.68M and 33 s with `[env]`. Note that cargo merges `.cargo/config.toml` files up the directory tree, so removing the file in a generated app that lives inside this repo does not switch the setting off; override it with a shell variable instead (`MALLOC_CONF=dirty_decay_ms:10000,muzzy_decay_ms:0`). For a release build use that override: it cuts peak memory (typical, full profile: 0.88 GB, against 1.34 GB with `[env]`) at no cost in time (about 32 s cold in each case), but it loses the check gain (4,079 cold check: 50 s and 12M faults, against 32 s and 0.68M with `[env]`). Muzzy-only (`muzzy_decay_ms:-1`) keeps the gain at 4,079 (32.3 s, 0.79M) but fails at `perps`: a 500 s timeout with 256M faults at 460 s, against 119 s, 5.28 GB and 1.68M faults with both set to -1. Timed dirty values (10000, 30000) lose the gain at 4,079 already. So keep both at -1 in `[env]` and override only for release. Measured 2026-11-02, single runs: `-1`/`-1` check 31.8 s, 1.49 GB, 0.68M faults; `muzzy_decay_ms:-1` alone 32.3 s, 1.39 GB, 0.79M; `10000/0` 49.8 s, 1.34 GB, 12.4M; `10000/10000` 47.4 s; `30000/0` 50.7 s.. At 2,044 components the fault count was about 0.5M either way, so the effect only shows at thousands of components.
* **`#![allow(long_running_const_eval)]`.** The plan is one constant. Above about 1,300 to 2,000 components its evaluation passes rustc's `long_running_const_eval` limit, which is a deny-by-default lint; the allow can only be given in the crate that evaluates the plan, i.e. the application crate. Without it the build fails with a misleading `E0275` instead of naming the lint (reported; the text was not reproduced for this document, so it is not quoted). `bench/gen_app.py` adds the allow above 1,500 components. Equivalent: `-A long_running_const_eval` in the app's cargo config.
* **`CARGO_INCREMENTAL=0`** is no longer the main advice for large cold checks: `MALLOC_CONF` removes most of the page-fault cost. Incremental off is still a little faster when no edits follow (`perps`, older `on` mode: 82 s, 6.7 GB). Keep incremental on for the dev edit loop: at the typical size an edit is 0.7 s incremental and 8.5 s with it off.
* **Large release builds.** Peak memory and time (full profile, `--mode enum`, `MALLOC_CONF=dirty_decay_ms:10000,muzzy_decay_ms:0`, 2026-11-02): typical 0.93 GB, 4,079: 2.94 GB / 101 s, 6,114: 4.33 GB, `perps` (12,412): 8.75 GB / 416 s, `full` (30,186): 19.66 GB / 1,537 s. The front end sets the peak at these sizes (`full`: 20 GB at the end of codegen, 247 s in the app crate; LLVM stays near 7 GB). Time still grows faster than components: at `full`, LLVM takes 497 s in the app crate and fat LTO 602 s in the runner.

### Measured numbers

All on the author's 4-core machine, nightly-2026-10-30. They are copied from `handoff/session-2.md` (dated entries); I did not re-measure them for this document. "typical" is the 823-component generated app, "tiny" 111, "small" 415, `perps` 12,412, "enum" is the current selector mode and "on" the old keyed-selector mode (`On<S>`, removed; the two differ by about 2% at typical).

| Date | What | Profile / mode | Result |
|---|---|---|---|
| 2026-11-01 | typical, run time, framework vs hand-written `match` | release, `enum`/`on`, pinned core | 107.6 vs 99.6 ns per packet (about 8%); gap accepted by the user |
| 2026-11-01 | typical, release build (app crate / runner) | release, ARITY 16 | 12.8 s / 19 s |
| 2026-11-01 | typical, first dev build / edit | dev, incremental | 10.4 s / 0.8 s |
| 2026-11-02 | 823 / 2,044 / 4,079 components, release build | release, `enum` | 0.87 GB, 16 s / 1.86 GB, 59 s / 3.83 GB, 226 s |
| 2026-11-01 | 2,044 components, cold `cargo check` | `on`, incremental | 17.3 s (after the `excl_group` fix) |
| 2026-11-02 | 4,079 components, cold `cargo check -p app`, single runs, shared machine | `enum`, incremental | jemalloc defaults 48.9 s, 1.33 GB, 12.7M page faults; `_RJEM_MALLOC_CONF` (ignored) 49.6 s, 13.1M; `MALLOC_CONF` 30.0 s, 1.49 GB, 0.68M; via `[env]` 29.7 s, 0.68M (my own run: 52 s, 13.4M vs 33 s, 0.68M) |
| 2026-11-02 | `perps` (12,412), cold `cargo check -p app` | `enum`, incremental, `MALLOC_CONF` | 125.9 s, 5.29 GB, 1.68M page faults |
| 2026-11-02 | older figures for `perps` and 4,079, cold `cargo check` | `on` mode, jemalloc defaults | 4,079: 46.5 s incremental, 25.1 s with `CARGO_INCREMENTAL=0`; `perps`: 82 s with `CARGO_INCREMENTAL=0`, 995 s incremental (2,004 s while self-profiling) |
| 2026-11-01 | 827 / 2,214 enum variants, `cargo check` | `enum` | 12.6 s / 67.9 s (3.0 GB); release at 2,214 timed out at 1,200 s |
| 2026-11-02 | chain of N components (`C0 <- Poll` .. `C(N-1)`), debug build | default recursion limit | 85 builds in 2 s; 90 in 14 s; 95 was killed by memory pressure at a 2 GB cap |

**Run-time noise threshold:** under 5 ns per packet at 1,000 components is noise on this machine (a gap that size changes with layout, alignment and the VM). Don't act on smaller differences without callgrind instruction counts and several processes. The measurement protocol (`scripts/bench.sh`, `scripts/measure.py`): pinned core, warm-up, alternating framework/baseline rounds in several processes, median of per-process minimum.

### PGO and compiler flags

Profile-guided optimization on top of the full release profile gains about 4% (framework) and 3.5% (hand-written baseline) at typical, measured pinned to one core of a Ryzen 9 7900X with `bench/gen_app.py --size typical --mode enum` (ns per packet, min / median of 8 alternating processes):

| Build | Framework | Baseline | Ratio |
|---|---|---|---|
| Full release profile | 108.1 / 109.3 | 102.6 / 103.3 | 1.054 |
| PGO, trained on a different seed and key subset | 103.8 / 104.2 | 98.9 / 99.4 | 1.050 |
| PGO, trained on the measured input (overstates) | 102.6 / 103.0 | 99.3 / 99.7 | 1.033 |

With a different training seed: small 90.6 to 86.2 (-4.8%, baseline -2.4%), tiny 56.3 to 52.6 (-6.6%, baseline -5.2%). A PGO build costs the same as a release build (about 31 s and 1.4 GB at typical) plus one instrumented build and a training run. Nothing else helped (all within 1% at typical): `-Cllvm-args=-inline-threshold=1000|3000` (11 s slower to build), `-Zmir-opt-level=4` (framework 1% slower), `-Zinline-mir-threshold/-hint-threshold`, `-align-all-functions=6`, `-align-all-nofallthru-blocks=5`, `-Clink-arg=-fuse-ld=lld` (1% slower), `-Zmerge-functions=disabled`, `-unroll-threshold=600`. BOLT is not installed. Run `scripts/pgo.sh APP_DIR [TRAIN_SEED]` on a generated app; it builds the instrumented runner, trains with `--seed`, merges with the toolchain's `llvm-profdata` and writes `APP_DIR/target-pgo/release/runner`.

### What costs time

* **Shared tails (`PG_TAIL`).** A component joined across exclusive branches (a market's depth and trade updates feeding the same cross, quoters and sinks) used to be compiled once per branch, so the tail's code existed twice per market. The plan now finds the plain runs that end every branch of a switch (`Plan::share_tail`, at least `TAIL_MIN` = 2) and emits them once as a node that every branch calls (`drive::exec_tail`, `#[inline(never)]` in release). Order is unchanged. Measured against 2ebfcdd with the generated `enum` app: 4,079 components 97.0 to 93.0 ns per packet (gap to the hand-written baseline 10.7 to 6.7 ns, build 117 to 101 s); typical no change outside noise (gap 3.2 to 2.5 ns); `perps` (12,412) 116.81 to 109.33 ns minimum (gap 26.8-28.4 to 15.6-18.7 ns), release build 537 to 472 s, peak 12.08 to 8.64 GB, text 10.48 to 9.55 MB. The rest of the gap is not explained; the leading suspect is the storage layout (a market's states and outputs are in separate blocks per component type, the hand-written code keeps them in one struct).
* **Wide components run through a function of their own (`WIDE_INPUTS` = 24).** A component with 24 or more subscriptions (`DepList::LEN`) is run by `drive::run_wide`, `#[inline(never)]` in release, one instance per position; before, its gather (a few dozen IR lines per input) was inlined into every switch arm and shared tail that runs it, so the push grew with the square of the graph (a risk sink over every cross, a sender over every quoter: a tail of 8,889 lines at 4,964 components, 17,154 at 9,929, and 174 to 387 of them). Found with `-Cno-prepopulate-passes -Cpasses=always-inline --emit=llvm-ir`: the IR after the AlwaysInliner grew 9.3 M, 18.4 M, 30.3 M lines at 4,964, 7,447, 9,929 components (the IR before it 3.8, 5.6, 7.5 M). The app crate's LLVM peak at 12,385 components fell from 7.2 to 2.95 GB and its LLVM time from 207 to 163 s; at 4,079 components the framework's run time is unchanged (min of 6 alternating runs on one core: 91.66 before, 91.43 after, ns per packet). At `perps` it is a large gain: framework min 110.01 to 96.41 ns per packet, gap to hand-written code about 17 to about 5 ns (6 alternating runs), build 472 to 416 s. The cost is one call per run of such a component.

* A body edit in the app crate re-runs no plan or trait query and recompiles 1 to 4 of about 80 codegen units. A body edit in the *library* crate re-runs many constant evaluations (about 2.9 s at typical).
* Typical mono items per component: about 48 (ARITY 16; 92 before the shared projection steps of the storage descent).
* Keyed dispatch compiles to compares plus jump tables, whose depth grows with the number of variants; at thousands of variants it dominates release time.
* Selectors cost more than anything else in `cargo check`: selector apps take 2 to 2.8 times longer to check than the same graph without.

### Tools

`bench/gen_app.py --size tiny|small|typical|perps|full|m<L>-<B>-<H> --mode 1a|enum|req` generates a multi-crate app (`marketlib/`, `app/`, `baseline/`, `runner/`). **`--mode` is required.** Use `enum` (selectors through one `#[derive(Demux)]` enum per exchange) for anything meant to resemble a real app, `req` for `enum` with `#[listener]` feeds, and `1a` only for the old no-selector shape. `scripts/measure.py --mode ..` drives it (its default `--out` is `target/harness`); `bench/gen_handler.py N OUT [for]` builds an OMS-style handler graph; `bench/gen_enum_keys.py` compares enum keys; `scripts/gen_selbench.py` writes `examples/selbench.rs`. `bench/gen_bench.py full 20` regenerates `bench/src/main.rs`; the committed file should be what the generator produces.

## Design history and rejected alternatives

Read this before trying something "simpler"; most simpler ideas were tried. The graph model changed several times (a pull-based tree fold, a single root component, then the current event model); rows for designs that no longer exist are collapsed.

| What | Outcome |
|---|---|
| Runtime registry, `HashMap<TypeId, Rc<dyn Any>>` | Everything is checked at run time. Rejected by R2. |
| A `container!` macro listing every component | Someone has to list all of them. Rejected by R3. |
| Dependency-injection handles instead of dataflow | Superseded by R4. |
| `Vec<Box<dyn Node>>` with `Box<dyn Any>` outputs | Dynamic dispatch and downcasts. Rejected by R6. |
| `Rc<Node<C>>` handles with a per-run epoch check | Stable, but about 10x slower than a hand-written struct on trivial components. |
| Specialization for type-level deduplication | **Does not work**: specialized associated types stay opaque during type checking. |
| A plan built from a graph of `&'static` description constants | Correct, but incremental builds were **exponential** (L=10: 67 s): incremental compilation fingerprints a constant by following its references and does not remember shared nodes, so a 4-way graph was hashed as a tree. The plan is now integer arrays only; this is why constants hold no references. |
| Storage as raw bytes with `generic_const_exprs` in every user crate | Needed `unsafe` and leaked the feature gate. Rejected by R7 and R10. |
| Typed storage, GCE confined to the library | Kept: each component stored once, safe construction, drop by glue. |
| `#[derive(ComponentSubscription({ sub1: X }))]` | Not valid Rust: derives take no arguments. Became derive + `#[subscribe(..)]`. |
| `Option<Output>` per component, `Cell`/`RefCell` outputs | A check or borrow flag per read. Stored outputs are plain slots in a flat tuple. |
| frunk-style `Selector<T, I>` with inferred indices | Blocked by E0207/E0283 in generic impls and when two outputs share a type. |
| `core::field::field_of!` / `type_info` to find an output field | On this nightly only an offset through an unsafe trait; `update`-only output via reflection was built as an experiment (`handoff/experiments/update-only/`) and stashed because it needs `unsafe`. |
| Cycle check as a bound on `Pipeline` / `Planned` / every `Component` | Reported as E0391 first, or +7 s, or +30%. The walk's own trait bounds prove acyclicity for free and give E0275. |
| Const function pointers in the plan | Do not compile on this nightly. |
| Subscription lists as a cons list | Depth of every proof grew with the list: 20 subscriptions overflowed the recursion limit. Now a fixed 16-ary tree, fill-first, per-shape impls generated from one constant. |
| Component identity by `fnv1a(type_name)` | Rejected: a hash must never decide identity. `TypeId` with `==` only, found through `table.rs` (name hash picks where to start; `TypeId ==` decides). A linear scan of the `TypeId`s was +1.2 s at 397 components. |
| `#[inline(always)]` everywhere | Exponential in debug builds when it forces inlining before dead branches go. Now release-only (`cfg_attr(not(debug_assertions), inline(always))`) on the run path; lookups are plain `#[inline]`. Without it, release run time fell off a cliff for large graphs. |
| Demux as graph nodes (the first event design: `packet::Binance<P>` nodes, phase 1b) | Replaced by selectors on the subscription: no node types parameterised by their parent, shorter subscriptions, one node kind besides the root. |
| `name: Src => Sel1, Sel2` | Replaced by patterns (`Sel1(Sel2(name)): Src`), which read like the `match` they are. `=>` is a parse error. |
| Rule "a selected edge's value must be implied by the reader's condition" | Replaced by stored selector values: the plan keeps the last matched value for exactly the inputs that need it. |
| Keyed selector `On<S>` over a `Frame { key, .. }` with `Symbol`/`Keyed` | Removed; enums are the only keyed form. Run time was equal or better with the identical bodies (3.97 vs 11.89 ns at 827 symbols with random keys), the gap being LLVM merging identical arms; with different bodies the two are equal (73.4 vs 72.8 cycles). Kept in branch `keep-on-mode`. |
| `#[derive(Handler)]` / `#[handles(..)]` / `Collect { begin, finish }` | Replaced by ordinary components with unnamed `#[subscribe]` entries and `UpdateHandler`. |
| `Blk<R, M, LO, HI>` named storage nodes | Merged: peak memory at 4,079 components 14.6 -> 6.2 GB. |
| `Field` (direct child index) | Not merged: `SelP::POS` work fell 30x but `check_well_formed` rose. |
| ARITY 8 / 16 / 32 | 16 kept: mono items 69.6k / 58.4k / 48.4k, typical release 13.9 / 12.8 / 12.1 s; 32 costs more at 2,044 and 4,079 components. |
| `#[inline(always)]` hint on the user's big component | Hurts typical by about 2 ns. |
| Reordering storage to keep a packet's components together (hypothesis: the gap at 4,079 is layout) | Refuted before any framework change. Storage is already in execution order, which is a depth-first walk, so one market's roughly 10 components and their outputs are contiguous (`Pipe` is 3,768 bytes for the 111-component app against 3,440 for the baseline's market structs plus the rest). Test with `BASE_SOA=1|2 bench/gen_app.py --mode enum`: the hand-written baseline at 4,079 components (85.4 ns) with states and outputs in two arrays like the framework's two trees: +1 ns (86.4); one array per field, i.e. one block per component type: +19 ns (104.5). So the layout the framework has costs about 1 ns, not the 6.7 ns gap, and a cone-ordered storage cannot gain more than that. |
| Out-parameter or `#[output]` field instead of returning `Output` | Not adopted: it would avoid a copy but not make store-forwarding stalls impossible. The advice is "build a local, then return it". |

Smaller decisions: configs are keyed by component `TypeId`; `type Config: Default` lets components go without a config entry; inputs are shared references, so any number of subscribers share one output without cloning; the order guarantee is the order the construction naturally produces, documented and tested so it stays a guarantee; `ARITY` is the one constant that sets the subscription-tree shape.

## Known limitations and rough edges

| Area | Detail |
|---|---|
| Nightly | Needs `const_type_name`, `const_trait_impl` and `const_cmp` (syntax `const trait` / `const impl` / `[const]`), `generic_const_exprs` (incomplete: the nightly prints "`feature(generic_const_exprs)` is not supported with the next-generation trait solver ... reverted the setting", and the library has an `incomplete_features` allowance) and `panic_internals`. Only the library crate is affected. |
| Identity | Exact (`TypeId`). Two types with the same `type_name` (two crate versions) are different components, shown by `Pipeline::order()` under the same name. |
| Capacity | `MAX_COMPONENTS` 32,768, `MAX_EDGES` 131,072, `MAX_DECISIONS` 32,768, `MAX_TERMS` 8,192. At most 15 sinks. Compile-time memory grows with the number of components. The plan's pools (push program 131,072 nodes, child pool 131,072, lists 262,144, key pool 65,536) have private limits with messages such as "the push program is too large: raise dataflow_flat's list pool". What `full` (29,650 components, 30,187 nodes) uses, measured 2026-11-02: 56,503 edges, 8,898 decisions, 23,727 pool words of terms, 60,440 term-list entries, push program 69,341 nodes and 69,334 child entries, lists 90,035, candidate lists 101,033, keys 13,317. Nodes are within 8% of `MAX_COMPONENTS`: `full` is about the largest graph that fits. A plan error is reported once (as `E0080` on `Planned::CHECK`), not once per component. |
| Graph depth | The plan's `Upstream` proof is as deep as the graph. The depth is about 1.1 solver levels per layer plus about 35 for the rest of the pipeline's proof, so at the default `recursion_limit` of 128 a chain builds up to 88 layers (about 4 s in `cargo check`). At 89 rustc warns about the overflow and still finishes (7 s, 0.83 GB). From 90 it fails, after an expensive cascade of follow-up errors (90: 15 s and 2.1 GB, 95: 20 s and 3 GB; this is the steep part, not the build itself). Deeper chains need `#![recursion_limit = "N"]` in the user's crate: with 4096, 250 layers take 6 s / 0.63 GB and 500 take 11.7 s / 2.1 GB (`cargo check`; dev and release builds of 250 layers 7.8 s). Many parallel chains are fine (16 chains of 50: 8 s). Typical graphs are shallow (about 6 levels) and wide. |
| Scale | `long_running_const_eval` allow needed above about 1,300 to 2,000 components. Cold `cargo check` of `full` (29,650, `enum`, patched rustc, empty `MALLOC_CONF`): passes, 310 s, 10.1 GB, 4.8M page faults. Run time above hand-written code: typical about 0 (-0.7 ns, within layout noise), 4,079 components 7.2 ns. Release of 12,412 components: 416 s, 8.75 GB, 4 to 7 ns per packet above hand-written code. Release of 30,186 components: 1,537 s, 19.66 GB, about 8.5 ns above hand-written code (4 runs; the hand-written code slows more than the framework at this size, for a reason not yet known). |
| Handle components | Selectors on them select from the view and are never stored (a selected input the condition does not imply is an error); `new` gets an `Option` of the selected view; no switch fusion with the source's run; no `#[listener]` that is itself `#[handle]`. `view` is not cached (called per subscriber run). Release `.text` is unchanged for apps without them; `cargo check` of a 4,079-component app is 11% slower for the library having them (see the section). A subscriber of a handle component reads `View<'a>` borrowed from the pipeline, so it can't keep it past `update`. |
| Listeners | `Select::REQUEST` is true only for derived selectors; a hand-written selector never reaches `Request`. One component can read several variants of one listener (stored selector values). |
| Unit selector on a data variant | Compiles and ignores the payload. |
| Patterns | Enum-variant style with at most one field; anything else is a hand-written `Select`. `for` bodies are one node type. |
| Inputs | Shared references only. A component can't take ownership of an upstream output. |
| Outputs | Stored in the pipeline and replaced when the component runs; large inline outputs are copied on each run (box big outputs, or make the component a [handle component](#handle-components): 2.2 times faster at 4 KB). Returning a `Vec` allocates per push. No accessor for stored outputs. |
| Per-run external input | Only through the root value of `push` and components that own the I/O (the `Nic` of the exchanges example). Passing the root packet by reference would save about 2 to 3 ns on a slow machine but changes the API; left as is. |
| Threads | Single-threaded. A `Pipeline` is `Send` when its components are. |
| Run time | About 8% over a hand-written `match` at typical with random keys (cause of the last 46 cycles unknown); the enum form is about 2.5% slower than the old keyed form at typical. |
| Debug | Every lookup step is a real call without inlining; large graphs are bigger and slower to build in debug. |
| Macro | Hard-codes `::dataflow_flat` paths (the crate has `extern crate self as dataflow_flat;`). |
| `order()` | Returns `type_name` strings: fine for debugging, not stable identifiers. |

## Inspecting compiler output

Used during development; flag spellings may differ on newer nightlies. They run on the `diamond` example, where the output is small enough to read.

```sh
cargo rustc -p dataflow-flat --example diamond -- -Zunpretty=expanded           # derive output
cargo rustc -p dataflow-flat --example diamond -- -Zprint-mono-items=yes         # instances
CARGO_INCREMENTAL=0 cargo rustc -p dataflow-flat --example diamond -- -Ztime-passes
cargo rustc --release -p dataflow-flat --example diamond -- --emit=asm -C llvm-args=-x86-asm-syntax=intel
# per-query self time: -Zself-profile with -Zself-profile-events=default,args, read with the `summarize` tool
# built from measureme (this nightly writes file format v9, which needs analyzeme/decodeme 12.0.3 or newer).
```

Keep the `-Z` flags identical on every build you compare: changing any of them forces a near-full rebuild. When comparing the `.text` of two builds, build both in directories with the identical name (swap the contents, don't copy to a differently named directory), because paths are embedded in the code. `handoff/tools/` has the profile readers and the `memset` shim used to find the incremental-compilation page-fault cost. Legacy symbol names hide generic arguments; to find an instance in the LLVM IR match the debug-info `DISubprogram(name: "walk<...>")`.

## Toolchain

The repo pins `nightly-2026-10-30` in `rust-toolchain.toml`; bump the date to upgrade deliberately. Without rustup, use `RUSTC_BOOTSTRAP=1`. Features in use, all confined to `dataflow-flat`: `const_type_name` (the hash that starts the plan's table probe), `const_trait_impl` and `const_cmp` (the plan's walk, `TypeId ==` in a `const fn`), `generic_const_exprs` (incomplete), and `panic_internals` (a const panic with a component's name). User crates (`dataflow-flat/examples/*`, `bench`, the tests, the error cases) need no `#![feature]`.
