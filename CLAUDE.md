# Working on this repo

Read `README.md` first. It covers the requirements, the model, the syntax, why each alternative was rejected, performance and the known limits. The person cares about the concept more than polish, so check claims by building small artifacts rather than reasoning about them. `handoff/` holds design notes and session logs (`session-2.md` is a log; `README.md` and `event-spec.md` are earlier design notes with a note at the top on what is superseded).

## Toolchain and commands

`rust-toolchain.toml` pins `nightly-2026-10-30`. Without rustup, use `RUSTC_BOOTSTRAP=1`. `dataflow-flat` enables `const_type_name`, `const_trait_impl`, `const_cmp`, `panic_internals` and `generic_const_exprs`; the library prints a "not supported with the next-generation trait solver ... reverted" warning (and uses `#![allow(incomplete_features)]`). That is expected. User crates (`dataflow-flat/examples/*`, `bench`, the tests, the error cases) must not need any `#![feature]`.

`.cargo/config.toml` sets `-Zshare-generics=n` (dev edit time), `-Ctarget-cpu=native` and `[env] MALLOC_CONF = "dirty_decay_ms:-1,muzzy_decay_ms:-1"` (rustc's jemalloc reads `MALLOC_CONF`, not `_RJEM_MALLOC_CONF`; without it cold checks of thousands of components spend most of their time in page faults: 13.4M vs 0.68M at 4,079 components, 52 s vs 33 s). Cargo merges config files up the tree, so a generated app inside the repo inherits it. Release means that plus the `[profile.release]` in `Cargo.toml`; a number measured without them is not a release number.

Always run cargo under a timeout and a memory cap (see "Rules for agents"). Large graphs compile slowly, and past bugs made builds exponential. Run long checks one at a time and write results to files.

```sh
CARGO_BUILD_JOBS=1 timeout 600 cargo test -p dataflow-flat        # tests are heavy: build one test with --test name
CARGO_BUILD_JOBS=1 timeout 300 cargo test --doc -p dataflow-flat  # doc tests (pipeline.rs, sinks.rs, node.rs)
timeout 900 scripts/check_errors.sh                # all 56 cases; or pass case names to run a subset
timeout 120 cargo run -p dataflow-flat --example exchanges   # also basic, diamond, selbench
python3 bench/gen_app.py --size typical --mode enum --out DIR # --mode is required: 1a | enum | req
python3 scripts/measure.py --mode enum --sizes tiny,small --out DIR   # default --out is target/harness
python3 scripts/gen_selbench.py [E S K packets]                        # writes examples/selbench.rs
python3 bench/gen_bench.py full 20 && scripts/bench.sh                 # old flat-graph benchmark; restores `full 20`
```

Sizes of `bench/gen_app.py` (components): tiny 111, small 415, typical 823, then 2,044 (`m15-150-35`), 4,079 (`m30-300-70`), `perps` 12,412, `full` 29,650. Safe under a few GB: tiny, small, typical (dev first build about 2 GB, release about 1.5 to 3 GB). 2,044 check about 1.2 GB; 4,079 release about 6 GB (needs a larger cap, ask first); `perps` check about 5.3 GB with `MALLOC_CONF`, 126 s; release memory is about 0.94 MB per component (823: 0.87 GB, 4,079: 3.83 GB), so `perps` release is about 11.7 GB (killed at a 12 GB cap) and `full` about 28 GB. Apps above about 1,300 to 2,000 components need `#![allow(long_running_const_eval)]` in the application crate (`gen_app.py` adds it above 1,500). `bench/` is excluded from default members; `bench/gen_bench.py` output (`bench/src/main.rs`) should be what the generator produces (`full 20`) when committed.

## Invariants: don't break these without discussing

1. **No dynamic dispatch in a push.** No `dyn`, downcasts or lookups in the push program or the drivers. Type-erased maps (`dyn Any` configs and listener requests) are allowed only in `Pipeline::build` (`build.rs`, `request.rs`).
2. **Compile-time constants contain no references.** `Plan` is a plain value of integer arrays plus one `[TypeId; MAX_COMPONENTS]`. A graph of `&'static` constants made incremental builds exponential. Keep constant arrays as integer arrays, not arrays of structs (validation is per field). A failed plan must give an empty program and one error, not a const panic mid-plan (a panic once hung for 13 GB).
3. **Component identity is the `TypeId`, never a hash.** `==` is the only operation on `TypeId` in const. Finding a component goes through `table.rs` (name hash picks where to start, `TypeId ==` decides); do not replace it by a linear scan (+1.2 s at 397 components) and do not let a hash decide identity. After the plan, everything works on indices. `tests/same_type_name.rs` (with `tests/samename/`) tests two types with the same `type_name`.
4. **`#[inline(always)]` only in release, on the push path.** The pattern is `#[cfg_attr(not(debug_assertions), inline(always))]` with `#[cfg_attr(debug_assertions, inline)]`; lookups in `gather.rs`/`store.rs` are plain `#[inline]`. Forced inlining in debug builds was exponential. Debug-build size and time are secondary to release run time.
5. **Each component is constructed, run (at most once per push) and dropped exactly once.** It holds structurally (typed storage, construction in execution order, drop glue). `tests/lifecycle.rs` and `tests/duplicates.rs` check it. Aliases of one type are one component.
6. **`#![forbid(unsafe_code)]` stays.** Do not reintroduce `unsafe`. The `update`-only reflection design needs it and is stashed.
7. **Execution order is a guarantee**: depth-first post-order in subscription listing order, restricted to what runs (documented on `Component` and `Pipeline`, in the README). `tests/order.rs` enforces it.
8. **GCE is confined to the library.** User crates need no feature gate and generic user code needs exactly one bound, `S: Sinks<R>`. `tests/generic_user.rs` and `check_errors.sh generic_sinks_bound_ok` enforce it.
9. **The step semantics**: the root fires on every push; a component runs iff at least one edge fired (OR); an edge with selectors fires iff its parent fired and the pattern matched; outputs are stored and persist between pushes; `push` returns nothing and there is no accessor for stored outputs. After absorption, a component's condition terms must be pairwise exclusive (otherwise "joins branches that can fire in the same step"); every component must be reachable from the root, which must be `R`.
10. **Selected inputs read the last matched value** (stored only where some subscriber's condition does not imply the match), `Clone` is required only for stored ones, and `new` is given initial values for selected inputs (the selector's `Initial`). Handled edges are never read in `update` and never stored.
11. **Subscriptions are enum-style patterns only** (`Sel(Sel(name)): Src`, unit selectors, `_` for handled leaves); `=>` is a parse error. Anything else is a hand-written `Select`. Do not add syntax without discussing.
12. **Keep `scripts/check_errors.sh` passing**, and add a case for every new class of user mistake. Match on error codes and on our own messages, not on full compiler output.

Changed since the earlier model: `run` returning the root's output (replaced by `push` returning nothing and stored outputs), `Root` as a component (the root is a value type and the anchor is `Pipeline<R, Sinks>`), the fold with `Done`/`Find`/`Get`, the depth limit of 41, "inline(always) on the walk and lookup chain" (release-only on the push path now), and the 8-ary tree (`ARITY` is 16).

## Rules for agents (from experience)

* **Always pass `--mode`** to `bench/gen_app.py` and `scripts/measure.py` (`enum` for anything resembling a real app, `req` for listeners; `1a` has no selectors). The default of the old tools silently measured the wrong thing.
* **Set memory caps from `MemAvailable`**: `systemd-run --user --scope -q -p MemoryMax=<cap> -p MemorySwapMax=0 -- timeout <s> ...`, with the cap the smaller of the coordinator's limit and `MemAvailable` minus 2 GB, `CARGO_BUILD_JOBS=1` (or 4 for a measurement), one build at a time. An OOM kill is a result to record, never a reason to repeat the build unchanged or to run uncapped. Don't build the whole test suite for a docs check; build the one example or scratch crate.
* **No wait loops**: no `until`/`while` polling, no `pgrep -f`, no `tail --pid`. Run long builds in the background and wait for the completion notification. Keep tool output small (write to files, grep or tail them).
* **`.text` comparisons** of two builds (to prove a change generates identical code) are only valid when both are built in identically named directories (paths are embedded in the code). Swap the contents of one directory; don't copy to another name. Keep every `-Z` flag identical across builds, including the profile dir, or the timed build rebuilds from scratch.
* Put scratch work under the worktree's git-ignored `target/`, never `/tmp`. Never commit `Cargo.lock` changes made by scratch crates. Use literal absolute paths in commands (worktree guards refuse `$VAR` paths and compound commands that obscure them).
* The run-time noise threshold is under 5 ns per packet at 1,000 components: do not act on a smaller difference without instruction counts (callgrind) and several processes.
* One milestone per subagent; hand back at the milestone or at about 120k tokens of context with a 20 to 30 line report (global rules in `~/.claude/CLAUDE.md`).

## Map of the code

| Path | What |
|---|---|
| `dataflow-flat/src/lib.rs` | Crate docs, feature gates, module list, re-exports |
| `dataflow-flat/src/component.rs` | `ComponentSubscription`, `Component`, `Sub<N, C, I>` (an entry: the node, its selector chain and its position in the listing), `DepList` (the subscription tree: leaves of 0 to 15 entries, inner nodes of exactly 16 subtrees; `ARITY` = 16 is defined once in `dataflow-macros`, which generates every per-shape impl through callback macros) |
| `dataflow-flat/src/node.rs` | The sealed `Node` trait, `Root`, the kinds (root, component, anchor, stored-slot node), `InitOf` |
| `dataflow-flat/src/select.rs` | `Select`, `SelectChain`, `SelInit`, `Initial`, `Keep`, `ChainKey`, `ChainWalk`, `option`/`result` selectors, `GUARD` |
| `dataflow-flat/src/plan.rs` | The compile-time plan: the const walk, `Plan` (TypeIds, first parents, decisions, conditions, exclusivity, stored slots, the push program and its pools), `Planned`, limits (`MAX_COMPONENTS` etc.) |
| `dataflow-flat/src/identity.rs`, `table.rs` | Component identity (`TypeId`) and the open-addressing table (name hash to start, `TypeId ==` to decide) |
| `dataflow-flat/src/flat.rs` | Balanced flat type-level structure: `NthT` halving descent, `FlatOf`, positions and ranges |
| `dataflow-flat/src/store.rs` | Flat storage of states and stored outputs, `Slot`, constant descents (`AtPos`, `SelP`) |
| `dataflow-flat/src/gather.rs` | Reading inputs from stored outputs and the pushed root (`GatherIn`) and at construction (`GatherInit`) |
| `dataflow-flat/src/drive.rs` | The phases over component positions: construction pass and push program (`Seq`/`Run`/`RunIf`/`Switch`/handler runs) |
| `dataflow-flat/src/build.rs`, `pipeline.rs`, `sinks.rs` | `Configs` and the construction pass; `Pipeline<R, S>` (`build`, `push`, `order`, `len`); `SinkList`, `Sinks<R>` (the one bound), the anchor |
| `dataflow-flat/src/request.rs` | Listeners: `Listener`, `Request<S>`, the first pass that collects requested selectors |
| `dataflow-flat/src/handler.rs` | Handlers: `UpdateHandler`, `HandledEdge`, `Hd` wrapper, handler runs in the push program |
| `dataflow-flat/src/catalog.rs` | Catalogs for `for` entries: `Catalog`, `Concat`, `Product`, `Zip`, `M`, `Map`/`MapDeps` |
| `dataflow-flat/tests/` | `lifecycle.rs`, `duplicates.rs`, `order.rs`, `selectors.rs`, `patterns.rs`, `stored_selectors.rs`, `handlers.rs`, `for_form.rs` (+ `for_form_cats.inc`), `listeners.rs`, `wide.rs`, `generic_user.rs`, `smoke.rs`, `same_type_name.rs` (+ `samename/foo-v1`, `foo-v2`, workspace members whose library is named `foo`) |
| `dataflow-flat/examples/` | `basic/`, `exchanges/` (NIC, two exchanges, listeners, exclusive join, handler sink), `diamond.rs` (small graph for reading MIR/IR/asm), `selbench.rs` (generated by `scripts/gen_selbench.py`) |
| `dataflow-macros/src/lib.rs` | `#[derive(ComponentSubscription)]` + `#[subscribe]` + `#[listener]`, `#[derive(Root)]`, `#[derive(Demux)]` + `#[demux(markers = ..)]`, `catalog!`, the callback macros (`leaf_shapes!`, `inner_shape!`, `tuple_sizes!`, `sink_tuples!`, ...) |
| `bench/gen_app.py` | The multi-crate market app generator (the main benchmark); `--mode 1a|enum|req` required |
| `bench/gen_handler.py`, `gen_enum_keys.py`, `enum_keys_build.sh`, `enum_keys_run.sh` | Handler graph (`for` option), enum-key comparison and its build/run drivers |
| `bench/gen_bench.py`, `bench/src/main.rs`, `bench/Cargo.toml` | The old layered flat-graph benchmark; `main.rs` is generated (`full 20`) |
| `bench/sfb/sfb.rs` | Standalone store-forwarding repro (no framework dependency) |
| `scripts/check_errors.sh` | 56 error cases, each a tiny crate under `target/error-cases/` |
| `scripts/measure.py`, `scripts/bench.sh`, `scripts/gen_selbench.py` | Measurement harness for generated apps; pinned flat-graph benchmark; selector benchmark generator |
| `handoff/tools/` | `memset-shim/` (`LD_PRELOAD` shim that found the incremental page-fault cost; hard-coded paths of a removed worktree), `profile-reader/` (self-profile aggregators), `excl-group-differential.py` and `excl-group-counters.diff` (old-vs-new differential for `excl_group`), `probes-scratch.diff`, `pv.sh` (hard-coded old path) |
| `handoff/agent-env/` | `cap.sh` and `rules.md` for a container without systemd (hard-coded scratchpad path) |

## Compile-time notes

* The plan's `Upstream` bound is proved recursively by the trait solver, so graph depth costs solver depth. Keep that proof two levels per layer (a bound on `C::Deps` rather than a trait on components), keep the constant-evaluation walk at one stack frame per layer (the `Upstream` impls call `enter`/`leave` inline), and keep `Upstream` implemented per tree node (an 8- or 16-ary tree, not a list: the cons list overflowed the recursion limit at 20 subscriptions).
* Chains are the worst case: the solver depth is about 1.1 per layer plus about 35 (measured 2026-11-02: 20 layers need a recursion limit in (48,64], 30 in (64,96], 60 in (96,112]). At the default limit of 128, 88 layers build in about 4 s; 89 still builds with an overflow warning (7 s, 0.83 GB); 90 and deeper fail after an expensive error cascade (90: 15 s, 2.1 GB; 95: 20 s, 3.0 GB, in type checking and MIR borrowck of the user's `main`). With `#![recursion_limit = "4096"]` the cost is gentle: 100 layers 4.3 s, 250 layers 6 s / 0.63 GB, 500 layers 11.7 s / 2.1 GB. 16 chains of 50 (800 components) need no change. Typical graphs are wide and shallow.
* At large sizes `cargo check` time was dominated by page faults from rustc's allocator returning memory; keep `MALLOC_CONF` set (`.cargo/config.toml` does); `CARGO_INCREMENTAL=0` is only a little faster for a cold check with no edits after it.
* `cargo check` misses post-monomorphization errors (plan errors, exclusive joins); take limits from real builds.
* Profile with `-Zself-profile` (format v9 needs analyzeme/decodeme 12.0.3 or newer) or `-Ztime-passes` before changing anything for speed. `query-keys` on a graph of more than a few hundred components makes a profile of tens of GB.
