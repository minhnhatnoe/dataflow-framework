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
scripts/check_errors.sh                  # 15 cases: each mistake fails the documented way

python3 bench/gen_bench.py full 20       # regenerate the 396-component benchmark (+ root)
cargo run --release -p bench             # flat runtime vs hand-written
```

`rust-toolchain.toml` pins a nightly. Without rustup, set `RUSTC_BOOTSTRAP=1` to unlock nightly features on a compatible compiler.

**Wrap builds in a timeout** (`timeout 120 cargo ...`). Large graphs compile slowly (the release benchmark build takes about 8 s), and past bugs made builds exponential (see [history](#design-history-and-rejected-alternatives)). A timeout turns a regression into a failure instead of a hang.

## Workspace layout

| Crate | Toolchain | Role |
|---|---|---|
| `dataflow-flat` | nightly | **The runtime.** Compile-time plan, typed storage tree, safe fold. `#![forbid(unsafe_code)]`. |
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

The traits (in `dataflow-flat/src/lib.rs`):

```rust
pub trait ComponentSubscription: 'static {
    type Deps: DepList;                // type-level list of subscriptions; only the runtime uses it
    type Input<'a>;                    // generated struct with one named reference per subscription
    fn input<'a>(raw: <Self::Deps as DepList>::Inputs<'a>) -> Self::Input<'a>;
}

pub trait Component: ComponentSubscription + Sized {   // (+ a hidden always-true supertrait, see cycles)
    type Config: Default + 'static;
    type Output: 'static;
    fn new(config: Self::Config) -> Self;
    fn update(&mut self, input: Self::Input<'_>) -> Self::Output;
}
```

The five stages are: macro expansion, type checking, the compile-time **plan**, the **states tree** (types), and the **fold** that runs it.

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

**Why a nested list.** Rust has no variadic generics, so no single impl can cover flat tuples of every length. With a nested list, two impls of `DepList` cover every length: one for `()`, and one for `(H, T)`, which handles the first element and recurses on the rest. Every operation (input types, the plan, the storage, the walk, the lookups, the cycle check) has this same recursive shape. User code never writes one: the derive builds the list from `#[subscribe]`, and the pipeline's own roots list is `(App, ())`.

**Why the input struct is in a hidden module.** `SumInput` appears in `Sum`'s public trait impl, so a private struct is a hard error (E0446). Rust 1.74 turned several of these private-in-public rules into warnings, but associated types in trait impls are still errors. Private fields fail as soon as the `Component` impl is written in a sibling module (E0616). Putting a `pub` struct with `pub` fields inside a private module passes both checks, and still leaves the struct unnameable from outside (E0412 if you try).

**Why `input.doubled` works anyway.** Inside `impl Component for Sum`, `Self::Input<'_>` is a projection on a concrete type, which the compiler resolves through the generated impl to `__Sum_input::SumInput<'_>`. Field access then only checks the *field's* visibility, which is `pub`. It never checks whether the struct's name is reachable. In code that's generic over `C: Component`, `C::Input` stays opaque and has no fields, but only the component's own `update` ever receives one.

The `PhantomData` marker keeps type parameters "used" when no subscription mentions them (otherwise E0392). The generated module is named `__<Component>_input`, keeping the component's capitalization to avoid collisions such as `Sum` and `SUM`.

### 2. Type checking (`cargo check`)

