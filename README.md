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
python3 bench/gen_bench.py full 20 wide  # same, but the root subscribes to 100 components (last 5 layers)
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
    type Deps = (Double, Square,);                      // a leaf of the subscription tree, see below
    type Input<'a> = __Sum_input::SumInput<'a>;
    fn input<'a>(raw: <Self::Deps as DepList>::Inputs<'a>) -> Self::Input<'a> {
        let (__dep0, __dep1,) = raw;
        __Sum_input::SumInput { doubled: __dep0, squared: __dep1, __marker: PhantomData }
    }
}
```

**Why a tree of tuples.** Rust has no variadic generics, so no single impl can cover tuples of every length, and every operation (input types, the plan's walk, the storage, the run, the lookups) needs one impl per shape of list. A cons list `(A, (B, (C, ())))` needs only two impls, but its depth is its length: a root with 100 subscriptions made every recursive proof and every lookup 100 steps deep. So `Deps` is a 4-ary tree of plain tuples:

| Subscriptions | `Deps` |
|---|---|
| 0 | `()` |
| 1 to 4 | a *leaf*: `(A,)`, `(A, B)`, `(A, B, C)`, `(A, B, C, D)`. No child slots, no padding. |
| 5 or more | an *inner node* `(A, B, C, D, K0, K1, K2, K3)`: four entries and four child nodes (unused children are `()`). Each child is a leaf or an inner node itself. 12 subscriptions are `(A, B, C, D, (E, F, G, H), (I, J, K, L), (), ())`. |

Listing order is the four entries of a node in order, then its children left to right. The derive fills nodes left to right, in the shallowest tree that holds everything (capacity 4, 20, 84, 340 at depth 1 to 4), so depth grows with log4 of the width: 100 subscriptions are four levels. There are nine node shapes, so every operation is nine small impls generated by a `macro_rules!` per trait. The derive builds the tree type and a matching destructuring pattern (`let (__dep0, __dep1, __dep2, __dep3, (__dep4, __dep5), (), (), ()) = raw;`) that names the fields of the input struct. User code never writes a tree: the pipeline's root is a single component type.

**Why the input struct is in a hidden module.** `SumInput` appears in `Sum`'s public trait impl, so a private struct is a hard error (E0446). Rust 1.74 turned several of these private-in-public rules into warnings, but associated types in trait impls are still errors. Private fields fail as soon as the `Component` impl is written in a sibling module (E0616). Putting a `pub` struct with `pub` fields inside a private module passes both checks, and still leaves the struct unnameable from outside (E0412 if you try).

**Why `input.doubled` works anyway.** Inside `impl Component for Sum`, `Self::Input<'_>` is a projection on a concrete type, which the compiler resolves through the generated impl to `__Sum_input::SumInput<'_>`. Field access then only checks the *field's* visibility, which is `pub`. It never checks whether the struct's name is reachable. In code that's generic over `C: Component`, `C::Input` stays opaque and has no fields, but only the component's own `update` ever receives one.

The `PhantomData` marker keeps type parameters "used" when no subscription mentions them (otherwise E0392). The generated module is named `__<Component>_input`, keeping the component's capitalization to avoid collisions such as `Sum` and `SUM`.

### 2. Type checking (`cargo check`)

Library code is checked once, generically. In the user's crate, the compiler does three things. First, it resolves `Self::Input<'_>`, so field typos and wrong field types are ordinary type errors. Second, it checks the derive's `where T: Component` bounds, so a subscription to a non-component is rejected. Third, at `Pipeline::<App>::build` it needs `App: Root`, which holds for every component whose storage and walk exist; naming a type that isn't a component is rejected there.

