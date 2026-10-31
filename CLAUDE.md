# Working on this repo

Read `README.md` first. It covers the requirements, the full design, why each alternative was rejected, and the known limitations. The person cares about the concept more than polish, so check claims by building small artifacts rather than reasoning about them.

## Toolchain and commands

`rust-toolchain.toml` pins `nightly-2026-10-30`. Without rustup, use `RUSTC_BOOTSTRAP=1`. `dataflow-flat` enables `const_type_name` (the name hash that only chooses where the plan's table starts probing), `const_trait_impl` and `const_cmp` (the plan's walk, and `TypeId ==` in a `const fn`; syntax is `const trait` / `const impl` / `[const]` bounds on this nightly) and `generic_const_exprs`; the library prints a "not supported with the next-generation trait solver ... reverted" warning (and uses `#![allow(incomplete_features)]`). That is expected. User crates (`example` and its `examples/diamond.rs`, `bench`, the tests, the error cases) must not need any `#![feature]`.

Always run cargo under a timeout: `timeout 120 cargo ...`. Large graphs compile slowly, and past bugs made builds exponential. Run long checks one at a time and write results to a file, rather than chaining many slow commands in one shell call.

```sh
timeout 120 cargo test                           # plan assertions + runtime guarantees + execution order
timeout 300 scripts/check_errors.sh              # or pass case names to run a subset
timeout 120 cargo run -p example
python3 bench/gen_bench.py full 20 && timeout 200 cargo run --release -p bench
scripts/bench.sh [layers] [wide]       # robust benchmark: pinned, warm-up, 11 alternating rounds x 5 processes; restores `full 20`
```

`bench` is excluded from default members because its release build takes about 5 s. `python3 bench/gen_bench.py <full|flat|hand> [layers] [wide]` (`wide`: the root subscribes to the last 5 layers, 100 components, to exercise a very wide subscription list) regenerates `bench/src/main.rs` (`flat` has only the framework side, `hand` only the baseline). Regenerate with `full 20` before committing; the committed file should be what the generator produces.

## Invariants: don't break these without discussing

1. **No dynamic dispatch while running.** No `dyn`, downcasts or lookups in `run` or the walk. Type-erased maps are only allowed in `Pipeline::build`.
2. **Compile-time constants contain no references.** `Plan` is a plain value of integer arrays plus one `[TypeId; MAX_COMPONENTS]` (`Planned::PLAN` is one `&'static Plan` to it). A `TypeId` is pointer-typed inside const evaluation but plain integer data in the generated code, and measured no cost for incremental builds; it is the only exception to "integers only". A graph of `&'static` constants made incremental builds exponential. Keep constant arrays as integer arrays, not arrays of structs (validation is per field).
2b. **Component identity is the `TypeId`, never a hash.** `==` is the only operation on `TypeId` in const (no ordering, no unsafe). Finding a component goes through `table.rs` (name hash to start, `TypeId ==` to confirm); do not replace that by a linear scan of the `TypeId`s (+1.2 s debug compile time at 397 components) and do not let a hash decide identity. After the plan, everything works on indices (`At::INDEX`). `dataflow-flat/tests/same_type_name.rs` (with `tests/samename/`) tests two types with the same `type_name`.
3. **Use `#[inline(always)]` on the walk and lookup chain**: `Walk::walk`/`after`, `Find`, `Get`, `Select`, `GatherInputs`, `Context::shorten`, `Done::into_output`, `Root::run`, `Pipeline::run`. It is required: with plain `#[inline]` LLVM stops inlining the root's walk into `run` from about 477 components and the ~4 KB finished-outputs tree is returned by value (up to 4x slower). Debug-build size and time are not a concern for this project (the user's decision, 2026-10-30). Past ~557 components the default recursion limit gives misleading E0080 errors (README, limitations).
4. **Each component is constructed, run and dropped exactly once.** This now holds structurally: the typed `Tree` stores each component once (at the edge where `Edge::FIRST` is true; an edge is a parent and a position, so aliases of one type are two edges), construction is `Tree::make`, and drop is compiler drop glue. `tests/graphs.rs` and `tests/runtime.rs` check it.
5. **`#![forbid(unsafe_code)]` stays.** The old raw-pointer soundness argument is gone; do not reintroduce `unsafe`. If a design needs it, discuss first.
6. **Execution order is a guarantee**: depth-first post-order in subscription listing order (documented on `Component` and `Pipeline`, and in the README). `dataflow-flat/tests/order.rs` enforces it; change those tests only on purpose.
7. **GCE is confined to the library.** `generic_const_exprs` is a feature of `dataflow-flat` only. User crates need no feature gate and generic user code needs exactly one bound, `Root`. `tests/generic_user.rs` and `check_errors.sh generic_root_bound_ok` enforce it.
8. **`run` returns the root's output by value and nothing is stored between runs.** No `outputs()`, no `ran` flag, no `Option` per output.
9. **Keep `scripts/check_errors.sh` passing**, and add a case for every new class of user mistake. Match on error codes and on our own panic messages, not on full compiler output.