Library code is checked once, generically. In the user's crate, the compiler does three things. First, it resolves `Self::Input<'_>`, so field typos and wrong field types are ordinary type errors. Second, it checks the derive's `where T: Component` bounds, so a subscription to a non-component is rejected. Third, at `Pipeline::<App>::build` it needs `App: Root`, which holds for every component whose storage and walk exist; naming a type that isn't a component is rejected there.

**Cycles.** A cycle is detected by the trait `NoDependencyCycle`, which holds for a component only if its dependency list is `NoDependencyCycles`, which holds for a list only if it holds for each component in it. On a cycle, the proof would need itself, so it fails, as "overflow evaluating the requirement `A: NoDependencyCycleCheck`" (E0275). By default nothing *requires* the proof, because requiring it of every component costs about 30% of the from-scratch debug build time, so a cycle surfaces later, as E0391 "cycle detected when simplifying constant ... `DepList>::HEADS`", from the plan's own constants. The cargo feature `dataflow-flat/cycle-diagnostics` turns the check on. See [history](#design-history-and-rejected-alternatives) for the attempts to get both.

### 3. Planning during compilation (constant evaluation, `cargo build`)

The run needs one kind of number: for each edge, whether this is where the walk first reaches the upstream component. It is computed by constant evaluation.

**Identity.** `Described::ID = fnv1a(type_name::<C>())`. This is what requires nightly (`const_type_name`). Comparing `TypeId`s isn't reliably available in constant evaluation, while `const_type_name` has been stable on nightly for years. Type names aren't guaranteed unique and hashes can collide, so `Pipeline::build` compares real `TypeId`s and panics on a collision. IDs 0 (`ROOT`) and 1 (`OWNER`) are reserved.

**Walk order per component.** `Described::SEQ` is a flat array listing the order in which a depth-first walk from the component first reaches each component upstream of it. Each entry is `(id, reached-from id)`. It's computed as one constant per component, from the constants of the components it subscribes to. This is necessary because a `const fn` can't call trait methods, so it can't walk types itself (const traits would allow that, but they're unstable and their syntax keeps changing). The compiler caches each constant, so this is computed once per concrete type: a bottom-up computation over the whole graph. The merge goes like this. Start with the component itself, with parent `OWNER`. For each subscription in order, append that subscription's walk order, skipping anything already reached, and replace `OWNER` with this component.

**Why the merge is correct.** The real walk visits subscriptions in the same order and enters a component only the first time it's reached. The set of reached components always includes everything upstream of its members, because the walk finishes a component's whole upstream before moving on, and the graph has no cycles. So skipping an already-reached entry also skips everything upstream of it, and the remaining entries keep their relative order and their parents.

Real values for the example graph, which `cargo test` asserts (`plan_trace`):

```
SEQ(Report) = [Report←OWNER, TopWords←Report, WordCount←TopWords,
               Tokenize←WordCount, Source←Tokenize, Longest←Report]
              (Longest's Tokenize and Source, and Report's own Source, are already reached)

plan, roots list (App, ()) merged with parent ROOT, in first-reached order:
  App ←ROOT   Report ←App   TopWords ←Report   WordCount ←TopWords
  Tokenize ←WordCount   Source ←Tokenize   Longest ←Report   Len<Tokenize> ←App

edges that are NOT first visits: Report→Source, Longest→Tokenize, Len<Tokenize>→Tokenize
```

**Plan and per-edge constants.** `Planned::PLAN` (one per roots list) holds each component's reached-from parent behind a hash index. The type-level code reads two kinds of constant. `Edge::<R, P, H>::FIRST = (parent H was first reached from == P)`. And `Plan::holds(top, id)`: is `top` on the first-parent chain of `id`, that is, does the storage subtree of `top` hold `id`? An edge is identified by its (parent, child) types, so a component listing the same dependency twice would make both edges `FIRST` and store (and run) the component twice. The merge rejects that with a panic during constant evaluation, which surfaces as compile error E0080. The derive dedupes subscriptions with identical spellings first, so this only triggers through things like type aliases.

**Every constant is a plain value with no nested references.** This is load-bearing; see [history](#design-history-and-rejected-alternatives). (The one exception is the one-level `SeqList` of already-computed `SEQ` constants.)

### 4. The states tree (types)

Constants can select types if the crate enables `generic_const_exprs` (GCE). The library does, and **user crates need no feature gate**: the constants only appear in the library's own impls, whose bounds the user's crate proves without spelling them. The storage of a component is:

```rust
struct Tree<R, C: Component> {
    deps: <C::Deps as StorageList<R, C>>::S,   // one entry per subscription
    node: C,
}
// StorageList for (H, T):   S = (<If<{ Edge::<R, P, H>::FIRST }> as Branch<R, H>>::Out, T::S)
//   If<true>::Out  = Tree<R, H>      the dependency's whole subtree, stored here
//   If<false>::Out = ()              stored elsewhere, nothing here
```

So every component is stored exactly once, in the subtree of the edge that first reaches it. For `App` that is:

```
Tree<App> {
  deps: ( Tree<Report> { deps: ( Tree<TopWords> { deps: ( Tree<WordCount> { deps: ( Tree<Tokenize> { deps: (Tree<Source>, ()), .. }, () ), .. }, () ), .. },
                                 ( Tree<Longest>  { deps: ((), ()), .. },        // Tokenize is stored under WordCount
                                 ( (),                                           // Source is stored under Tokenize
                                   () )) ), .. },
          ( Tree<Len<Tokenize>> { deps: ((), ()), .. }, () ) ),
  node: App }
