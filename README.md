# dataflow: compile-time-wired component graphs in Rust

A framework for building programs out of many small components (designed for hundreds) that form a directed acyclic graph. Each component is constructed from an owned config, has one `update` method that receives the outputs of the components it subscribes to, and produces an output of its own. You name only the components whose results you want (the *roots*). Everything they depend on is discovered by the compiler, built exactly once per concrete type (generic instantiations count as different types), and updated once per run in topological order. At run time there is no dynamic dispatch, no lookup and no bookkeeping: the release build of a `run` is the same machine code a hand-written struct would produce.

**Status:** working prototype, and the focus is still on the design rather than polish. Everything in this document was built and checked in a sandbox using rustc 1.75 with nightly features unlocked (`RUSTC_BOOTSTRAP=1`). A current nightly should work, but see [Toolchain](#toolchain).

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
| R3 | With hundreds of components, the user specifies only the minimal set (the roots). Nothing else is listed anywhere. |
| R4 | Each component takes an owned config struct in its constructor and has a single `update` that takes the results of upstream components and produces its own result. |
| R5 | Components are topologically sorted and invoked in that order. |
| R6 | No dynamic dispatch. |
| R7 | Nightly features are acceptable. |
| R8 | Declaration syntax: the user names subscriptions, and `update` receives a struct with named fields (`input.sub1`), not a tuple. |

## Quick start

```sh
cargo run -p example                     # text pipeline: 7 components from 2 roots
cargo run -p diamond                     # smallest interesting graph (4 components)
cargo test                               # plan assertions + runtime guarantees
scripts/check_errors.sh                  # 10 cases: each mistake fails the documented way

python3 bench/gen_bench.py full 20       # regenerate the 396-component benchmark
cargo run --release -p bench             # flat runtime vs hand-written
```

`rust-toolchain.toml` selects nightly. Without rustup, set `RUSTC_BOOTSTRAP=1` to unlock nightly features on a stable compiler; that's how this was developed.

**Wrap builds in a timeout** (`timeout 120 cargo ...`). Large graphs compile slowly (the release benchmark build takes about 30 to 70 s), and two past bugs made builds exponential (see [history](#design-history-and-rejected-alternatives)). A timeout turns a regression into a failure instead of a hang.

## Workspace layout

| Crate | Toolchain | Role |
|---|---|---|
| `dataflow-flat` | nightly | **The main runtime.** Flat memory layout planned during compilation. |
| `dataflow-macros` | any | `#[derive(ComponentSubscription)]` and `#[subscribe(...)]`. Re-exported by `dataflow-flat`. |
| `example` | nightly | Text pipeline: `Source → Tokenize → {WordCount → TopWords, Longest, Len<Tokenize>} → Report`. |
| `diamond` | nightly | `Src → {Double, Square} → Sum`. Small enough to read its MIR, LLVM IR and assembly. |
| `bench` | nightly | Generated 396-component, 1,504-edge layered graph, run through the flat runtime and a hand-written baseline. Not a default member. |
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
    type Output = u64;        // subscribers receive `&u64`

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

The application names only its roots and supplies configs:

```rust
use dataflow_flat::{deps, Configs, Deps, Pipeline};

type Roots = Deps![Report, Len<Tokenize>];

let configs = Configs::new()
    .with::<Source>(SourceConfig { lines: vec![/* ... */] })
    .with::<Tokenize>(TokenizeConfig { lowercase: true })
    .with::<TopWords>(TopWordsConfig { n: 2 });

let mut pipeline = Pipeline::<Roots>::build(configs)?;   // Err lists every unused config
loop {
    pipeline.run();                                      // each component once, in topological order
    let deps!(report, token_count) = pipeline.outputs().unwrap();   // typed, in the order the roots are listed
}
```

`Configs` is keyed by component type, not config type, so two components, or two instantiations of one generic component, never share a config by accident. Roots still use the tuple macros `Deps!` and `deps!`; replacing them with a named struct is a [suggested next step](#suggested-next-steps).

## How it works, start to finish

The traits (in `dataflow-flat/src/lib.rs`):

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

**Why a nested list.** Rust has no variadic generics, so no single impl can cover flat tuples of every length. With a nested list, two impls of `DepList` cover every length: one for `()`, and one for `(H, T)`, which handles the first element and recurses on the rest. Every operation (input types, the walk, gathering inputs, the plan, the cycle check) has this same recursive shape. `Deps![A, B]` is just syntax that produces `(A, (B, ()))`.

**Why the input struct is in a hidden module.** `SumInput` appears in `Sum`'s public trait impl, so a private struct is a hard error (E0446). Rust 1.74 turned several of these private-in-public rules into warnings, but associated types in trait impls are still errors. Private fields fail as soon as the `Component` impl is written in a sibling module (E0616). Putting a `pub` struct with `pub` fields inside a private module passes both checks, and still leaves the struct unnameable from outside (E0412 if you try).

**Why `input.doubled` works anyway.** Inside `impl Component for Sum`, `Self::Input<'_>` is a projection on a concrete type, which the compiler resolves through the generated impl to `__Sum_input::SumInput<'_>`. Field access then only checks the *field's* visibility, which is `pub`. It never checks whether the struct's name is reachable. In code that's generic over `C: Component`, `C::Input` stays opaque and has no fields, but only the component's own `update` ever receives one.

The `PhantomData` marker keeps type parameters "used" when no subscription mentions them (otherwise E0392). The generated module is named `__<Component>_input`, keeping the component's capitalization to avoid collisions such as `Sum` and `SUM`.

### 2. Type checking (`cargo check`)

Library code is checked once, generically. In the user's crate, the compiler does three things. First, it resolves `Self::Input<'_>`, so field typos and wrong field types are ordinary type errors. Second, it checks the derive's `where T: Component` bounds, so a subscription to a non-component is rejected. Third, at `Pipeline::<Roots>::build` it proves `Roots: AcyclicList`. `AcyclicList` holds for a list only if `Acyclic` holds for each component in it, and `Acyclic` holds for a component only if its own dependency list is `AcyclicList`. On a cycle, the proof would need itself, so it fails. With the derive, the error is "`(A, ()): AcyclicList` not satisfied" (E0599); with the legacy tuple syntax it was E0275 and showed the path.

### 3. Planning during compilation (constant evaluation, `cargo build`)

The run needs two kinds of numbers: where each component lives in a single buffer, and, for each edge, whether this is where the walk first reaches the upstream component. Both are computed by constant evaluation.

**Identity.** `Described::ID = fnv1a(type_name::<C>())`. This is what requires nightly (`const_type_name`). Comparing `TypeId`s isn't reliably available in constant evaluation, while `const_type_name` has been stable on nightly for years. Type names aren't guaranteed unique and hashes can collide, so `Pipeline::build` compares real `TypeId`s and panics on a collision. IDs 0 (`ROOT`) and 1 (`OWNER`) are reserved.

**Walk order per component.** `Described::SEQ` is a flat array listing the order in which a depth-first walk from the component first reaches each component upstream of it. Each entry is `(id, reached-from id, size and alignment of its storage Node<C>)`. It's computed as one constant per component, from the constants of the components it subscribes to. This is necessary because a `const fn` can't call trait methods, so it can't walk types itself (const traits would allow that, but they're unstable and their syntax keeps changing). The compiler caches each constant, so this is computed once per concrete type: a bottom-up computation over the whole graph. The merge goes like this. Start with the component itself, with parent `OWNER`. For each subscription in order, append that subscription's walk order, skipping anything already reached, and replace `OWNER` with this component.

**Why the merge is correct.** The real walk visits subscriptions in the same order and enters a component only the first time it's reached. The set of reached components always includes everything upstream of its members, because the walk finishes a component's whole upstream before moving on, and the graph has no cycles. So skipping an already-reached entry also skips everything upstream of it, and the remaining entries keep their relative order and their parents.

Real values for the example graph, which `cargo test` asserts:

```
SEQ(Report) = [Report←OWNER, TopWords←Report, WordCount←TopWords,
               Tokenize←WordCount, Source←Tokenize, Longest←Report]
              (Longest's Tokenize and Source, and Report's own Source, are already reached)

plan (roots merged with parent ROOT; memory in first-reached order, 160 bytes):
  slot 0 Report ←ROOT @0    slot 1 TopWords ←Report @24   slot 2 WordCount ←TopWords @48
  slot 3 Tokenize ←WordCount @64   slot 4 Source ←Tokenize @88   slot 5 Longest ←Report @112
  slot 6 Len<Tokenize> ←ROOT @144

edges that are NOT first visits: Report→Source, Longest→Tokenize, Len<Tokenize>→Tokenize
```

Execution order is the reverse of first-reached order in each branch: upstream components run first, so `Source` runs first. Memory order and execution order don't need to match, because every access uses a fixed offset.

**Plan and per-edge constants.** `Planned::PLAN` (one per `Roots` type) holds offsets, each component's reached-from parent, a hash index, and the total size and alignment. The run code reads two constants: `Offset::<Roots, C>::OFFSET`, and `Edge::<Roots, P, H>::FIRST = (parent H was first reached from == P)`. An edge is identified by its (parent, child) types, so a component listing the same dependency twice, or a root listed twice, would make both edges `FIRST` and run the component twice. The merge rejects that with a panic during constant evaluation, which surfaces as compile error E0080. The derive dedupes subscriptions with identical spellings first, so this only triggers through things like type aliases.

**Every constant is a plain value with no references inside.** This is load-bearing; see [history](#design-history-and-rejected-alternatives).

### 4. Code generation

The walk is generic code, written once:

```rust
// <(H, T) as DepList>::eval::<R, P>        (P = the parent whose list this is)
if Edge::<R, P, H>::FIRST { eval_node::<R, H>(base); }
T::eval::<R, P>(base);

// eval_node::<R, C>
<C::Deps as DepList>::eval::<R, C>(base);                 // subscriptions first
let raw = <C::Deps as DepList>::inputs::<R>(base);        // &outputs at base + OFFSET
let node = base.add(Offset::<R, C>::OFFSET) as *mut Node<C>;
let out = (*node).component.update(C::input(raw));
(*node).output = Some(out);
```

The pipeline flows through the compiler like this:

- **Generic MIR.** The flag is still unevaluated: `_2 = const _; switchInt(move _2)`.
- **Monomorphization.** This creates one `eval_node` instance per component and one list instance per edge, so code size grows with components plus edges, never with the number of paths. It also evaluates the constants.
- **LLVM IR.** The flag becomes a literal: `br i1 false` on `Square→Src`, `br i1 true` on `Double→Src`.
- **LLVM optimization (release).** LLVM removes the dead branches before inlining each function into its callers, so the whole diamond's `run` becomes straight-line code with no calls and no branches. `Src` is incremented once, its output is shared through a register, and the only bookkeeping left is writing the `Some` tag on each output.

### 5. Startup (`Pipeline::build`, the only runtime lookups)

The config pass walks the graph and takes each config from `Configs` by `TypeId`, or uses `C::Config::default()`. A config can therefore never be missing. It collects all unused configs (supplied for components outside the graph) into a single `BuildError`, and checks for identity collisions. Then the pipeline makes one allocation with the plan's size and alignment. The construct pass walks the graph again with the same first-visit flags, and writes `C::new(config)` at `C`'s offset after its subscriptions. Construction is therefore also topological, and `order()` records it.

### 6. Run, inputs, outputs, drop

`run(&mut self)` executes the walk and overwrites every output slot in place, with no allocation. Inputs are `&` references into other components' slots, at `base + constant offset`. The raw-pointer code is sound for these reasons:

- Every subscription ran earlier in this run, and nothing rewrites it until the next run.
- The only mutable access during `update` is to the component itself, and its own output is written after `update` returns. Distinct components occupy disjoint memory, and the graph is acyclic.
- `run` takes `&mut Pipeline`.
- References can't escape, because the component type and its `Output` are both `'static`. Keeping an input in `self` fails with "lifetime may not live long enough"; `scripts/check_errors.sh` checks this.

`outputs(&self)` returns the roots' outputs, typed, and returns `None` before the first run. Dropping the pipeline walks the graph once more, drops each component and output exactly once, and frees the buffer.

## Guarantees and where each error shows up

| Mistake | Caught by | Error | Checked in |
|---|---|---|---|
| Typo in `input.field` | `cargo check` | E0609, with a "similar name" suggestion | `check_errors.sh field_typo` |
| Wrong type for an input | `cargo check` | E0308 | `wrong_input_type` |
| Subscribing to a non-component | `cargo check` | E0277 (also prints a stray "see issue #48214") | `not_a_component` |
| Two subscriptions with the same name | `cargo check` | `duplicate subscription name` | `duplicate_name` |
| Dependency cycle | `cargo check` | E0599 "`(A, ()): AcyclicList` not satisfied" | `cycle` |
| Keeping an input reference past `update` | `cargo check` | lifetime error | `keep_input_reference` |
| Naming the generated input struct | `cargo check` | E0412 | `name_input_struct_outside` |
| Same dependency twice through an alias | `cargo build` only (detected after monomorphization) | E0080 "lists the same dependency twice" | `duplicate_dep_via_alias` |
| A root listed twice | `cargo build` only | E0080 | `root_listed_twice` |
| More than `MAX_COMPONENTS` | `cargo build` only | E0080 | |
| Config supplied for a component not in the graph | `Pipeline::build` at startup | `BuildError { unused }`, all at once | `unused_config` |
| Identity collision | `Pipeline::build` | panic | |

Runtime guarantees, checked by `cargo test` in `dataflow-flat/tests/runtime.rs`:

- Each component runs once per run, in topological order, even with several edges into it.
- No outputs are available before the first run.
- Unused configs are reported together; a component with no entry in `Configs` gets `Config::default()`.
- A shared component is constructed once and dropped once.
- The same component under two names gives one dependency and the same reference.

The plan assertions above are in the `plan_trace` unit test.

## Performance and compile time

Measured on the generated 396-component, 1,504-edge layered graph (20 layers × 20 components, 4 random subscriptions each), with trivial `update` bodies, so these numbers isolate framework overhead:

| | nightly `dataflow-flat` | hand-written struct |
|---|---|---|
| Time per run | ~250–550 ns | ~165–430 ns |
| Debug build from scratch (incremental) | 21 s | |
| Rebuild after editing an `update` body | 1.9 s | |
| Rebuild after adding one subscription | 8.9 s | |
| Release build | 30 s | |

Of the flat runtime's debug build, about 9 s is constant evaluation of the plan. That cost is roughly components × average upstream size, which is quadratic for a graph this densely connected. It's cached incrementally and recomputed only when a dependency list or an `Output` type changes, since output types determine offsets. Editing an `update` body doesn't touch it. The stable runtime's cost per run is pointer chasing between heap nodes plus a per-run "already ran?" check on every edge.

## Design history and rejected alternatives

Read this before trying something "simpler"; most simpler ideas were tried.

| Step | What | Outcome |
|---|---|---|
| 1 | Runtime registry: `HashMap<TypeId, Rc<dyn Any>>`, components pull dependencies in `build` | Works, but every check happens at run time. Rejected by R2. |
| 2 | `container!` macro listing every concrete type, `Provides<T>` impls, lazy `OnceCell`s | Compile-time checked, but someone has to list every component. Rejected by R3. |
| 3 | Derive on `Dep<T>` fields + `TypeId` registry; only roots named | Worked with hundreds of components, but it was a dependency-injection model (components hold handles to each other) rather than dataflow. Superseded by R4. |
| 4 | Dataflow with `Vec<Box<dyn Node>>` + `Box<dyn Any>` output slots | Correct, but uses dynamic dispatch and downcasts. Rejected by R6. |
| 5 | Typed `Rc<Node<C>>` handles + per-run epoch check on each edge | No dynamic dispatch and works on stable, but about 10× slower than a hand-written struct on trivial components. Removed once `dataflow-flat` superseded it (it measured ~4,700 ns per run on the benchmark graph). |
| 6 | Specialization for type-level deduplication (insert into a type-level set) | **Doesn't work**: specialized (`default`) associated types stay opaque during type checking, even for concrete types. Verified. |
| 7 | `generic_const_exprs` / const traits | Avoided: incomplete or with frequently changing syntax. |
| 8 | Compile-time plan from a graph of `&'static` description constants | Correct, but **incremental builds were exponential** (L=10: 67 s, L=20: >100 s). Incremental compilation fingerprints a constant by following its references, and it doesn't remember shared nodes, so the 4-way-branching graph was hashed as a tree, about 4^depth. Shrinking the plan arrays didn't help; profiling showed the cost in the stage that evaluates constants. |
| 9 | Per-component flat walk-order arrays merged at every partial list (A,B,C → B,C → C) + linear plan lookups | Fixed the exponential growth (L=20: 26 s), but did redundant merges. |
| 10 | One-pass merge per component + hashed plan index | **Current.** L=20 debug build in about 20 s, release in about 30 s. |
| 11 | `#[inline(always)]` on the walk | **Exponential in debug builds**: forced inlining at `-O0` happens before the dead `if false` branches are removed, so it inlined every path through the graph. Use `#[inline]`. Release builds were unaffected. |
| 12 | Tuple syntax: `type Deps = Deps![A, B]`, `fn call(&mut self, deps!(a, b): Inputs<'_, Self>)` | Worked, but felt awkward. Replaced by the derive (R8). Still used for roots. |
| 13 | `#[derive(ComponentSubscription({ sub1: X }))]` | Not valid Rust: derives can't take arguments. Became derive + `#[subscribe(...)]`. |
| 14 | Generated `SumInput` as a sibling of `Sum` | Must be `pub` (E0446) with `pub` fields (E0616 for sibling-module impls). Moved into a hidden private module so it can't be named from outside. |

Smaller decisions:

- **Configs** are keyed by component type.
- **`type Config: Default`** lets components go without config entries. It replaced an optional `default_config()` method, so a missing config can no longer happen; a config with no sensible default needs a placeholder `Default` impl (or a `new` that validates).
- **Inputs are shared references**, so any number of subscribers share one output without cloning.
- **Subscriptions with identical spellings** share one dependency instead of erroring.

## Known limitations and rough edges

| Area | Detail |
|---|---|
| Nightly | Needs `const_type_name`. `#![feature(const_mut_refs)]` is also present for old nightlies, and `#![allow(stable_features)]` silences it on newer ones. |
| Identity | Hash of `type_name`. Collisions and name ambiguity are caught at startup by comparing `TypeId`s, not at compile time. |
| Capacity | `MAX_COMPONENTS = 1024`. Compile-time memory is roughly `MAX_COMPONENTS ×` the number of components (each walk-order array has fixed capacity). |
| Compile time | Planning is roughly quadratic for dense graphs (about 9 s at 396 components / 1,504 edges). |
| Error quality | Cycle errors don't show the path. Subscribing to a non-component adds "see issue #48214" noise. Duplicate-through-alias and root-listed-twice only appear with `cargo build`. |
| Inputs | Shared references only. A component can't take ownership of an upstream output. |
| Outputs | Stored as `Option<Output>`, which costs one tag store per component per run. |
| Threads | Single-threaded, and `Pipeline` isn't `Send` (it holds a raw pointer). |
| Per-run input | None: sources produce data from their own state. |
| Panics | If a constructor panics during `build`, the buffer leaks. Nothing is double-dropped. |
| Roots | Still use tuple syntax: `Deps![...]` and `deps!(a, b)`. |
| Macro | Hard-codes `::dataflow_flat` paths (the crate uses `extern crate self as dataflow_flat;` so this works inside it too). A user item named `__<Component>_input` would collide. |
| `order()` | Returns `type_name` strings. Fine for debugging, not stable identifiers. |

## Suggested next steps

1. **Named roots.** A derive such as `#[derive(Roots)] struct AppOutputs { report: Report, count: Len<Tokenize> }` that generates the roots list and makes `outputs()` return a struct with named fields, removing the last tuple syntax from user code.
2. **Per-run external input.** For example, `run(&mut self, input: &I)` with source components reading `I`.
3. **Better cycle errors.** For example, a separate `cargo build`-time check that reports the path, or naming the trait something self-explanatory.
4. **Remove the `Option` tag stores.** Use `MaybeUninit` outputs and one pipeline-level "has run" flag.
5. **Parallel execution.** Components at the same depth are independent, but this needs `Send`/`Sync` bounds and a scheduler that doesn't reintroduce dynamic dispatch.
6. **Owned inputs.** For example, move an output when it has exactly one subscriber, or add an explicit "consume" subscription kind.
7. **Reduce planning cost.** For example, smaller entries, or sharing work between components with overlapping upstream sets.

## Inspecting compiler output

All of these were used during development; flag spellings may differ on newer nightlies. They run on the `diamond` crate, where the output is small enough to read.

```sh
cargo rustc -p diamond -- -Zunpretty=expanded                       # derive output
cargo rustc -p dataflow-flat --lib -- --emit=mir -o /tmp/flat.mir    # generic walk: `const _` + switchInt
cargo rustc -p diamond -- -Zprint-mono-items=lazy                    # instances: one eval_node per component
cargo rustc -p diamond -- --emit=llvm-ir -o /tmp/d.ll                # `br i1 false` / `br i1 true`
cargo rustc --release -p diamond -- --emit=asm \
    -C llvm-args=-x86-asm-syntax=intel -o /tmp/d.s                   # run_once: straight-line code
CARGO_INCREMENTAL=0 cargo rustc -p bench -- -Ztime-passes            # where compile time goes
```

Legacy symbol names hide generic arguments. To find a particular instance in the LLVM IR, match on the debug-info `DISubprogram(name: "eval<...>")` instead.

## Toolchain

Developed against rustc 1.75 with `RUSTC_BOOTSTRAP=1`. Features in use: `const_type_name` (required) and `const_mut_refs` (stable since 1.83). Stable features in use: generic associated types (`type Input<'a>`, `type Inputs<'a>`), `extern crate self`, const panics, and `Layout::from_size_align` in const. On a current nightly, compiler-internal details described above may differ, such as whether rustc skips generating code for constant-false branches, or how incremental compilation fingerprints constants. The design doesn't depend on them, but the compile-time numbers might.