## Map of the code

| Path | What |
|---|---|
| `dataflow-flat/src/lib.rs` | Crate docs (the four stages, top-down), feature gates, module list, re-exports |
| `dataflow-flat/src/component.rs` | `ComponentSubscription`, `Component`, `Sub<C, I>` (an entry's marker: the component and its position in the listing), `DepList` (the subscription tree the derive builds: leaves of 0-7 entries (`()` is the empty leaf), inner nodes that are exactly 8 child subtrees and no entries; a tuple of 8 is always an inner node. The 8 is `ARITY`, defined once in `dataflow-macros`, which generates every per-shape impl through callback macros (`leaf_shapes!`, `inner_shape!`, `tuple_sizes!`, ...); the derive fills children left to right, `7 * 8^(d-1)` each) |
| `dataflow-flat/src/plan.rs` | The compile-time plan: the const dependency walk (`visit`, `Upstream`), `Plan` (`TypeId`s by index, first parents, the position that reached each, and subtree ends as indices), `Planned`, `Edge<R, P, H, I>` (`FIRST` = first parent and first position), and the `plan_trace` test |
| `dataflow-flat/src/identity.rs`, `table.rs` | Component identity: `At::INDEX` (position of a component in a root's plan), `Hashed` (name hash used only as a probe start), the open-addressing `Table` confirmed by `TypeId ==` |
| `dataflow-flat/src/storage.rs` | The typed states tree: `Tree`, `MaskN`/`StoredN` (only first-edge entries are stored), `DepStorage`, `Placed` |
| `dataflow-flat/src/lookup.rs` | Finished outputs and lookups: `Done`, `Node` (a finished tuple node as a context entry), `Span` (plan range of a finished piece), `Select`/`Get`, `Context` (also the span of its head entry), `TopHolds`, `Find`, `GatherInputs` |
| `dataflow-flat/src/walk.rs` | The fold (`Walk`: `Out`, `After`, `walk`, `after` per piece) and the `Root` bound |
| `dataflow-flat/src/build.rs`, `pipeline.rs` | `Configs` (keyed by `TypeId`) and `ConstructPass` (configs, order); `Pipeline` |
| `dataflow-flat/tests/` | `duplicates.rs` (the same dependency twice through an alias: constructed and updated once, `ptr::eq`), `runtime.rs` (run/drop/by-value guarantees), `graphs.rs` (lifecycle on diamond and example graphs, Send, panics, alignment), `order.rs` (execution order), `generic_user.rs` (one-bound generic code), `wide.rs` (lifecycle and values for 22 and 100 subscriptions), `same_type_name.rs` (two types with identical `type_name` are two components; uses the helper crates in `samename/foo-v1` and `foo-v2`, workspace members whose packages are `foo-v1`/`foo-v2` but whose library is `foo`) |
| `dataflow-macros/src/lib.rs` | `#[derive(ComponentSubscription)]` + `#[subscribe(...)]` |
| `example/` | Usage example with a root `App`; `examples/diamond.rs` (`cargo run -p example --example diamond`) is the small graph for reading MIR/IR/asm |
| `bench/` | Generated benchmark; `gen_bench.py` is the source of truth |

Suggested next steps are at the end of `README.md`. The first is per-run external input.

Compile-time notes: the plan's `Upstream` bound is proved recursively by the trait solver, so graph depth costs solver depth (README, "Graph depth"); keep that proof two levels per layer (a bound on `C::Deps` rather than a trait on components) and keep `Upstream` implemented per tree node (a component's subscriptions must stay an 8-ary tree, not a list: the cons list overflowed the recursion limit at 20 subscriptions). Profile with `-Zself-profile` and `summarize` (README, "Inspecting compiler output") before changing anything for speed.