```

**Construction is safe code.** `Branch::make` builds the dependencies, takes the config, calls `C::new`, and returns the struct. The `deps` field comes first, so construction order is topological, and the same recursion fills in `Pipeline::order()`. A panicking constructor unwinds through ordinary drops: what was built is dropped exactly once. **Dropping is compiler drop glue**: there is no destroy walk, no allocation and no manual layout. Over-aligned components are fine because they are ordinary fields.

### 5. The fold (`run`)

`run` walks the states tree structurally. Walking `Tree<R, C>` does this:

```rust
let deps = self.deps.walk(ctx);                 // upstream subtrees first, each returns `Done` by value
let out  = self.node.update(C::input(gather(deps, ctx)));   // &mut field reborrow; inputs are & references
Done { deps, out }                              // this subtree's outputs, by value
```

`Done<R, C> { deps, out }` holds a finished subtree's outputs. A later sibling needs to read outputs of earlier ones (`Longest` reads `Tokenize`, which finished inside `TopWords`'s subtree). The walk therefore passes a *context*: a type-level list `(Done<TopWords>, (Done<Tokenize>, ..))` whose runtime value is a tuple of shared references to finished subtrees that live in the enclosing stack frames. After walking `a`, the walk of the later siblings gets `(&a, ctx)`.

**Lookups are monomorphized, not searched.** An input is found by `FindB<R, H, B>`. At each context entry the constant `HeadHas<R, L, H>` says whether that entry's subtree holds `H` (via `Plan::holds`), and the impl for that constant is chosen; inside the entry `GetB<R, H, B>` descends the same way until `Same<C, H>` says it has arrived. The branch is a const parameter of the trait, so each step is one monomorphized function, and no step exists for paths not taken. After inlining, an input is a field access at a known path. Finding `Tokenize` for `Longest`: the first context entry is `Done<TopWords>`, whose first-parent chain contains `Tokenize` (`Tokenize ← WordCount ← TopWords`), so go in; its `deps` head is `WordCount`, which also holds it; its head is `Tokenize` itself.

**Why it is safe.** Everything is ordinary borrowing. `update` gets a `&mut` reborrow of the component field of the subtree being walked and shared references to `Done` values that are not being mutated (they are locals of the enclosing frames and are only borrowed, never moved, until the walk returns). Inputs can't escape because components and outputs are `'static` and `Input<'a>` is only handed to `update`.

**Why it is fast.** Every function in the walk is `#[inline]`, and the lookup chain is a chain of monomorphic one-liners. In the release build of the diamond, `run_once` compiles to nine instructions with no calls and no branches, and the benchmark runs at the speed of the hand-written struct (see [Performance](#performance-and-compile-time)). There is no `Option` per output and no tag store, because outputs only exist as locals of the fold.

At the end, `Pipeline::run` drops everything except the root: the finished outputs tree is destructured and only the root's `out` is returned (`Done::into_output`).

### 6. Startup (`Pipeline::build`, the only runtime lookups)

The config pass walks the graph and takes each config from `Configs` by `TypeId`, or uses `C::Config::default()`. A config can therefore never be missing. It collects all unused configs (supplied for components outside the graph) into a single `BuildError`, and checks for identity collisions. Then `Root::make` runs the construction described above. The `TypeId` maps exist only here.

## Guarantees and where each error shows up