**Cycles.** The plan's walk needs the trait bound `A::Deps: const Upstream`, which holds for a dependency list only if it holds for the dependency list of every component in it. The trait solver proves this when `Pipeline::<A>` is first used. On a cycle the proof needs itself and fails, as E0275 "overflow evaluating the requirement `(B,): Upstream`", which names a component of the cycle. This is always on and costs nothing extra: it is the same proof the plan needs anyway. (Before step 29 below a cycle surfaced as E0391 on the plan's constants, which names no component, and the readable form cost 30% of the build and was an opt-in cargo feature, since removed.) Two consequences of proving it by recursion: the proof is as deep as the graph (about six solver levels per layer), so graphs deeper than roughly 15 layers of four dependencies press on the compiler's default recursion limit of 128, which makes the compile slower, and much deeper graphs need `#![recursion_limit = "256"]` in the user's crate (see [Performance](#performance-and-compile-time)); and a component's own subscriptions add little to that depth because `Upstream` is implemented per tree node (100 subscriptions are four levels, where the cons list with chunks of eight made it thirteen).

### 3. Planning during compilation (constant evaluation, `cargo build`)

The run needs one kind of number: for each edge, whether this is where the walk first reaches the upstream component. It is computed by constant evaluation, by running the depth-first walk itself once, inside a `const fn` (`dataflow-flat/src/plan.rs`).

**Identity.** `Described::ID = fnv1a(type_name::<C>())`. This is what requires nightly (`const_type_name`). Comparing `TypeId`s isn't reliably available in constant evaluation, while `const_type_name` has been stable on nightly for years. Type names aren't guaranteed unique and hashes can collide, so `Pipeline::build` compares real `TypeId`s and panics on a collision. IDs 0 (`ROOT`) and 1 are reserved.

**The walk.** `Planned::PLAN` (one per root) is `Plan::of::<A>()`, which calls `visit::<A>(&mut plan, ROOT)`. Visiting a component `C` from parent `P`:

1. look `C` up by id. If it was reached before, do nothing (if its entry is still open, the walk is inside it: that is a cycle, which panics; in practice the trait solver finds cycles first, see below);
2. otherwise append `C` with parent `P`;
3. call `<C::Deps as Upstream>::visit_all`, which visits each dependency in listing order;
4. record where `C`'s range ends.

Step 3 is a call to a trait method on the dependency-list type from inside a `const fn`, which is what `const_trait_impl` is for: without it a `const fn` cannot follow the types. `Upstream` is a `const trait` implemented for `()` and for each node shape of the subscription tree (a leaf of 1 to 4 entries, or an inner node of 4 entries and 4 children, which visits its entries and then its children). The walk costs one hash probe per edge (about 1,900 visits for the 397-component benchmark). The previous formulation computed a walk-order constant per component and merged them, which is quadratic (237,000 entry visits at the same size) and was 60% of constant-evaluation time.

Because the walk is depth-first, the entries come out in first-reached order, which is a pre-order of the first-parent tree: **the storage subtree of a component is a contiguous range of the plan**. `Plan::range_of(id)` is the range `lo..hi` of the subtree of `id`, and "does the subtree of `top` hold `id`?" is a range test. More generally, the components stored under *any* run of consecutive entries of one component's subscription tree are a contiguous range too (each stored subtree is followed directly by the next one), so every finished piece of the run (see the fold) has a span `LO..HI` that is computed from integers.

Real values for the example graph, which `cargo test` asserts (`plan_trace`):

```
plan, in first-reached order, with the parent each was first reached from:
  App ←ROOT   Report ←App   TopWords ←Report   WordCount ←TopWords
  Tokenize ←WordCount   Source ←Tokenize   Longest ←Report   Len<Tokenize> ←App

edges that are NOT first visits: Report→Source, Longest→Tokenize, Len<Tokenize>→Tokenize
```

**Per-edge constants.** `Edge::<R, P, H>::FIRST = (parent H was first reached from == P)`, for root `R`. An edge is identified by its (parent, child) types, so a component listing the same dependency twice would make both edges `FIRST` and store (and run) the component twice. `Upstream::visit_all` rejects that with a panic during constant evaluation, which surfaces as compile error E0080. A leaf checks its (at most four) ids against each other. For an inner node the check must also see duplicates in different nodes, so it collects the ids of the whole tree in listing order into a scratch array of the `Plan` and checks that for duplicates. The derive dedupes subscriptions with identical spellings first, so this only triggers through things like type aliases.

**Every constant is a plain value with no references.** This is load-bearing; see [history](#design-history-and-rejected-alternatives). (`Planned::PLAN` is a `&'static Plan` to a single plain value, so using it does not copy 16 KB.)

### 4. The states tree (types)

Constants can select types if the crate enables `generic_const_exprs` (GCE). The library does, and **user crates need no feature gate**: the constants only appear in the library's own impls, whose bounds the user's crate proves without spelling them. The storage of a component is:

```rust
struct Tree<R, C: Placed<R>> {
    deps: <C::Deps as DepStorage<R, C>>::S,   // the first-edge dependencies' subtrees
    node: C,
}
// DepStorage for a leaf (A, B, C, D):   S = the tuple of Tree<R, X> for the entries X
//   whose edge is the first to reach them, in listing order: Mask4<{FIRST_A}, {FIRST_B}, ..>
//   picks the tuple type, e.g. (A, B, C, D) with only A and C first is (Tree<R, A>, Tree<R, C>)
//   and with none first is ().
// DepStorage for an inner node:         S = (the entries' tuple, S of child 0, .., S of child 3)
```

`C: Placed<R>` is the one-line spelling of "`C::Deps: DepStorage<R, C>`" (a supertrait with an associated-type bound; it holds for every component of a graph that compiles). So every component is stored exactly once, in the subtree of the edge that first reaches it, and an edge that does not first reach its dependency stores nothing: no `()` placeholder in the type, the construction, the walk or the lookups (an earlier version kept one for each of the 1,525 non-first edges of the benchmark). For `App` that is:

```
Tree<App> {
  deps: ( Tree<Report> { deps: ( Tree<TopWords> { deps: ( Tree<WordCount> { deps: ( Tree<Tokenize> { deps: (Tree<Source>,), .. }, ), .. }, ), .. },
                                 Tree<Longest>  { deps: (), .. } ),         // Tokenize is stored under WordCount
          .. },                                                           // Source is stored under Tokenize
          Tree<Len<Tokenize>> { deps: (), .. } ),
  node: App }
```

**Construction is safe code.** `Tree::make` builds the dependencies (entries in listing order, then children left to right), takes the config, calls `C::new`, and returns the struct. The `deps` field comes first, so construction order is topological, and the same recursion fills in `Pipeline::order()`. A panicking constructor unwinds through ordinary drops: what was built is dropped exactly once. **Dropping is compiler drop glue**: there is no destroy walk, no allocation and no manual layout. Over-aligned components are fine because they are ordinary fields.

### 5. The fold (`run`)

`run` walks the states tree structurally. Walking `Tree<R, C>` does this:

```rust
let deps = self.deps.walk(ctx);                 // upstream subtrees first, each returns `Done` by value
let out  = self.node.update(C::input(gather((&deps, ctx))));   // &mut field reborrow; inputs are & references
Done { deps, out }                              // this subtree's outputs, by value
```

`Done<R, C> { deps, out }` holds a finished subtree's outputs; `deps` has the shape of the storage (a tuple of `Done`s and, for wide components, of child tuples). A later sibling needs to read outputs of earlier ones (`Longest` reads `Tokenize`, which finished inside `TopWords`'s subtree). The walk therefore passes a *context* (`Context`): a type-level list whose runtime value is a tuple of shared references to finished pieces that live in the enclosing stack frames. A piece is either one finished component (`Done`) or one whole finished node of a subscription tree (`Node<R, X>`, a reference to a tuple). While a node's entries are walked, each finished entry is pushed onto the context for the next one (a later entry may read an earlier one). When a node is finished it enters the context of whatever comes next as **one** entry. So for an inner node the four entries are one `Node` entry for the children, and each finished child is one more; a component with 100 subscriptions walks its last child with a context of a few entries more than its parent's, not 99 (the context grows with the depth of the subscription tree, about 4 entries per level). The component being walked adds one entry of its own, a `Node` whose value is a reference to its finished dependency tuple, so its inputs are looked up in `(Node<deps>, ctx)`.

Each walkable piece (`Tree`, a tuple node, or `()`) implements `Walk<R, T>`, which says what it produces (`Out`), what a later sibling sees afterwards (`After`: a `Done` entry for a tree, a `Node` entry for a tuple, nothing for `()`), and how to build that. A tuple node walks its fields one after another, so all node shapes share the sequencing code.

**Lookups are monomorphized, not searched.** Every finished piece has a span `LO..HI` of plan indices (a `Done` covers its subtree; a tuple covers the union of its fields), so "does this piece hold `H`?" is a range test on integers, computed per node type with no references in the constants. An input is found by `Find<R, H, HERE>`: at each context entry `TopHolds<R, L, H>` says whether the entry holds `H`, and the impl for that constant is chosen. Inside the entry, `Select<R, H>::POS` says which part of the piece holds it (for a `Done`, itself or its dependencies; for a tuple, the index of the field) and `Get<R, H, POS>` is implemented per field position (`self.k`). The branch is a const parameter of the trait, so each step is one monomorphized function, and no step exists for paths not taken. After inlining, an input is a field access at a known path. Finding `Tokenize` for `Longest`: the first context entry is `Longest`'s own (an empty tuple: `Tokenize` is stored elsewhere), which does not hold it; the next is the finished `Done<TopWords>`, whose span covers `Tokenize` (`Tokenize ← WordCount ← TopWords`), so go in; `Select` says its dependencies, whose field 0 is `Done<WordCount>`, which also covers it; its field 0 is `Done<Tokenize>`, which is `Tokenize` itself.

**Why it is safe.** Everything is ordinary borrowing. `update` gets a `&mut` reborrow of the component field of the subtree being walked and shared references to `Done` values that are not being mutated (they are locals of the enclosing frames and are only borrowed, never moved, until the walk returns). Inputs can't escape because components and outputs are `'static` and `Input<'a>` is only handed to `update`.

**Why it is fast.** Every function in the walk and the lookup chain is `#[inline(always)]` (plain `#[inline]` stops being honored on large graphs, see the run-time cliff), and the lookup chain is a chain of monomorphic one-liners. In the release build of the diamond, `run_once` compiles to nine instructions with no calls and no branches, and the benchmark runs at the speed of the hand-written struct (see [Performance](#performance-and-compile-time)). There is no `Option` per output and no tag store, because outputs only exist as locals of the fold.

At the end, `Root::run` drops everything except the root: the finished outputs tree is destructured and only the root's `out` is returned (`Done::into_output`). The root is walked as `Tree<A, A>`: the plan is for root `A`, and the type parameter `R` of every plan-dependent type is the root component itself.

### 6. Startup (`Pipeline::build`, the only runtime lookups)

`Configs` is keyed by the compile-time id of the component (with its `TypeId` kept alongside). `Pipeline::build` first checks every supplied id against the root's plan (`Plan::contains`, a hash probe) and reports all configs for components outside the graph in a single `BuildError`, before any constructor runs. Then `Root::make` runs the construction described above: each `Tree::make` takes its config from `Configs` by id, or uses `C::Config::default()`, so a config can never be missing. Construction also keeps an id to `TypeId` map and panics if two components of the graph share an id (the ids are hashes of type names), or if a supplied config's component has the id of a different one. There is no separate pass over the graph for configs; the `TypeId` maps exist only here.

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
| Dependency cycle | `cargo check` | E0275 "overflow evaluating the requirement `(B,): Upstream`", naming a component of the cycle | `cycle`, `self_cycle` |
| Keeping an input reference past `update` | `cargo check` | lifetime error | `keep_input_reference` |
| Naming the generated input struct | `cargo check` | E0412 | `name_input_struct_outside` |
| Same dependency twice through an alias | `cargo build` only (detected after monomorphization) | E0080 "lists the same dependency twice" (also when the two are in different nodes of a wide subscription list) | `duplicate_dep_via_alias`, `duplicate_dep_across_nodes` |
| More than `MAX_COMPONENTS` | `cargo build` only | E0080 "too many components: raise dataflow_flat::MAX_COMPONENTS" | `too_many_components` |
| Config supplied for a component not in the graph | `Pipeline::build` at startup, before any constructor runs | `BuildError { unused }`, all at once | `unused_config` |
| Identity collision | `Pipeline::build` (during construction) | panic | |

A root listed twice used to be a possible mistake; with a single root component it cannot be written. Two subscriptions of the root to the same thing are covered by `duplicate_dep_via_alias`.

Runtime guarantees, checked by `cargo test` in `dataflow-flat/tests/`:

- Each component runs once per run, in the documented depth-first post-order, even with several edges into it (`order.rs`: the exact order on the example graph, and graphs where a later-listed dependency is upstream of an earlier one).
- `run` returns the root's output by value (`runtime.rs`); outputs of non-root components are dropped by the end of each run, and none exist before the first run (`runtime.rs`, counted with a `Drop` impl).
- Unused configs are reported together, before any constructor runs; a component with no entry in `Configs` gets `Config::default()` (`runtime.rs`).
- Wide components (5, 16, 17, 21 and 22 subscriptions in `order.rs`; 22 and 100 in `wide.rs`) keep listing order across node boundaries, including a later-listed dependency that is upstream of an earlier one, give every input the right value, and construct, run and drop each component once.
- A shared component is constructed once, updated once per run and dropped once (`graphs.rs`, `runtime.rs`); the same component under two names gives one dependency and the same reference.
- A pipeline is `Send` when its components are, can be moved and boxed between runs, and runs on another thread (`graphs.rs`).
- A constructor that panics during `build` drops what was already built, each exactly once (`graphs.rs`).
- Over-aligned components stay aligned (`graphs.rs`).
- Generic code over the root needs one bound and compiles without a feature gate (`generic_user.rs`, `check_errors.sh generic_root_bound_ok`).
- The framework contains no `unsafe` (`#![forbid(unsafe_code)]`).

The plan assertions above are in the `plan_trace` unit test.

## Performance and compile time

Measured on the generated 396-component, 1,504-edge layered graph (20 layers × 20 components, 4 random subscriptions each), with trivial `update` bodies, so these numbers isolate framework overhead. There is one more component, the root `App` (397 in total), which XORs the 20 last-layer outputs, and the hand-written baseline computes the same checksum inside every run (`assert_eq!`, "results agree"). The baseline keeps only the components in its struct; each output is a local passed by reference to the next `update`, like the framework's outputs (see "Hand-written baseline" below). 24-core Linux box, pinned `nightly-2026-10-30`. All columns were measured in one session on this machine, each from a copy of the tree at that commit in its own `CARGO_TARGET_DIR`: "before" is `45ffa94` (the cons-list version), then each kept step is cumulative.

**How run time is measured.** `scripts/bench.sh [layers] [wide]` regenerates the benchmark, builds it in release and runs the binary 5 times pinned with `taskset -c ${BENCH_CPU:-2}`. Each process does 5,000 untimed warm-up runs per side, then 11 timed rounds of 50,000 runs per side, alternating which side runs first, and prints the min and median ns/run per side over the rounds. Both sides run the same total number of times, so `results agree` compares the checksum of the last run. The script prints each process and a summary, then restores `full 20`. Noise guidance: run functions are fully inlined, so differences under about 5 ns, or between different binaries (a different graph is a different binary), are code layout and noise, not framework cost. Prefer the min; a large min-to-median gap means an order or alignment effect (at 25 layers one side was 205 ns instead of 178 ns in every round where it ran second, in one build, and the other side in another), not a slower framework. Unpinned medians drift by about 2 ns. Numbers from `scripts/bench.sh` (min of mins, ns/run, flat vs hand-written): full 20: 140.7 vs 140.4; full 25: 178.1 vs 177.4; full 20 wide: 147.3 vs 147.2.

Protocol (`bench` crate, `python3 bench/gen_bench.py full 20`): `cargo clean -p bench -p dataflow-flat -p dataflow-macros`, then `CARGO_INCREMENTAL=1 cargo build -p bench` (debug scratch, includes the macro crate but not `syn`/`quote`) and the same with `CARGO_INCREMENTAL=0`; edit the first generated `update` body and rebuild (edit 1, then edit 2 and 3 with other changes); `cargo clean --release ...` and `cargo build --release -p bench` (release scratch); run the release binary three times and the debug binary once. An edit that changes the length of a line near the top of a file recompiles much of the crate (1.7 s), while an edit that keeps text positions is the steady state (0.65 s); see the end of this section. One caveat on **release scratch**: in this protocol it is the first release build in a fresh target directory, so it also compiles `syn`, `quote` and `proc-macro2` (about 1.1 s); with those cached ("deps cached") it is 5.4 s before and 3.3 s now.

| | before (`45ffa94`) | 4-ary tree (`f26cd96`) | + configs by id (`d80b2c1`) | + first-edge entries only (`f022071`, now) |
|---|---|---|---|---|
| Time per run, release | 144–146 ns | 144–147 ns | 142–146 ns | 144–147 ns |
| Time per run, debug | 138 µs | 106 µs | 107 µs | **89 µs** |
| Debug build from scratch, incremental | 3.88 s | 2.56 s | 2.55 s | **2.40 s** |
| Debug build from scratch, non-incremental | 3.20 s | 2.15 s | 2.12 s | **2.00 s** |
| Rebuild after editing an `update` body: first / later | 3.02 s / 0.73 s | 1.75 s / 0.67 s | 1.75 s / 0.64 s | **1.66 s / 0.64 s** |
| Release build from scratch (deps cached) | 5.4 s | 4.15 s | | **3.3 s** |
| Release build from scratch (protocol, includes deps) | 6.53 s | 5.28 s | 5.14 s | **4.39 s** |
| Monomorphized functions (`-Zprint-mono-items`, `full`) | 20,479 | 14,461 | 13,639 | **12,608** |
| Overflow warning at the default recursion limit | yes | no | no | **no** |

Timings vary by about ±2% between runs. For the framework-only (`flat`) variant: 19,682 functions before, 11,811 now; debug scratch 3.71 s incremental / 3.05 s non-incremental before, 2.22 s / 1.88 s now; release 144–145 ns before, 147–152 ns now (within layout noise; `flat` has no hand-written code in the binary). The hand-written baseline numbers in the table's earlier measurements (214 ns, then 161 ns) were taken with the older baseline that stored every output in a struct field; see "Hand-written baseline" below for the current one.

**Functions by kind** (`flat`, before -> now): `Find` 3,947 -> 3,836 (steps through context entries; the context is shorter but the same lookups still cross it), `Get` 3,725 -> 2,492, `Walk::walk` 2,318 -> 799, `GatherInputs::gather` 1,921 -> 401 (one per node instead of one per subscription), `DepStorage::make` 1,921 -> 402, `DepList::collect_configs` 1,382 -> 0, `Slot::make` 746 -> 0 (`Stored4::make` 381), `Context::shorten` 216 -> 216, `Walk::after` 0 -> 215. The remaining 40% are lookups: `Find` and `Get` are 6,300 of 11,800.

**Wide components.** A root subscribing to 100 components (`python3 bench/gen_bench.py full 20 wide`, the last 5 layers; the graph is the same 397 components). Before: does not compile at the default recursion limit (E0275 "overflow evaluating the requirement `App: Root`"); with `#![recursion_limit = "256"]` it compiles with an overflow warning, debug scratch 6.09 s incremental (edit 1: 4.89 s); with 512 it is 4.57 s (edit 1: 3.29 s). Now: compiles at the default limit, no warning, debug scratch 2.55 s, edit 1 1.77 s, later edits 0.66 s, release scratch 4.67 s (protocol), 13,127 functions, 148–150 ns/run, debug run 99 µs.

**Recursion limit** (`#![recursion_limit]` in the user crate; debug scratch incremental / edit 1 / edit 2, seconds; ns/run is release):

| | before | now |
|---|---|---|
| L=20 (397 components), default limit | 3.87 / 3.02 / 0.73, warning | 2.41 / 1.67 / 0.65 |
| L=20, limit 256 | 2.85 / 2.00 / 0.70 | 2.44 / 1.67 / 0.65 |
| L=25 (497 components), default limit | 7.13 / 6.04 / 1.04, warning | 3.04 / 2.18 / 0.88 |
| L=25, limit 256 | 3.59 / 2.55 / 0.95 | 3.05 / 2.12 / 0.87 |
| ns/run, L=20 | 145–149 | 145–150 |
| ns/run, L=22 / L=24 | 158 / 176 | 158 / 223 |
| ns/run, L=25 | 294–297 | **745–771** |

The compile time no longer depends on the recursion limit at L=20, and the cliff at L=25 is mostly gone (the walk-based plan's proof depth is the graph's depth times two, and a component's own subscriptions no longer add to it). `MAX_COMPONENTS` is 512, so L=26 and above does not fit this generator.

**Which trait solver proves our bounds in user crates.** On this nightly `-Znext-solver=globally` is on by default "for testing" (`-Zhelp` still says `coherence (default)`, which is stale). A crate that enables `generic_const_exprs` is reverted to `-Znext-solver=coherence` (the old solver except for coherence) with the warning `dataflow-flat` prints; user crates are not reverted and print nothing, so they run the **next-generation solver**, and the expensive proofs (`App::Deps: const Upstream`, `Tree<A, A>: Walk`, the `Edge::FIRST` / `TopHolds::YES` consts in where-clauses) are proved by it in the user crate. Evidence: `RUSTFLAGS=-Znext-solver=globally` gives the same times as the default, while `-Znext-solver=coherence` and `-Znext-solver=no` (which behave the same) are 75 times slower. Measured on the `bench` crate alone (`-Ztime-passes`, non-incremental, one run each; the old solver's numbers are the same for `coherence` and `no`):

| graph | solver | debug scratch | release scratch | `type_check_crate` | `coherence_checking` | monomorphization collector |
|---|---|---|---|---|---|---|
| full 20 (397) | next (default) | 2.1 s | 3.0 s | 0.52 s | 0.46 s | 0.43 s |
| full 20 (397) | old (`coherence` / `no`) | 154 s / 152 s | 153 s / 151 s | 22.0 s | 20.6 s | 124.7 s |
| full 25 (497) | next | 2.7 s | 4.4 s | 0.66 s | 0.59 s | 0.56 s |
| full 25 (497) | old | 260 s (debug) | not run | 37.0 s | 35.1 s | 211 s |
| full 20 wide (397) | next | 2.4 s | 3.6 s | 0.57 s | 0.51 s | 0.48 s |
| full 20 wide (397) | old | 165 s (debug) | not run | 23.2 s | 21.4 s | 134 s |

The old solver is not wrong, just exponential-ish in the number of nested proofs: no warnings or errors differ, and the generated code is the same (release run time `scripts/bench.sh` protocol at full 20: 141.5 ns next, 141.5 ns `coherence`, 141.2 ns `no`; hand-written 141.5 to 142). The solver only changes compile time. At the default recursion limit, with `MAX_COMPONENTS` raised to 2048 in a scratch copy: the next solver builds up to 27 layers (537 components, 3.0 s debug) and fails from 28 layers (557 components) with the misleading `E0080 ... component not in this node` from `Select::POS` (53 errors at 28 layers); `#![recursion_limit = "512"]` fixes it (28 layers 3.1 s, 36 layers 4.2 s). The old solver, slowly, does not hit that limit: 28 layers builds in 343 s and 32 layers (637 components) in 489 s at the default limit, so the cliff above 27 layers is specific to how the next solver reports (or evaluates under) the recursion limit. Recommendation: no flag. The default (next solver in user crates) is the only usable choice; do not pass `-Znext-solver=coherence` or `=no` to user crates. If the default ever flips back to the old solver, the compile times of the large graphs become minutes, so watch the nightly bump.

**Run-time cliff for the largest graphs (fixed by `#[inline(always)]`).** With plain `#[inline]`, from about 477 components LLVM stopped inlining the root's `walk` into `run` (the walk function is about 6,700 lines), and the root's finished-outputs tree (about 4 KB) was returned by value through the return slot. Every `#[inline]` in `walk.rs`, `lookup.rs` and `pipeline.rs` is now `#[inline(always)]`, and LLVM honored it at every size tried (0 calls left in `run_flat`). Release ns/run (min/median of 5, measured before adopting it, `#[inline]` -> `#[inline(always)]`):

| Components | `#[inline]` | `#[inline(always)]` |
|---|---|---|
| 397 | 144 / 145 | 142 / 144 |
| 437 | 156 / 158 | 161 / 161 |
| 477 | 224 / 227 | 176 / 177 |
| 497 | 767 / 774 | 180 / 183 |
| 597 (cap raised to 2048 for the experiment only) | 754 / 758 | 281 / 288 |
| 797 (same) | 686 / 692 | 441 / 444 |
| 1193 (same) | 1252 / 1261 | 662 / 668 |
| 1979 (same) | 3234 / 3263 | 1103 / 1105 |

Release build time: no cost up to about 800 components (slightly faster), +16% at 1193, +59% at 1979; same peak memory. Re-measured after adoption on the fair locals baseline (flat vs hand-written, min/median of 5 runs, "results agree" every time): L=20 (397) 143/147 vs 142/144; L=24 (477) 209/211 vs 218/218; L=25 (497) 229/232 vs 232/234; `full 20 wide` 148/151 vs 146/146. (Absolute numbers on this later session ran higher than the experiment's for both sides, e.g. the hand-written one at 477 is 218 ns; the flat-to-hand ratio is what matters: the framework matches the baseline at every size.) A debug scratch build of `bench full 20` after `cargo clean -p bench -p dataflow-flat` took 2.6 s; debug builds were not otherwise measured and are not a goal.

**Hand-written baseline.** The baseline struct holds only the components; `Hand::run` computes each output as a local in the same order, passes references to the locals, and returns the XOR of the root outputs, so it does the same work per run as the framework (including the root XOR). An earlier version stored every output in a struct field (about 400 stores per run) and XORed the roots once after the timing loop; it measured 161 ns (214 ns in some builds, which was attributed to function alignment) and made the framework look about 10% faster than hand-written code. With locals the baseline measures 140-141 ns (min 140) against 142-146 ns for `flat`, i.e. the two are equal within about 3%, and the hand-written version is slightly ahead. At `full 20 wide` it is 146-147 ns against 148-149 ns. At L=25 it is 179-181 ns against 745-757 ns for `flat` (the run-time cliff above; the old baseline there was 270 ns). The alignment experiment (`-Zmin-function-alignment=64`) was not repeated with the new baseline.

The flat runtime is faster at run time in release than the old raw-pointer one, most likely because there is no per-output `Option` tag store (outputs only exist as locals of the fold, which LLVM keeps in registers). Debug builds do not inline, so every lookup step is a real call and `run` is slower than the raw-pointer runtime in debug (89 µs for 397 components, against 59 µs for the raw-pointer runtime and 138 µs for the cons-list version before this work). Editing an `update` body re-runs neither the plan nor the types.

**Where the compile time goes** (`-Zself-profile` with `summarize`, debug, non-incremental, `flat`-only L=20 benchmark, measured at `45ffa94` and not repeated; `-Ztime-passes` at that commit: total 3.17 s, monomorphization collection 1.37 s, LLVM IR 0.32 s, LLVM passes 0.42 s, type checking 0.61 s, macro expansion 0.15 s). Constant evaluation of the plan was 0.2 s and trait solving about 0.6 s; the rest is instantiating and compiling the monomorphized functions, which is what the tree, the config removal and the first-edge-only storage cut by 40%.

**Rebuilds after a scratch build** took 1.66 s for the first and 0.64 s for later ones in the earlier protocol (it was 3.0 s against 0.73 s before this work). Measured with `RUSTFLAGS="-Ztime-passes -Zincremental-info"` on the same three builds (`bench` crate only): scratch 2.19 s, first rebuild 1.70 s, second rebuild 0.69 s. The first rebuild re-runs the front end that the second skips: `type_check_crate` 0.52 s (second: 0.19 s), `coherence_checking` 0.48 s, `monomorphization_collector_graph_walk` 0.49 s and `codegen_crate` 0.53 s, whereas the second rebuild re-runs only borrow checking of the edited body and a few small passes. **The cause is not "the first rebuild" but what the edit does to the source positions.** Reproduced (`bench` full 20, debug, incremental, `-Ztime-passes`): scratch 2.5 s; a rebuild with no change or only a touched file 0.6 s; an edit that keeps the byte length and line numbers of everything (a constant `1` to `2`) 0.7 s, whether it is the first or the fifth rebuild; an edit that changes the length of a line near the top of the generated file (`wrapping_add(1)` to `wrapping_add(1001)`, or back to `7`) 1.7 to 1.9 s every time, even for the second and third edit (e.g. 3 to 30 and 31 to 4 were slow, 30 to 31 fast); inserting one line at the top of the file 2.3 to 2.4 s, the same as a scratch build; appending a comment at the end of the file 0.6 s. The earlier protocol made its first edit lengthen a line near the top (`wrapping_add(1)` to `...(1001)`) and its later edits keep the length, which looked like a one-off. `-Zself-profile` (with `summarize`) shows the slow rebuilds re-run `check_well_formed` (2,765 items), `typeck_root` (803), `mir_borrowck` (403), `eval_to_allocation_raw`, `items_of_instance` and the trait solver's `evaluate_root_goal_for_proof_tree_raw` for every component, where the fast ones re-run one body; the saved dependency graph has 720,000 nodes and 5.7 million edges (`TraitSelect`, `layout_of`, `resolve_instance_raw`, `eval_to_allocation_raw` dominate). So the position of source text feeds the query results of this crate (their hashes include spans), and shifting it makes every query that depends on a span of the component structs red. We did not find which query carries the span. It is partly ours: a trivial crate of 400 structs (plain `rustc -Cincremental`) showed no such difference (type check 0.03 s whatever the edit), but it is too small to show anything. A layout that splits the generated components into a second file (`mod flat;`) did not slow down after edits that change a line's length in either file (0.6 to 0.7 s; scratch 2.3 s), so in practice the rebuild cost is the cost of editing near the top of a file with hundreds of components; one file per layer or per group of components avoids it. This was not made the generator's default because the benchmark measures framework overhead, not editing. Nothing in the framework was changed for it. What did help is the number of instantiated functions: the first rebuild fell from 3.0 s to 1.66 s together with the function count.

## Design history and rejected alternatives

Read this before trying something "simpler"; most simpler ideas were tried. Rows 16 to 28 use the names of their time: `Branch` is now `Slot`, `StorageList` is `DepStorage`, `GetB`/`FindB` are `Get`/`Find` with a `HERE` parameter, `HeadHas` is `TopHolds`, `Refs` is `Context`, and `PushAll` no longer exists (row 31), `Pending` is a `Node` entry (row 40), and `Slot`/`If`/`IsSame` are gone (row 42).

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
| 28 | Cycle check as a supertrait of every `Component` | Gives E0275 at each component in the cycle, +2.1 s (+30%) debug scratch. Was kept as the opt-in feature `cycle-diagnostics`; the feature was removed once cycles were always diagnosed (step 33 made the check free). |
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
| 39 | **Subscription lists as a 4-ary tree of tuples** instead of the cons list `(A, (B, (C, ())))`: leaves of 1 to 4 plain entries, inner nodes of 4 entries and 4 children, one macro-generated impl per node shape for `DepList`, `Upstream`, `DepStorage`, `Walk`, `Get`, `GatherInputs`; a duplicate check that sees the whole tree | **Kept, current.** The cons list made the depth of every proof and every lookup the length of the list: a root with 20 subscriptions overflowed the default recursion limit while peeling it (`warning: overflow evaluating the requirement .. DepStorage ..`), one with 100 failed to compile at the default limit. The tree is log4 deep. Benchmark: 19,682 -> 14,461 monomorphized functions, debug scratch 3.9 s -> 2.6 s, release scratch 6.5 s -> 5.3 s, debug run 138 -> 106 us, the warning is gone and the compile time no longer depends on the recursion limit at L=20; a 100-subscription root compiles (the old code could not at the default limit). Run time in release unchanged at L=20. |
| 40 | Context entries per finished *node*: a finished node (a tuple of `Done`s and child tuples) goes into the context as one `Node` entry; finished pieces carry a span of plan indices, so "does it hold H" is a range test per node type and `Select`/`Get` pick the field | **Kept** (part of 39). Context length grows with the depth of the subscription tree instead of the number of subscriptions (a 100-subscription root walks its last child with a handful of context entries per tree level, about 4 times 4 levels at most by construction, not 99; estimated from the walk, not counted). `Pending` is gone: the component being walked pushes a `Node` for its dependency tuple. |
| 41 | Configs keyed by the compile-time id (`Configs` stores the `TypeId` too); `Pipeline::build` checks supplied ids against the root's plan before any constructor; construction takes the config or the default and checks id collisions with a map; `DepList::collect_configs` and `ConfigPass` removed | **Kept, marginal.** 822 fewer monomorphized functions (14,461 -> 13,639) but only 0.02 to 0.04 s faster (debug scratch non-incremental 2.14 s -> 2.11 s, five runs each, twice) and 0.1 s in the release build. Kept because it also removes a pass over the graph and a `HashMap<TypeId>` operation per component at startup, and the code is smaller. Unused configs are still reported before any constructor runs. |
| 42 | Store only the first-edge entries of a node (a `Mask` of the `Edge::FIRST` constants picks the tuple type); an inner node is `(entries, child 0, .., child 3)`; no `()` placeholders for the 1,525 non-first edges of the benchmark | **Kept, current.** Debug scratch 2.55 s -> 2.40 s, non-incremental 2.12 s -> 2.00 s, release scratch 5.1 s -> 4.4 s, debug run 107 -> 89 us, 13,639 -> 12,608 functions. `Walk`, `Span`, `Select` and `Get` needed no change for the missing positions because they work from plan ranges, not listing positions. |
| 43 | Cheaper lookups: skip up to four context entries per `Find` step (the stride is computed from a per-context distance constant `Dist`, and an `H`-independent `Tail` step does the skipping) | **Rejected.** 12,608 -> 11,582 functions, but debug scratch got slower (non-incremental 2.04 s -> 2.06 s over five runs each, twice; incremental 2.40 s -> 2.45 s), release unchanged, debug run unchanged (89 -> 90 us): the extra trait obligations (`Dist`, `Tail` projections) cost what the saved instances saved. The diff is kept out of the tree. A positional jump straight to the context entry (field path `.1.1.1.0` generated per distance) was not tried: it needs one impl per distance, and the distance reaches 60 at 20 layers. |
| 44 | **`#[inline(always)]` on the walk and lookup chain, adopted unconditionally** (reverses row 11) | Row 11's exponential debug blowup came from the dead `if false` branches of the old raw-pointer runtime, which the current fold does not have. What changed the decision: with plain `#[inline]` the release run time fell off a cliff on large graphs (LLVM stopped inlining the root's walk from about 477 components), and `inline(always)` removed it (497 components: 767 -> 180 ns). Debug-build cost is not a concern for this project (decision of 2026-10-30). |

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
| Identity | Hash of `type_name`. Collisions and name ambiguity are caught when the pipeline is built (an id to `TypeId` map during construction, a panic), not at compile time. A collision with a component that has a supplied config can show as that panic rather than as an unused config. |
| Capacity | `MAX_COMPONENTS = 512` (a fixed-size plan array; exceeding it is E0080 "too many components", checked by `check_errors.sh too_many_components`). A component can have 100 subscriptions (tested, `tests/wide.rs`, and the `wide` benchmark) at 4 levels of solver depth; the derive's tree has capacity 340 at depth 4 and grows by a level per factor of 4. One component's subscriptions are also bounded by `MAX_COMPONENTS` (a scratch array in the plan). |
| Graph depth | Several recursive compile-time proofs (storage, walk, the plan's `Upstream` bound) are as deep as the graph (and the subscription trees along it). A chain of about 43 components overflows the default recursion limit ("queries overflow the depth limit"). Wide layered graphs no longer press on the limit at 20 to 25 layers of 4 dependencies (they did from 17 layers before the subscription tree: debug scratch at L=25 and the default limit 7.1 s before, 3.0 s now, the same as with `recursion_limit = 256`), but very deep graphs still need `#![recursion_limit = "256"]` in the user crate. |
| Compile time | 2.0 to 2.4 s from scratch at 397 components (bench `full`, non-incremental / incremental), of which roughly 1.5 s is monomorphizing and compiling about 12,600 small generic functions (lookup steps are half of them); see [Performance](#performance-and-compile-time). |
| Run time in debug | About 1.5× slower than the old raw-pointer runtime (89 µs vs 59 µs per run at 397 components): every lookup step is a real call without inlining. |
| Run time in release for the largest graphs | Fixed by `#[inline(always)]` (it was a cliff from about 477 components; see [Performance](#performance-and-compile-time)). Debug builds of large graphs may be bigger and slower to build; that is accepted. |
| Recursion limit past ~557 components | Only reachable with `MAX_COMPONENTS` raised above 512. Past about 557 components the default recursion limit gives misleading `E0080 ... component not in this node` const panics from `Select::POS` instead of a recursion-overflow error. `#![recursion_limit = "1024"]` in the user crate fixes it. Not fixed in the library yet. |
| Error quality | A cycle is E0275 "overflow evaluating the requirement `(B,): Upstream`" pointing at library code; it names a component of the cycle but not the whole cycle. Subscribing to a non-component adds "see issue #48214" noise. Duplicate-through-alias (also across nodes of a wide list) and too-many-components only appear with `cargo build`, and the first also prints a stray E0284. |
| Inputs | Shared references only. A component can't take ownership of an upstream output. |
| Outputs | Intermediate outputs live only during `run` and are dropped before it returns; only the root's output survives (by value). Outputs are built as locals of the fold, so large inline outputs live on the stack during `run`; box big outputs. |
| Threads | Single-threaded. A `Pipeline` is `Send` when its components are. |
| Per-run input | None: sources produce data from their own state. |
| Macro | Hard-codes `::dataflow_flat` paths (the crate uses `extern crate self as dataflow_flat;` so this works inside it too). A user item named `__<Component>_input` would collide. |
| `order()` | Returns `type_name` strings. Fine for debugging, not stable identifiers. |

## Suggested next steps

1. **Per-run external input.** For example, `run(&mut self, input: &I)` with source components reading `I`.
2. **Reduce compile time further.** What is left is monomorphization and code generation of about 12,600 tiny functions, half of them lookup steps. Tried and rejected: skipping four context entries per lookup step (history row 43). Not tried: a positional jump straight to the context entry (field path generated per distance), sharing `Get` steps across components with the same container type, a flat finished-outputs structure instead of nested `Done`s. A shallower proof for the plan's `Upstream` bound would move the recursion-limit cliff out further for deep graphs.
3. **Remove GCE if a stable mechanism appears** (const traits, or a stable way to turn a constant into a type choice), which would also remove the next-solver warning.
4. **Parallel execution.** Components at the same depth are independent, but this needs `Send`/`Sync` bounds and a scheduler that doesn't reintroduce dynamic dispatch. The fold's structure (a tree of finished subtrees) suggests joining sibling subtrees.
5. **Owned inputs.** For example, move an output when it has exactly one subscriber, or add an explicit "consume" subscription kind. The by-value `Done` tree makes this more natural than before.
6. **Cheaper debug runs.** For example, collapsing the lookup chain for the common case of a direct dependency (the `Node` entry for the component's own dependencies already removed the per-dependency push).
7. **Make the recursion-limit failure past ~557 components a clear error** (needs a raised `MAX_COMPONENTS`): see the limitations table.

(Done since the last version of this list: named roots, replaced by the single root component; removing the `Option` tag stores; a cheaper plan and an always-on readable cycle error; subscription lists as a tree; no `()` placeholders for non-first edges; no separate config pass.)

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
