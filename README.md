# dataflow: compile-time-wired component graphs in Rust

A framework for building programs out of many small components (designed for hundreds) that form a directed acyclic graph. Each component is constructed from an owned config, has one `update` method that receives the outputs of the components it subscribes to, and produces an output of its own. You write one ordinary *root* component (`App`) that subscribes to what the program wants. Everything upstream of it is discovered by the compiler, built exactly once per concrete type (generic instantiations count as different types), and updated once per run in a documented order. `run` returns the root's output by value. At run time there is no dynamic dispatch, no lookup, no bookkeeping and no `unsafe`: the release build of a `run` is the same machine code a hand-written struct would produce.

**Status:** working prototype, and the focus is still on the design rather than polish. Everything in this document was built and checked on the pinned nightly (see [Toolchain](#toolchain)).

## Contents

1. [Requirements that shaped the design](#requirements-that-shaped-the-design)
2. [Quick start](#quick-start)
3. [Workspace layout](#workspace-layout)
4. [Writing components](#writing-components)
5. [How it works, start to finish](#how-it-works-start-to-finish)
6. [Guarantees and where each error shows up](#guarantees-and-where-each-error-shows-up)
7. [Performance and compile time](#performance-and-compile-time)
8. [Design history and rejected alternatives](#design-history-and-rejected-alternatives)
9. [Known limitations and rough edges](#known-limitations-and-rough-edges)
10. [Suggested next steps](#suggested-next-steps)
11. [Inspecting compiler output](#inspecting-compiler-output)
12. [Toolchain](#toolchain)

## Requirements that shaped the design

These came from the person who asked for the framework, roughly in the order they arrived:

| # | Requirement |
|---|---|
| R1 | Components depend on other components. If two components depend on the same component, only one instance exists. Each concrete type, including generic instantiations such as `Component2<u32>` vs `Component2<String>`, is a singleton. |
| R2 | Dependencies are resolved at compile time. |
| R3 | With hundreds of components, the user specifies only the minimal set. Nothing else is listed anywhere. (Now: the single root component.) |
| R4 | Each component takes an owned config struct in its constructor and has a single `update` that takes the results of upstream components and produces its own result. |
| R5 | Components are topologically sorted and invoked in that order. The order is a documented guarantee, not an implementation detail. |
| R6 | No dynamic dispatch. |
| R7 | Nightly features are acceptable. They must not leak into user crates, though: user code needs no feature gate. |
| R8 | Declaration syntax: the user names subscriptions, and `update` receives a struct with named fields (`input.sub1`), not a tuple. |
| R9 | One root component; `run` returns its output by value. No tuple macros in the public API. |
| R10 | `#![forbid(unsafe_code)]` in the framework. |

## Quick start

```sh
cargo run -p example                     # text pipeline: 8 components under one root
cargo run -p diamond                     # smallest interesting graph (4 components)
cargo test                               # plan assertions + runtime guarantees + execution order
scripts/check_errors.sh                  # 14 cases: each mistake fails the documented way

python3 bench/gen_bench.py full 20       # regenerate the 396-component benchmark (+ root)
cargo run --release -p bench             # flat runtime vs hand-written
```

`rust-toolchain.toml` pins a nightly. Without rustup, set `RUSTC_BOOTSTRAP=1` to unlock nightly features on a compatible compiler.

**Wrap builds in a timeout** (`timeout 120 cargo ...`). Large graphs compile slowly (the release benchmark build takes about 8 s), and past bugs made builds exponential (see [history](#design-history-and-rejected-alternatives)). A timeout turns a regression into a failure instead of a hang.

## Workspace layout

| Crate | Toolchain | Role |
|---|---|---|
| `dataflow-flat` | nightly | **The runtime.** Compile-time plan (`plan.rs`), typed storage tree (`storage.rs`), lookups (`lookup.rs`), safe fold (`walk.rs`), startup (`build.rs`, `pipeline.rs`). `#![forbid(unsafe_code)]`. |
| `dataflow-macros` | any | `#[derive(ComponentSubscription)]` and `#[subscribe(...)]`. Re-exported by `dataflow-flat`. |
| `example` | nightly | Text pipeline: `Source → Tokenize → {WordCount → TopWords, Longest, Len<Tokenize>} → Report → App`. `run` returns a `Summary`. |
| `diamond` | nightly | `Src → {Double, Square} → Sum` (the root). Small enough to read its MIR, LLVM IR and assembly. |
| `bench` | nightly | Generated 396-component, 1,504-edge layered graph plus a root, run through the flat runtime and a hand-written baseline. Not a default member. |
| `scripts/check_errors.sh` | | Compiles small crates that each contain one mistake and checks the error. |

## Writing components

```rust
use dataflow_flat::{Component, ComponentSubscription};

#[derive(ComponentSubscription)]
#[subscribe(
    doubled: Double,          // name: component type
    squared: Square,
)]
pub struct Sum {
    calls: u64,               // the component's own state
}

impl Component for Sum {
    type Config = ();         // owned, consumed by `new`; must be Default + 'static
    type Output = u64;        // subscribers receive `&u64`; must be 'static

    fn new((): ()) -> Self {
        Sum { calls: 0 }
    }

    fn update(&mut self, input: Self::Input<'_>) -> u64 {
        self.calls += 1;
        input.doubled + input.squared       // both &u64, from this run
    }
}
```

A component that subscribes to nothing uses the derive with no `#[subscribe]`. Generic components work in both directions. A component can subscribe to its own type parameter, as in `#[subscribe(items: U)] struct Len<U: Component<Output = Vec<String>>>(PhantomData<U>)`. It can also have parameters that no subscription uses, such as `struct Worker<Q>` subscribing to `Sum`, where `Worker<EmailQueue>` and `Worker<BillingQueue>` are separate singletons. If two subscriptions name the same type, as in `a: Double, b: Double`, they share one dependency and both fields point at the same output.

### The root component

The application defines **one ordinary component** whose subscriptions are what the program wants. Nothing subscribes to it, and its output is what `run` returns:

```rust
pub struct Summary { pub report: String, pub token_count: usize }

#[derive(ComponentSubscription)]
#[subscribe(report: Report, token_count: Len<Tokenize>)]
pub struct App;

impl Component for App {
    type Config = ();
    type Output = Summary;
    fn new((): ()) -> Self { App }
    fn update(&mut self, input: Self::Input<'_>) -> Summary {
        Summary { report: input.report.clone(), token_count: *input.token_count }
    }
}
```

```rust
use dataflow_flat::{Configs, Pipeline};

let configs = Configs::new()
    .with::<Source>(SourceConfig { lines: vec![/* ... */] })
    .with::<Tokenize>(TokenizeConfig { lowercase: true })
    .with::<TopWords>(TopWordsConfig { n: 2 });

let mut pipeline = Pipeline::<App>::build(configs)?;   // Err lists every unused config
loop {
    let summary: Summary = pipeline.run();             // each component once, in the documented order
}
```

`Configs` is keyed by component type, not config type, so two components, or two instantiations of one generic component, never share a config by accident. A component without an entry gets `Config::default()`.

**Outputs live only during `run`.** The root's output is returned (so it can be an owned struct, a `String`, a `u64`, anything `'static`); every other component's output is dropped before `run` returns. Nothing keeps outputs between runs, and there is no `outputs()`. Components keep their own state between runs. If you want to keep something from an upstream output, copy it in the root's `update`.

**Generic code over the root** needs exactly one bound, `Root`, which is implemented automatically for every component:

```rust
use dataflow_flat::{Pipeline, Root};
fn drive<A: Root>(p: &mut Pipeline<A>) -> A::Output { p.run() }
```

### Execution order

The order of `update` calls is a guarantee, checked by `dataflow-flat/tests/order.rs`. It is a depth-first post-order that follows subscription listing order. For a component `C` subscribing to `[D1, .., Dn]`, the updates that happen during `C`'s walk come in blocks: block `i` is `Di` plus everything upstream of `Di` that has not already run, in this same order. The blocks come in listing order, then `C`. A later-listed dependency that is upstream of an earlier one therefore runs inside the earlier block. For the example graph the order is `Source, Tokenize, WordCount, TopWords, Longest, Report, Len<Tokenize>, App`. `Pipeline::order()` reports the construction order, which is the same order.

## How it works, start to finish

The traits (in `dataflow-flat/src/component.rs`):

```rust
pub trait ComponentSubscription: 'static {
    type Deps: DepList;                // type-level list of subscriptions; only the runtime uses it
    type Input<'a>;                    // generated struct with one named reference per subscription
    fn input<'a>(raw: <Self::Deps as DepList>::Inputs<'a>) -> Self::Input<'a>;
}

pub trait Component: ComponentSubscription + Sized {
    type Config: Default + 'static;
    type Output: 'static;
    fn new(config: Self::Config) -> Self;
    fn update(&mut self, input: Self::Input<'_>) -> Self::Output;
}
```

The five stages are: macro expansion, type checking, the compile-time **plan** (`plan.rs`), the **states tree** (types, `storage.rs`), and the **fold** that runs it (`lookup.rs`, `walk.rs`).

### 1. Macro expansion

`#[derive(ComponentSubscription)]` reads `#[subscribe(...)]` (`#[derive(...)]` itself can't take arguments, so the subscriptions go in this helper attribute, the same way `serde` uses `#[serde(...)]`). For `Sum` it generates:

```rust
#[doc(hidden)]
#[allow(non_snake_case)]
mod __Sum_input {
    use super::*;
    pub struct SumInput<'__input> where Double: Component, Square: Component {
        pub doubled: &'__input <Double as Component>::Output,
        pub squared: &'__input <Square as Component>::Output,
        #[doc(hidden)] pub __marker: PhantomData<(&'__input (), fn() -> ())>,
    }
}

impl ComponentSubscription for Sum where Double: Component, Square: Component {
    type Deps = (Double, (Square, ()));                 // nested list, see below
    type Input<'a> = __Sum_input::SumInput<'a>;
    fn input<'a>(raw: <Self::Deps as DepList>::Inputs<'a>) -> Self::Input<'a> {
        let (__dep0, (__dep1, ())) = raw;
        __Sum_input::SumInput { doubled: __dep0, squared: __dep1, __marker: PhantomData }
    }
}
```

**Why a nested list.** Rust has no variadic generics, so no single impl can cover flat tuples of every length. With a nested list, two impls of `DepList` cover every length: one for `()`, and one for `(H, T)`, which handles the first element and recurses on the rest. Every operation (input types, the plan's walk, the storage, the run, the lookups) has this same recursive shape. User code never writes one: the derive builds the list from `#[subscribe]`, and the pipeline's own roots list is `(App, ())`.

**Why the input struct is in a hidden module.** `SumInput` appears in `Sum`'s public trait impl, so a private struct is a hard error (E0446). Rust 1.74 turned several of these private-in-public rules into warnings, but associated types in trait impls are still errors. Private fields fail as soon as the `Component` impl is written in a sibling module (E0616). Putting a `pub` struct with `pub` fields inside a private module passes both checks, and still leaves the struct unnameable from outside (E0412 if you try).

**Why `input.doubled` works anyway.** Inside `impl Component for Sum`, `Self::Input<'_>` is a projection on a concrete type, which the compiler resolves through the generated impl to `__Sum_input::SumInput<'_>`. Field access then only checks the *field's* visibility, which is `pub`. It never checks whether the struct's name is reachable. In code that's generic over `C: Component`, `C::Input` stays opaque and has no fields, but only the component's own `update` ever receives one.

The `PhantomData` marker keeps type parameters "used" when no subscription mentions them (otherwise E0392). The generated module is named `__<Component>_input`, keeping the component's capitalization to avoid collisions such as `Sum` and `SUM`.

### 2. Type checking (`cargo check`)

Library code is checked once, generically. In the user's crate, the compiler does three things. First, it resolves `Self::Input<'_>`, so field typos and wrong field types are ordinary type errors. Second, it checks the derive's `where T: Component` bounds, so a subscription to a non-component is rejected. Third, at `Pipeline::<App>::build` it needs `App: Root`, which holds for every component whose storage and walk exist; naming a type that isn't a component is rejected there.

**Cycles.** The plan's walk needs the trait bound `A::Deps: const Upstream`, which holds for a dependency list only if it holds for the dependency list of every component in it. The trait solver proves this when `Pipeline::<A>` is first used. On a cycle the proof needs itself and fails, as E0275 "overflow evaluating the requirement `(B, ()): Upstream`", which names a component of the cycle. This is always on and costs nothing extra: it is the same proof the plan needs anyway. (Before step 29 below a cycle surfaced as E0391 on the plan's constants, which names no component, and the readable form cost 30% of the build and was an opt-in cargo feature. The feature `cycle-diagnostics` still exists but does nothing.) Two consequences of proving it by recursion: the proof is as deep as the graph (about six solver levels per layer), so graphs deeper than roughly 15 layers of four dependencies press on the compiler's default recursion limit of 128, which makes the compile slower, and much deeper graphs need `#![recursion_limit = "256"]` in the user's crate (see [Performance](#performance-and-compile-time)); and a component may list many subscriptions (100 were tried) because `Upstream` takes them eight at a time.

### 3. Planning during compilation (constant evaluation, `cargo build`)

The run needs one kind of number: for each edge, whether this is where the walk first reaches the upstream component. It is computed by constant evaluation, by running the depth-first walk itself once, inside a `const fn` (`dataflow-flat/src/plan.rs`).

**Identity.** `Described::ID = fnv1a(type_name::<C>())`. This is what requires nightly (`const_type_name`). Comparing `TypeId`s isn't reliably available in constant evaluation, while `const_type_name` has been stable on nightly for years. Type names aren't guaranteed unique and hashes can collide, so `Pipeline::build` compares real `TypeId`s and panics on a collision. IDs 0 (`ROOT`) and 1 are reserved.

**The walk.** `Planned::PLAN` (one per root) is `Plan::of::<A>()`, which calls `visit::<A>(&mut plan, ROOT)`. Visiting a component `C` from parent `P`:

1. look `C` up by id. If it was reached before, do nothing (if its entry is still open, the walk is inside it: that is a cycle, which panics; in practice the trait solver finds cycles first, see below);
2. otherwise append `C` with parent `P`;
3. call `<C::Deps as Upstream>::visit_all`, which visits each dependency in listing order;
4. record where `C`'s range ends.

Step 3 is a call to a trait method on the dependency-list type from inside a `const fn`, which is what `const_trait_impl` is for: without it a `const fn` cannot follow the types. `Upstream` is a `const trait` implemented for `()` and for dependency lists in chunks of up to eight entries (see the cycle paragraph for why chunks). The walk costs one hash probe per edge (about 1,900 visits for the 397-component benchmark). The previous formulation computed a walk-order constant per component and merged them, which is quadratic (237,000 entry visits at the same size) and was 60% of constant-evaluation time.

Because the walk is depth-first, the entries come out in first-reached order, which is a pre-order of the first-parent tree: **the storage subtree of a component is a contiguous range of the plan**. `Plan::holds(top, id)` ("does the subtree of `top` hold `id`?") is therefore two index lookups and a range test.

Real values for the example graph, which `cargo test` asserts (`plan_trace`):

```
plan, in first-reached order, with the parent each was first reached from:
  App ←ROOT   Report ←App   TopWords ←Report   WordCount ←TopWords
  Tokenize ←WordCount   Source ←Tokenize   Longest ←Report   Len<Tokenize> ←App

edges that are NOT first visits: Report→Source, Longest→Tokenize, Len<Tokenize>→Tokenize
```

**Per-edge constants.** `Edge::<R, P, H>::FIRST = (parent H was first reached from == P)`, for root `R`. An edge is identified by its (parent, child) types, so a component listing the same dependency twice would make both edges `FIRST` and store (and run) the component twice. `Upstream::visit_all` rejects that with a panic during constant evaluation, which surfaces as compile error E0080. The derive dedupes subscriptions with identical spellings first, so this only triggers through things like type aliases.

**Every constant is a plain value with no references.** This is load-bearing; see [history](#design-history-and-rejected-alternatives). (`Planned::PLAN` is a `&'static Plan` to a single plain value, so using it does not copy 16 KB.)

### 4. The states tree (types)

Constants can select types if the crate enables `generic_const_exprs` (GCE). The library does, and **user crates need no feature gate**: the constants only appear in the library's own impls, whose bounds the user's crate proves without spelling them. The storage of a component is:

```rust
struct Tree<R, C: Placed<R>> {
    deps: <C::Deps as DepStorage<R, C>>::S,   // one entry per subscription
    node: C,
}
// DepStorage for (H, T):   S = (<If<{ Edge::<R, C, H>::FIRST }> as Slot<R, H>>::Out, T::S)
//   If<true>::Out  = Tree<R, H>      the dependency's whole subtree, stored here
//   If<false>::Out = ()              stored elsewhere, nothing here
```

`C: Placed<R>` is the one-line spelling of "`C::Deps: DepStorage<R, C>`" (a supertrait with an associated-type bound; it holds for every component of a graph that compiles). So every component is stored exactly once, in the subtree of the edge that first reaches it. For `App` that is:

```
Tree<App> {
  deps: ( Tree<Report> { deps: ( Tree<TopWords> { deps: ( Tree<WordCount> { deps: ( Tree<Tokenize> { deps: (Tree<Source>, ()), .. }, () ), .. }, () ), .. },
                                 ( Tree<Longest>  { deps: ((), ()), .. },        // Tokenize is stored under WordCount
                                 ( (),                                           // Source is stored under Tokenize
                                   () )) ), .. },
          ( Tree<Len<Tokenize>> { deps: ((), ()), .. }, () ) ),
  node: App }
```

**Construction is safe code.** `Tree::make` builds the dependencies, takes the config, calls `C::new`, and returns the struct. The `deps` field comes first, so construction order is topological, and the same recursion fills in `Pipeline::order()`. A panicking constructor unwinds through ordinary drops: what was built is dropped exactly once. **Dropping is compiler drop glue**: there is no destroy walk, no allocation and no manual layout. Over-aligned components are fine because they are ordinary fields.

### 5. The fold (`run`)

`run` walks the states tree structurally. Walking `Tree<R, C>` does this:

```rust
let deps = self.deps.walk(ctx);                 // upstream subtrees first, each returns `Done` by value
let out  = self.node.update(C::input(gather((&deps, ctx))));   // &mut field reborrow; inputs are & references
Done { deps, out }                              // this subtree's outputs, by value
```

`Done<R, C> { deps, out }` holds a finished subtree's outputs. A later sibling needs to read outputs of earlier ones (`Longest` reads `Tokenize`, which finished inside `TopWords`'s subtree). The walk therefore passes a *context* (`Context`): a type-level list `(Done<TopWords>, (Done<Tokenize>, ..))` whose runtime value is a tuple of shared references to finished subtrees that live in the enclosing stack frames. After walking `a`, the walk of the later siblings gets `(&a, ctx)`. The component being walked adds one entry of its own, `Pending<R, C>`, whose value is a reference to its finished dependency tuple, so its inputs are looked up in `(Pending, ctx)`.

**Lookups are monomorphized, not searched.** An input is found by `Find<R, H, HERE>`. At each context entry the constant `TopHolds<R, L, H>` says whether that entry's subtree holds `H` (via `Plan::holds`), and the impl for that constant is chosen; inside the entry `Get<R, H, HERE>` descends the same way until `IsSame<C, H>` says it has arrived. The branch is a const parameter of the trait, so each step is one monomorphized function, and no step exists for paths not taken. After inlining, an input is a field access at a known path. Finding `Tokenize` for `Longest`: the first context entry is `Done<TopWords>`, whose first-parent chain contains `Tokenize` (`Tokenize ← WordCount ← TopWords`), so go in; its `deps` head is `WordCount`, which also holds it; its head is `Tokenize` itself.

**Why it is safe.** Everything is ordinary borrowing. `update` gets a `&mut` reborrow of the component field of the subtree being walked and shared references to `Done` values that are not being mutated (they are locals of the enclosing frames and are only borrowed, never moved, until the walk returns). Inputs can't escape because components and outputs are `'static` and `Input<'a>` is only handed to `update`.

**Why it is fast.** Every function in the walk is `#[inline]`, and the lookup chain is a chain of monomorphic one-liners. In the release build of the diamond, `run_once` compiles to nine instructions with no calls and no branches, and the benchmark runs at the speed of the hand-written struct (see [Performance](#performance-and-compile-time)). There is no `Option` per output and no tag store, because outputs only exist as locals of the fold.

At the end, `Root::run` drops everything except the root: the finished outputs tree is destructured and only the root's `out` is returned (`Done::into_output`). The root is walked as `Tree<A, A>`: the plan is for root `A`, and the type parameter `R` of every plan-dependent type is the root component itself.

### 6. Startup (`Pipeline::build`, the only runtime lookups)

The config pass walks the graph and takes each config from `Configs` by `TypeId`, or uses `C::Config::default()`. A config can therefore never be missing. It collects all unused configs (supplied for components outside the graph) into a single `BuildError`, and checks for identity collisions. Then `Root::make` runs the construction described above. The `TypeId` maps exist only here.

## Guarantees and where each error shows up

| Mistake | Caught by | Error | Checked in |
|---|---|---|---|
| Typo in `input.field` | `cargo check` | E0609, with a "similar name" suggestion | `check_errors.sh field_typo` |
| Wrong type for an input | `cargo check` | E0308 | `wrong_input_type` |
| Subscribing to a non-component | `cargo check` | E0277 (also prints a stray "see issue #48214") | `not_a_component` |
| `Pipeline::<X>` where `X` is not a component | `cargo check` | E0277 "`X: Root` is not satisfied" | `root_not_a_component` |
| Generic code over `Pipeline<A>` with `A: Component` but no `Root` bound | `cargo check` | E0277 "`A` can't be the root of a `Pipeline`" (a custom `on_unimplemented` message that says to add `A: Root`) | `generic_missing_root_bound` |
| Generic code with the `Root` bound | compiles, no feature gate | | `generic_root_bound_ok` |
| Two subscriptions with the same name | `cargo check` | `duplicate subscription name` | `duplicate_name` |
| Dependency cycle | `cargo check` | E0275 "overflow evaluating the requirement `(B, ()): Upstream`", naming a component of the cycle | `cycle`, `self_cycle` |
| Keeping an input reference past `update` | `cargo check` | lifetime error | `keep_input_reference` |
| Naming the generated input struct | `cargo check` | E0412 | `name_input_struct_outside` |
| Same dependency twice through an alias | `cargo build` only (detected after monomorphization) | E0080 "lists the same dependency twice" | `duplicate_dep_via_alias` |
| More than `MAX_COMPONENTS` | `cargo build` only | E0080 "too many components: raise dataflow_flat::MAX_COMPONENTS" | `too_many_components` |
| Config supplied for a component not in the graph | `Pipeline::build` at startup | `BuildError { unused }`, all at once | `unused_config` |
| Identity collision | `Pipeline::build` | panic | |

A root listed twice used to be a possible mistake; with a single root component it cannot be written. Two subscriptions of the root to the same thing are covered by `duplicate_dep_via_alias`.

Runtime guarantees, checked by `cargo test` in `dataflow-flat/tests/`:

- Each component runs once per run, in the documented depth-first post-order, even with several edges into it (`order.rs`: the exact order on the example graph, and graphs where a later-listed dependency is upstream of an earlier one).
- `run` returns the root's output by value (`runtime.rs`); outputs of non-root components are dropped by the end of each run, and none exist before the first run (`runtime.rs`, counted with a `Drop` impl).
- Unused configs are reported together; a component with no entry in `Configs` gets `Config::default()`.
- A shared component is constructed once, updated once per run and dropped once (`graphs.rs`, `runtime.rs`); the same component under two names gives one dependency and the same reference.
- A pipeline is `Send` when its components are, can be moved and boxed between runs, and runs on another thread (`graphs.rs`).
- A constructor that panics during `build` drops what was already built, each exactly once (`graphs.rs`).
- Over-aligned components stay aligned (`graphs.rs`).
- Generic code over the root needs one bound and compiles without a feature gate (`generic_user.rs`, `check_errors.sh generic_root_bound_ok`).
- The framework contains no `unsafe` (`#![forbid(unsafe_code)]`).

The plan assertions above are in the `plan_trace` unit test.

## Performance and compile time

Measured on the generated 396-component, 1,504-edge layered graph (20 layers × 20 components, 4 random subscriptions each), with trivial `update` bodies, so these numbers isolate framework overhead. There is one more component, the root `App` (397 in total), which XORs the 20 last-layer outputs, and the hand-written baseline computes the same checksum (`assert_eq!`, "results agree"). 24-core Linux box, pinned `nightly-2026-10-30`. The master column is the previous raw-pointer runtime at commit `586adb1`, measured the same way on the same machine in an earlier session. "Before" is the safe fold at commit `f5e47dc`, re-measured in the session that produced "now" with the protocol below; "now" is the current tree.

Protocol (`bench` crate, `python3 bench/gen_bench.py full 20`): `cargo clean -p bench -p dataflow-flat -p dataflow-macros`, then `CARGO_INCREMENTAL=1 cargo build -p bench` (debug scratch, includes the macro crate but not `syn`/`quote`); edit the first generated `update` body and rebuild (edit 1, then edit 2 with another change); `cargo clean --release ...` and `cargo build --release -p bench` (release scratch); run the release binary three times. The first rebuild after a scratch build recompiles almost everything (the incremental cache from a scratch build is not yet complete: 2.9 s now, 4.1 s before); the second and later rebuilds are the steady state (0.7 s).

| | master (raw pointers) | before (`f5e47dc`) | now | now, `#![recursion_limit = "256"]` in the user crate | hand-written struct |
|---|---|---|---|---|---|
| Time per run, release | 262–268 ns | 141–145 ns | 143–147 ns | same | 161–167 ns alone; 211–216 ns in the bench binary (code layout, see below) |
| Time per run, debug | 59 µs | 157 µs | 136 µs | same | 2.6–2.8 µs |
| Debug build from scratch, incremental | 5.5 s | 5.9 s (6.8 s earlier) | **3.8 s** | 2.8 s | |
| Rebuild after editing an `update` body: first / later | 0.58 s | 4.1 s / 0.6–0.7 s | 2.9 s / 0.7 s | 1.9 s / 0.7 s | |
| Release build from scratch | 6.8 s | 7.2 s (8.3 s earlier) | **5.3 s** | 4.4 s | |
| Debug build from scratch with `cycle-diagnostics` | | 9.0 s | (feature removed, same as above) | | |

Timings vary by about ±2% between runs; the "earlier" values are what the previous session measured on a busier machine. These are much lower than the numbers measured with rustc 1.75 (21 s debug, 30 s release), which were dominated by constant evaluation that the current compiler does faster.

**Growth with graph size** (same protocol, `gen_bench.py full L`; compile times in seconds as debug scratch / edit 1 / edit 2 / release scratch; run time is the flat runtime, release, ns per run). No exponential behavior in either version; `MAX_COMPONENTS` is 512, so L=26 and above does not fit this generator.

| L (components) | before: scratch / edit 1 / edit 2 / release | ns/run | now: scratch / edit 1 / edit 2 / release | ns/run | now with `recursion_limit = 256` |
|---|---|---|---|---|---|
| 5 (101) | 1.1 / 0.5 / 0.1 / 1.3 | 36–37 | 0.8 / 0.4 / 0.1 / 1.1 | 35–38 | |
| 10 (199) | 2.2 / 1.3 / 0.3 / 2.8 | 71–72 | 1.3 / 0.8 / 0.3 / 2.1 | 70–75 | |
| 20 (397) | 5.8 / 4.1 / 0.6 / 7.2 | 142–146 | 3.8 / 2.9 / 0.7 / 5.3 | 143–146 | 2.8 / 1.9 / 0.7 / 4.4 |
| 25 (497) | 8.5 / 6.2 / 0.9 / 10.2 | 291–301 | 6.9 / 5.9 / 1.0 / 8.8 | 289–298 | 3.6 / 2.5 / 0.9 / 5.7 |

The run time at L=25 is slower than at L=20 by more than the component count (flat 295 ns vs hand-written 270 ns): the same in both versions, so not a regression; it was not investigated. With the default recursion limit the walk-based plan loses part of its advantage as graphs get deeper: the proof it needs (see [Cycles](#2-type-checking-cargo-check)) approaches the limit of 128 around L=17 and the solver slows down. For the framework-only (`flat`) variant of the benchmark at L=20, the walk-based plan builds in 3.5 s at the default limit and 2.6 s with `recursion_limit = 256`; the per-component-constant plan it replaced built in 3.5 s at either limit.

**Where the compile time goes** (`-Zself-profile` with `summarize`, debug, non-incremental, `flat`-only L=20 benchmark, so the totals differ from the table above; the split between non-constant phases is approximate). Before: 5.6 s, of which constant evaluation was 2.9 s (the per-component walk-order constants `SEQ` 2.0 s, the lists of references to them 0.5 s, the per-lookup `HeadHas` constants 0.35 s); the rest was type checking and trait solving about 1 s, monomorphization collection 0.7 s, LLVM and linking about 1.5 s. Now: 3.5 s, constant evaluation 0.2 s, trait solving about 0.6 s (0.4 s more than before: the `const Upstream` proof), monomorphization collection 0.7 s, LLVM and linking about 1.5 s, expanding the 401 derive invocations 0.15 s. About 19,300 functions are instantiated, 40% of them lookup steps (`Find` and `Get`), 12% `Walk`, 10% `DepStorage::make`, 10% `GatherInputs`, 7% `collect_configs`; reducing those is the remaining lever (see [next steps](#suggested-next-steps)).

**Hand-written baseline layout.** The hand-written struct measures 161–167 ns alone and 211–216 ns inside the bench binary that also holds the flat runtime. That is function alignment, not the framework: with `RUSTFLAGS="-Zmin-function-alignment=64"` (or `-C llvm-args=-align-all-functions=6`) the same binary measures hand-written 161–165 ns and flat 143–147 ns, i.e. the flat runtime is about 10% faster than the hand-written struct when layout is neutral.

The flat runtime is faster at run time in release than the old raw-pointer one, most likely because there is no per-output `Option` tag store (outputs only exist as locals of the fold, which LLVM keeps in registers). Debug builds do not inline, so every lookup step is a real call and `run` is slower than the raw-pointer runtime in debug (136 µs for 397 components; it was 157 µs before the `Pending` context entry replaced the push of every dependency onto the context). Editing an `update` body re-runs neither the plan nor the types.

## Design history and rejected alternatives

Read this before trying something "simpler"; most simpler ideas were tried. Rows 16 to 28 use the names of their time: `Branch` is now `Slot`, `StorageList` is `DepStorage`, `GetB`/`FindB` are `Get`/`Find` with a `HERE` parameter, `HeadHas` is `TopHolds`, `Refs` is `Context`, and `PushAll` no longer exists (row 31).

| Step | What | Outcome |
|---|---|---|
| 1 | Runtime registry: `HashMap<TypeId, Rc<dyn Any>>`, components pull dependencies in `build` | Works, but every check happens at run time. Rejected by R2. |
| 2 | `container!` macro listing every concrete type, `Provides<T>` impls, lazy `OnceCell`s | Compile-time checked, but someone has to list every component. Rejected by R3. |
| 3 | Derive on `Dep<T>` fields + `TypeId` registry; only roots named | Worked with hundreds of components, but it was a dependency-injection model (components hold handles to each other) rather than dataflow. Superseded by R4. |
| 4 | Dataflow with `Vec<Box<dyn Node>>` + `Box<dyn Any>` output slots | Correct, but uses dynamic dispatch and downcasts. Rejected by R6. |
| 5 | Typed `Rc<Node<C>>` handles + per-run epoch check on each edge | No dynamic dispatch and works on stable, but about 10× slower than a hand-written struct on trivial components (~4,700 ns per run on the benchmark graph). Removed once the flat runtime superseded it. |
| 6 | Specialization for type-level deduplication (insert into a type-level set) | **Doesn't work**: specialized (`default`) associated types stay opaque during type checking, even for concrete types. Verified. |
| 7 | `generic_const_exprs` / const traits | Avoided at first: incomplete or with frequently changing syntax. Revisited in step 16, where it is what makes the safe design possible. |
| 8 | Compile-time plan from a graph of `&'static` description constants | Correct, but **incremental builds were exponential** (L=10: 67 s, L=20: >100 s). Incremental compilation fingerprints a constant by following its references, and it doesn't remember shared nodes, so the 4-way-branching graph was hashed as a tree, about 4^depth. Shrinking the plan arrays didn't help; profiling showed the cost in the stage that evaluates constants. |
| 9 | Per-component flat walk-order arrays merged at every partial list (A,B,C → B,C → C) + linear plan lookups | Fixed the exponential growth (L=20: 26 s), but did redundant merges. |
| 10 | One-pass merge per component + hashed plan index | Was current for the plan until step 33. In the raw-pointer runtime: one buffer, constant offsets, a walk of `if FIRST { eval_node }`, outputs stored as `Option<Output>`. The runtime half was replaced in step 17. |
| 11 | `#[inline(always)]` on the walk | **Exponential in debug builds**: forced inlining at `-O0` happens before the dead `if false` branches are removed, so it inlined every path through the graph. Use `#[inline]`. Release builds were unaffected. Also tried on the lookup chain of the safe fold: no gain. |
| 12 | Tuple syntax: `type Deps = Deps![A, B]`, `fn call(&mut self, deps!(a, b): Inputs<'_, Self>)` | Worked, but felt awkward. Replaced by the derive (R8). Still used for roots until step 25. |
| 13 | `#[derive(ComponentSubscription({ sub1: X }))]` | Not valid Rust: derives can't take arguments. Became derive + `#[subscribe(...)]`. |
| 14 | Generated `SumInput` as a sibling of `Sum` | Must be `pub` (E0446) with `pub` fields (E0616 for sibling-module impls). Moved into a hidden private module so it can't be named from outside. |
| 15 | **Variant A**: inline byte storage with GCE | Worked, but it forced `#![feature(generic_const_exprs)]` into every user crate, and it kept all the `unsafe` walks. Rejected. |
| 16 | **Variant B**: typed storage tree, `Tree<R, C>`, with each dependency stored as `<If<{ Edge::FIRST }> as Branch>::Out` | Every component stored once; construction is safe; drop is compiler drop glue; the GCE gate stays inside the library (users need one nameable bound). Kept as the base of what shipped. Outputs still needed storing and reading back (next rows). |
| 17 | **fold3**: `run` is a fold over the tree; each subtree returns its outputs by value (`Done`); later siblings read earlier ones through a context of `&` references; inputs found by monomorphized `FindB`/`GetB` steps chosen by `HeadHas`/`Same` constants | **Current.** `#![forbid(unsafe_code)]`, no `Option` per output, release speed of the hand-written struct. |
| 18 | "split": a states tree plus a second tree of `Option<Output>` read through typed paths | Safe, but 79 surviving `Some` checks, ~270 ns per run, and not fully inlined. Rejected. |
| 19 | fold / fold2: the same idea as fold3 with the context type spelled out in symbols | 60 MB of symbol names, slower builds. fold3 makes `Done` nominal, so names stay small. |
| 20 | fold4: a continuation instead of the `PushAll` context | No gain. |
| 21 | frunk-style `Selector<T, I>` with an inferred index | Blocked: E0207 (unconstrained type parameter) in generic impls, and E0283 (ambiguity) when two outputs share a type. |
| 22 | `Cell` / `OnceCell` / `RefCell` for outputs | The same `Option` check, or borrow flags. |
| 23 | `core::field::field_of!` / `core::mem::type_info` to find outputs | On this nightly `Field` only gives an offset through an unsafe trait, and `type_info` is value-level: it can't turn a `TypeId` into a type. |
| 24 | Roots as a tuple list, `Deps![...]` / `deps!(..)`, with `outputs()` returning the root outputs | Worked; a named-roots derive (`#[derive(Roots)]`) was the planned improvement. |
| 25 | **Single root component**: the application defines one ordinary component (`App`); `Pipeline::<App>` builds it, `run` returns its output by value; internally the roots list is `(App, ())` | **Current.** Removes `Deps!`/`deps!`, `outputs()`, the `ran` flag and stored outputs, and makes named roots unnecessary: the root's output type is any struct you like. |
| 26 | Cycle check as a bound on `Root`/`Pipeline`/`build` | Does not help: the trait solver evaluates the plan's constants before (or while) it proves the bound, and the cycle in the constants is reported as E0391 first. |
| 27 | Cycle check as a bound on `Planned` | Gives a single clean E0275, but the whole-graph proof is re-run at every use of `Planned`: +7 s (debug scratch 13.7 s). Rejected. |
| 28 | Cycle check as a supertrait of every `Component` | Gives E0275 at each component in the cycle, +2.1 s (+30%) debug scratch. Was kept as the opt-in feature `cycle-diagnostics`; removed in step 33 (the feature is a no-op now). |
| 29 | `Seq` as two integer arrays (`ids`, `parents`) instead of an array of `Entry` structs | **Kept** (then superseded by 33). The compiler validates a constant's value field by field, which is slow for 512 structs: evaluating the lists of references to `SEQ` (`HEADS`) went from 0.49 s to 0.01 s at L=20. Debug scratch (flat-only) 5.6 s -> 5.37 s. |
| 30 | Cheaper constant-evaluation loops: probe the hash set inline instead of through `IdSet` methods (SEQ 2.0 s -> 1.2 s), one probe read and a local length counter and the dependency's own entry handled before the loop (-> 0.84 s); `PLAN` as `&'static Plan` so uses do not copy 16 KB; `holds` as a range test on first-reached order instead of walking the first-parent chain (`HeadHas` evaluations 0.35 s -> 0.09 s) | **Kept** (SEQ part superseded by 33). Debug scratch (flat-only) 5.37 s -> 3.85 s. Measured: per-visit cost is interpreter steps, not array size (MAX_COMPONENTS 1024 changed nothing), so every statement removed from the inner loop counts. |
| 31 | Look inputs up in `(Pending<R, C>, ctx)`: the component being walked adds one context entry that refers to its finished dependency tuple, instead of pushing every dependency onto the context (`PushAll`) | **Kept.** 1,921 of 20.8k monomorphized functions removed; debug scratch 3.85 s -> 3.71 s; debug run 157 µs -> 136 µs; release unchanged. |
| 32 | Split `lib.rs` into modules; clearer names (`DepStorage`, `Slot`, `Top`/`TopHolds`/`IsSame`, `Get`, `Context`, `Find`, `GatherInputs`); `Placed<R>` as a one-line bound for `C::Deps: DepStorage<R, C>`; the plan and every plan-dependent type are keyed by the root component itself (no `(A, ())` roots list, no `Stored`, `IntoRoot`, `RootParent`) | **Kept.** 3.71 s -> 3.5 s (flat-only), release unchanged. `Walk` carries an `on_unimplemented` message so a non-component root reads "can't be the root of a `Pipeline`". |
| 33 | **Plan by running the depth-first walk in constant evaluation** (`const_trait_impl`): a `const fn visit` that calls `Upstream::visit_all` (a `const trait` implemented for dependency lists) records components in first-reached order; no per-component `SEQ` | **Kept, current.** Constant evaluation 1.4 s -> 0.2 s at L=20; debug scratch 5.9 s -> 3.8 s over the session's start (bench `full`), release scratch 7.2 s -> 5.3 s. A dependency cycle is now found by the trait solver proving the walk's bounds and reported by default as E0275 naming a component (replaces rows 26-28's attempts at no extra cost). Costs: a new unstable feature whose syntax changed within the last year (`const impl`); the proof is as deep as the graph, which with the default recursion limit slows builds beyond about 17 layers of 4 dependencies and needs `#![recursion_limit]` for deeper ones. |
| 34 | Variants of 33: `Visit` as a trait on components and `Upstream` on lists (three solver levels per layer); `Upstream` peeling one list entry per impl; chunks of 2, 4, 8 entries | `Visit` + chunk 8 built the L=20 benchmark (flat-only) in 4.6 s at the default recursion limit, 2.5 s at 512; chunk 2: 6.1 s; chunk 4: 4.7 s; one entry per impl: fails at L=20 with the default limit (E0275). Replaced by the current shape (a free `const fn visit` bounded by `C::Deps: const Upstream`, so two levels per layer): 3.5 s default, 2.6 s with 256. |
| 35 | Cheaper cycle proof, keeping the proof per component (`H::Deps: NoDependencyCycles` directly instead of through `NoDependencyCycle`) | No gain (6.0 s -> 5.9 s). Diagnosis: the opt-in check cost nothing at 100 components, +0.4 s at 200 and +2.1 s at 400, and `check_well_formed` was where the time went: every `impl Component` re-proves its whole upstream graph in a fresh solver context. Superseded by 33. |
| 36 | Function pointers or a table of `fn`s in constants to call per-type visitors without const traits | **Does not compile**: "function pointer calls are not allowed in constant functions" on this nightly. |
| 37 | Derive-generated inherent `const fn __visit` per component (no const traits, no trait solving, no recursion limit) | Not built. A component that subscribes to its own type parameter (`Len<U>`) would have to call a method on `U`, which needs a `[const]` trait bound written in the user's crate and so a feature gate there. |
| 38 | `MAX_COMPONENTS` 512 -> 1024 (while the plan was per-component `SEQ`) | Debug scratch 5.6 s -> 6.0 s, not kept; array size is a small part of the cost. |

Smaller decisions:

- **Configs** are keyed by component type.
- **`type Config: Default`** lets components go without config entries. It replaced an optional `default_config()` method, so a missing config can no longer happen; a config with no sensible default needs a placeholder `Default` impl (or a `new` that validates).
- **Inputs are shared references**, so any number of subscribers share one output without cloning.
- **Subscriptions with identical spellings** share one dependency instead of erroring.
- **`MAX_COMPONENTS = 512`** (it was documented as 1024 earlier; the code says 512).
- **The order guarantee** (depth-first post-order in listing order) is the order the fold naturally produces; it is documented and tested so that it stays a guarantee.

## Known limitations and rough edges

| Area | Detail |
|---|---|
| Nightly | Needs `const_type_name`, `const_trait_impl` (the plan's walk; its syntax changed from `impl const` to `const impl` recently, so a different nightly may need edits in `plan.rs`) and `generic_const_exprs`. The last is incomplete: this nightly prints "`feature(generic_const_exprs)` is not supported with the next-generation trait solver ... reverted the setting to `-Znext-solver=coherence` for this crate" and an `incomplete_features` allowance is in the library. Only the library crate is affected, but the warning shows in every build of it. If the compiler stops supporting GCE, the typed storage tree has no replacement yet. |
| Identity | Hash of `type_name`. Collisions and name ambiguity are caught at startup by comparing `TypeId`s, not at compile time. |
| Capacity | `MAX_COMPONENTS = 512` (a fixed-size plan array; exceeding it is E0080 "too many components", checked by `check_errors.sh too_many_components`). A component can have 100 or more subscriptions. |
| Graph depth | Several recursive compile-time proofs (storage, walk, the plan's `Upstream` bound) are as deep as the graph. A chain of about 43 components overflows the default recursion limit ("queries overflow the depth limit"), and wide layered graphs of 17 or more layers of 4 dependencies get slower to compile well before that. Add `#![recursion_limit = "256"]` to the user crate for deep graphs; it also makes large graphs compile faster (see [Performance](#performance-and-compile-time)). This limit existed before the walk-based plan (the chain limit is unchanged), but the plan's proof moved the wide-graph threshold down. |
| Compile time | 3.8 s from scratch at 397 components (bench `full`), of which roughly 2 s is monomorphizing and compiling about 19,000 small generic functions (lookup steps are 40% of them); see [Performance](#performance-and-compile-time). |
| Run time in debug | About 2.3× slower than the old raw-pointer runtime (136 µs vs 59 µs per run at 397 components): every lookup step is a real call without inlining. |
| Error quality | A cycle is E0275 "overflow evaluating the requirement `(B, ()): Upstream`" pointing at library code; it names a component of the cycle but not the whole cycle. Subscribing to a non-component adds "see issue #48214" noise. Duplicate-through-alias and too-many-components only appear with `cargo build`, and the first also prints a stray E0284. |
| Inputs | Shared references only. A component can't take ownership of an upstream output. |
| Outputs | Intermediate outputs live only during `run` and are dropped before it returns; only the root's output survives (by value). Outputs are built as locals of the fold, so large inline outputs live on the stack during `run`; box big outputs. |
| Threads | Single-threaded. A `Pipeline` is `Send` when its components are. |
| Per-run input | None: sources produce data from their own state. |
| Macro | Hard-codes `::dataflow_flat` paths (the crate uses `extern crate self as dataflow_flat;` so this works inside it too). A user item named `__<Component>_input` would collide. |
| `order()` | Returns `type_name` strings. Fine for debugging, not stable identifiers. |

## Suggested next steps

1. **Per-run external input.** For example, `run(&mut self, input: &I)` with source components reading `I`.
2. **Reduce compile time further.** The plan is now cheap; what is left is monomorphization and code generation of about 19,000 tiny functions. Ideas, not tried: store only first-edge subtrees in the states tuple (no `()` placeholders for the roughly 1,100 non-first edges at L=20, about 2,000 fewer instances, but the stored list would then depend on a const at every use), a single construction-and-config pass (changes when unused configs are reported: after constructors have run), and a shallower proof for the plan's `Upstream` bound so that the recursion-limit cliff moves out.
3. **Remove GCE if a stable mechanism appears** (const traits, or a stable way to turn a constant into a type choice), which would also remove the next-solver warning.
4. **Parallel execution.** Components at the same depth are independent, but this needs `Send`/`Sync` bounds and a scheduler that doesn't reintroduce dynamic dispatch. The fold's structure (a tree of finished subtrees) suggests joining sibling subtrees.
5. **Owned inputs.** For example, move an output when it has exactly one subscriber, or add an explicit "consume" subscription kind. The by-value `Done` tree makes this more natural than before.
6. **Cheaper debug runs.** For example, collapsing the lookup chain for the common case of a direct dependency (the `Pending` entry already removed the per-dependency push).

(Done since the last version of this list: named roots, replaced by the single root component; removing the `Option` tag stores; a cheaper plan and an always-on readable cycle error.)

## Inspecting compiler output

All of these were used during development; flag spellings may differ on newer nightlies. They run on the `diamond` crate, where the output is small enough to read.

```sh
cargo rustc -p diamond -- -Zunpretty=expanded                       # derive output
cargo rustc -p dataflow-flat --lib -- --emit=mir -o /tmp/flat.mir    # generic walk and lookups
cargo rustc -p diamond -- -Zprint-mono-items=yes                     # instances: one walk per Tree, one function per lookup step
cargo rustc -p diamond -- --emit=llvm-ir -o /tmp/d.ll
cargo rustc --release -p diamond -- --emit=asm \
    -C llvm-args=-x86-asm-syntax=intel -o /tmp/d.s                   # run_once: nine instructions, no calls, no branches
CARGO_INCREMENTAL=0 cargo rustc -p bench -- -Ztime-passes            # where compile time goes
# per-query self time: -Zself-profile=<dir> -Zself-profile-events=default,args, then `summarize summarize <dir>/*.mm_profdata`
# (cargo install --git https://github.com/rust-lang/measureme summarize crox)
```

Legacy symbol names hide generic arguments. To find a particular instance in the LLVM IR, match on the debug-info `DISubprogram(name: "walk<...>")` instead.

## Toolchain

The repo pins `nightly-2026-10-30` (rustc 1.101.0-nightly, 2026-10-29) in `rust-toolchain.toml`; bump that date to upgrade deliberately. Features in use, all confined to `dataflow-flat`: `const_type_name` (required), `const_trait_impl` (the plan's walk calls trait methods from a `const fn`; unstable but not marked incomplete, and the syntax is `const trait` / `const impl` / `[const]` / `const` bounds on this nightly) and `generic_const_exprs` (incomplete, see [limitations](#known-limitations-and-rough-edges)). Stable features in use (besides those below): associated type bounds in supertraits (`trait Placed<R>: Component<Deps: DepStorage<R, Self>>`), `#[diagnostic::on_unimplemented]`, generic associated types (`type Input<'a>`, `type Inputs<'a>`, `type Refs<'a>`), `extern crate self`, const panics. User crates need no `#![feature]`. On a different nightly, compiler-internal details described above may differ, such as how incremental compilation fingerprints constants or what the next trait solver does with GCE. The design doesn't depend on the former, but the compile-time numbers might.