| Mistake | Caught by | Error | Checked in |
|---|---|---|---|
| Typo in `input.field` | `cargo check` | E0609, with a "similar name" suggestion | `check_errors.sh field_typo` |
| Wrong type for an input | `cargo check` | E0308 | `wrong_input_type` |
| Subscribing to a non-component | `cargo check` | E0277 (also prints a stray "see issue #48214") | `not_a_component` |
| `Pipeline::<X>` where `X` is not a component | `cargo check` | E0277 `Component`/`Root` not satisfied | `root_not_a_component` |
| Generic code over `Pipeline<A>` with `A: Component` but no `Root` bound | `cargo check` | E0277 `A: Root` not satisfied | `generic_missing_root_bound` |
| Generic code with the `Root` bound | compiles, no feature gate | | `generic_root_bound_ok` |
| Two subscriptions with the same name | `cargo check` | `duplicate subscription name` | `duplicate_name` |
| Dependency cycle (default) | `cargo check` | E0391 "cycle detected when simplifying constant ... `HEADS`" (does not name your components) | `cycle`, `self_cycle` |
| Dependency cycle with feature `cycle-diagnostics` | `cargo check` | E0275 "overflow evaluating the requirement `A: NoDependencyCycleCheck`", at each component in the cycle | `cycle_diagnostics`, `self_cycle_diagnostics` |
| Keeping an input reference past `update` | `cargo check` | lifetime error | `keep_input_reference` |
| Naming the generated input struct | `cargo check` | E0412 | `name_input_struct_outside` |
| Same dependency twice through an alias | `cargo build` only (detected after monomorphization) | E0080 "lists the same dependency twice" | `duplicate_dep_via_alias` |
| More than `MAX_COMPONENTS` | `cargo build` only | E0080 | |
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

Measured on the generated 396-component, 1,504-edge layered graph (20 layers × 20 components, 4 random subscriptions each), with trivial `update` bodies, so these numbers isolate framework overhead. The new column has one more component, the root `App` (397 in total), which XORs the 20 last-layer outputs, and the hand-written baseline computes the same checksum (`assert_eq!`, "results agree"). 24-core Linux box, pinned `nightly-2026-10-30`. The master column is the previous raw-pointer runtime at commit `586adb1`, measured the same way on the same machine.

| | master (raw pointers) | now (safe fold) | hand-written struct |
|---|---|---|---|
| Time per run, release | 262–268 ns | 141–148 ns | 161–167 ns (alone); 159–161 in master's binary, 212–216 in the new binary (code layout, not the framework) |
| Time per run, debug | 59 µs | 157 µs (about 2.7×) | 2.6–2.8 µs |
| Debug build from scratch, incremental (includes the proc-macro dependencies) | 5.5 s | 6.8 s | |
| Rebuild after editing an `update` body | 0.58 s | 0.66 s | |
| Release build | 6.8 s | 8.3 s | |
| Debug build from scratch with `cycle-diagnostics` | | 9.0 s | |

Raw numbers: three runs each; timings vary by about ±2%. These are much lower than the numbers measured with rustc 1.75 (21 s debug, 30 s release), which were dominated by constant evaluation that the current compiler does faster. The earlier README also listed "rebuild after adding one subscription"; that row was not re-measured.

The new runtime is faster at run time in release, most likely because there is no per-output `Option` tag store (outputs only exist as locals of the fold, which LLVM keeps in registers), and it is a little slower to compile because the types do more work: every `Tree`, `Done` and lookup step is a monomorphized type or function. Debug builds do not inline, so every lookup step is a real call and `run` is about 2.7× slower than before in debug (still about 160 µs for 397 components). Editing an `update` body re-runs neither the plan nor the types.

## Design history and rejected alternatives

Read this before trying something "simpler"; most simpler ideas were tried.

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
| 10 | One-pass merge per component + hashed plan index | **Still current** for the plan. In the raw-pointer runtime: one buffer, constant offsets, a walk of `if FIRST { eval_node }`, outputs stored as `Option<Output>`. The runtime half was replaced in step 17. |
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
| 28 | Cycle check as a supertrait of every `Component` | Gives E0275 at each component in the cycle, +2.1 s (+30%) debug scratch. **Kept as the opt-in feature `cycle-diagnostics`**; the default is the cheaper E0391. |

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
| Nightly | Needs `const_type_name` and `generic_const_exprs`. The latter is incomplete: this nightly prints "`feature(generic_const_exprs)` is not supported with the next-generation trait solver ... reverted the setting to `-Znext-solver=coherence` for this crate" and an `incomplete_features` allowance is in the library. Only the library crate is affected, but the warning shows in every build of it. If the compiler stops supporting GCE, the typed storage tree has no replacement yet. |
| Identity | Hash of `type_name`. Collisions and name ambiguity are caught at startup by comparing `TypeId`s, not at compile time. |
| Capacity | `MAX_COMPONENTS = 512`. Compile-time memory is roughly `MAX_COMPONENTS ×` the number of components (each walk-order array has fixed capacity). |
| Compile time | Planning is roughly quadratic for dense graphs; the typed storage adds monomorphization work (6.8 s vs 5.5 s from scratch at 397 components). |
| Run time in debug | About 2.7× slower than the old raw-pointer runtime (157 µs vs 59 µs per run at 397 components): every lookup step is a real call without inlining. |
| Error quality | By default a cycle is reported as E0391 on the plan's constants, which doesn't name your components; `cycle-diagnostics` gives a readable E0275 for +30% debug build time. Subscribing to a non-component adds "see issue #48214" noise. Duplicate-through-alias only appears with `cargo build`. |
| Inputs | Shared references only. A component can't take ownership of an upstream output. |
| Outputs | Intermediate outputs live only during `run` and are dropped before it returns; only the root's output survives (by value). Outputs are built as locals of the fold, so large inline outputs live on the stack during `run`; box big outputs. |
| Threads | Single-threaded. A `Pipeline` is `Send` when its components are. |
| Per-run input | None: sources produce data from their own state. |
| Macro | Hard-codes `::dataflow_flat` paths (the crate uses `extern crate self as dataflow_flat;` so this works inside it too). A user item named `__<Component>_input` would collide. |
| `order()` | Returns `type_name` strings. Fine for debugging, not stable identifiers. |

## Suggested next steps

1. **Per-run external input.** For example, `run(&mut self, input: &I)` with source components reading `I`.
2. **Reduce compile time.** For example, smaller entries in the plan, sharing work between components with overlapping upstream sets, or making the cycle proof cheap enough to be always on.
3. **Remove GCE if a stable mechanism appears** (const traits, or a stable way to turn a constant into a type choice), which would also remove the next-solver warning.
4. **Parallel execution.** Components at the same depth are independent, but this needs `Send`/`Sync` bounds and a scheduler that doesn't reintroduce dynamic dispatch. The fold's structure (a tree of finished subtrees) suggests joining sibling subtrees.
5. **Owned inputs.** For example, move an output when it has exactly one subscriber, or add an explicit "consume" subscription kind. The by-value `Done` tree makes this more natural than before.
6. **Cheaper debug runs.** For example, collapsing the lookup chain for the common case of a direct dependency.

(Done since the last version of this list: named roots, replaced by the single root component; removing the `Option` tag stores.)

## Inspecting compiler output

All of these were used during development; flag spellings may differ on newer nightlies. They run on the `diamond` crate, where the output is small enough to read.

```sh
cargo rustc -p diamond -- -Zunpretty=expanded                       # derive output
cargo rustc -p dataflow-flat --lib -- --emit=mir -o /tmp/flat.mir    # generic walk and lookups
cargo rustc -p diamond -- -Zprint-mono-items=lazy                    # instances: one walk per Tree, one function per lookup step
cargo rustc -p diamond -- --emit=llvm-ir -o /tmp/d.ll
cargo rustc --release -p diamond -- --emit=asm \
    -C llvm-args=-x86-asm-syntax=intel -o /tmp/d.s                   # run_once: nine instructions, no calls, no branches
CARGO_INCREMENTAL=0 cargo rustc -p bench -- -Ztime-passes            # where compile time goes
```

Legacy symbol names hide generic arguments. To find a particular instance in the LLVM IR, match on the debug-info `DISubprogram(name: "walk<...>")` instead.

## Toolchain

The repo pins `nightly-2026-10-30` (rustc 1.101.0-nightly, 2026-10-29) in `rust-toolchain.toml`; bump that date to upgrade deliberately. Features in use, all confined to `dataflow-flat`: `const_type_name` (required) and `generic_const_exprs` (incomplete, see [limitations](#known-limitations-and-rough-edges)). Stable features in use: generic associated types (`type Input<'a>`, `type Inputs<'a>`, `type Refs<'a>`), `extern crate self`, const panics. User crates need no `#![feature]`. On a different nightly, compiler-internal details described above may differ, such as how incremental compilation fingerprints constants or what the next trait solver does with GCE. The design doesn't depend on the former, but the compile-time numbers might.
