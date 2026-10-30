# Working on this repo

Read `README.md` first. It covers the requirements, the full design, why each alternative was rejected, and the known limitations. The person cares about the concept more than polish, so check claims by building small artifacts rather than reasoning about them.

## Toolchain and commands

`rust-toolchain.toml` pins `nightly-2026-10-30`. Without rustup, use `RUSTC_BOOTSTRAP=1`. `dataflow-flat` enables `const_type_name`, `const_trait_impl` (the plan's walk; syntax is `const trait` / `const impl` / `[const]` bounds on this nightly) and `generic_const_exprs`; the library prints a "not supported with the next-generation trait solver ... reverted" warning (and uses `#![allow(incomplete_features)]`). That is expected. User crates (`example`, `diamond`, `bench`, the tests, the error cases) must not need any `#![feature]`.

Always run cargo under a timeout: `timeout 120 cargo ...`. Large graphs compile slowly, and past bugs made builds exponential. Run long checks one at a time and write results to a file, rather than chaining many slow commands in one shell call.

```sh
timeout 120 cargo test                           # plan assertions + runtime guarantees + execution order
timeout 300 scripts/check_errors.sh              # or pass case names to run a subset
timeout 120 cargo run -p example
python3 bench/gen_bench.py full 20 && timeout 200 cargo run --release -p bench
timeout 120 cargo test -p dataflow-flat --features cycle-diagnostics   # the feature is a no-op now (cycles are always diagnosed); kept so old commands work
```

`bench` is excluded from default members because its release build takes about 10 s. `python3 bench/gen_bench.py <full|flat|hand> [layers]` regenerates `bench/src/main.rs` (`flat` has only the framework side, `hand` only the baseline). Regenerate with `full 20` before committing; the committed file should be what the generator produces.

## Invariants: don't break these without discussing

1. **No dynamic dispatch while running.** No `dyn`, downcasts or lookups in `run` or the walk. Type-erased maps are only allowed in `Pipeline::build`.
2. **Compile-time constants contain no references.** `Plan` is a plain value of integer arrays (`Planned::PLAN` is one `&'static Plan` to it). A graph of `&'static` constants made incremental builds exponential. Keep constant arrays as integer arrays, not arrays of structs (validation is per field).
3. **Use `#[inline]`, never `#[inline(always)]`, on the walk and lookup chain** (`Walk::walk`, `Find`, `Get`, `GatherInputs`, `Context::shorten`). Forced inlining in debug builds expands every path through the graph, and it gave no gain in release.
4. **Each component is constructed, run and dropped exactly once.** This now holds structurally: the typed `Tree` stores each component once (at the edge where `Edge::FIRST` is true), construction is `Tree::make`, and drop is compiler drop glue. `tests/graphs.rs` and `tests/runtime.rs` check it.
5. **`#![forbid(unsafe_code)]` stays.** The old raw-pointer soundness argument is gone; do not reintroduce `unsafe`. If a design needs it, discuss first.
6. **Execution order is a guarantee**: depth-first post-order in subscription listing order (documented on `Component` and `Pipeline`, and in the README). `dataflow-flat/tests/order.rs` enforces it; change those tests only on purpose.
7. **GCE is confined to the library.** `generic_const_exprs` is a feature of `dataflow-flat` only. User crates need no feature gate and generic user code needs exactly one bound, `Root`. `tests/generic_user.rs` and `check_errors.sh generic_root_bound_ok` enforce it.
8. **`run` returns the root's output by value and nothing is stored between runs.** No `outputs()`, no `ran` flag, no `Option` per output.
9. **Keep `scripts/check_errors.sh` passing**, and add a case for every new class of user mistake. Match on error codes and on our own panic messages, not on full compiler output.

## Map of the code

| Path | What |
|---|---|
| `dataflow-flat/src/lib.rs` | Crate docs (the four stages, top-down), feature gates, module list, re-exports |
| `dataflow-flat/src/component.rs` | `ComponentSubscription`, `Component`, `DepList` (the nested list the derive builds) |
| `dataflow-flat/src/plan.rs` | The compile-time plan: `Described`, the const dependency walk (`visit`, `Upstream`), `Plan`, `Planned`, `Edge`, and the `plan_trace` test |
| `dataflow-flat/src/storage.rs` | The typed states tree: `Tree`, `If`/`Slot`, `DepStorage`, `Placed` |
| `dataflow-flat/src/lookup.rs` | Finished outputs and lookups: `Done`, `Pending`, `Top`/`TopHolds`/`IsSame`, `Get`, `Context`, `Find`, `GatherInputs` |
| `dataflow-flat/src/walk.rs` | The fold (`Walk`) and the `Root` bound |
| `dataflow-flat/src/build.rs`, `pipeline.rs` | `Configs` and the two startup passes; `Pipeline` |
| `dataflow-flat/tests/` | `runtime.rs` (run/drop/by-value guarantees), `graphs.rs` (lifecycle on diamond and example graphs, Send, panics, alignment), `order.rs` (execution order), `generic_user.rs` (one-bound generic code) |
| `dataflow-macros/src/lib.rs` | `#[derive(ComponentSubscription)]` + `#[subscribe(...)]` |
| `example/`, `diamond/` | Usage examples with a root `App` / `Sum`; `diamond` is for reading MIR/IR/asm |
| `bench/` | Generated benchmark; `gen_bench.py` is the source of truth |

Suggested next steps are at the end of `README.md`. The first is per-run external input.

Compile-time notes: the plan's `Upstream` bound is proved recursively by the trait solver, so graph depth costs solver depth (README, "Graph depth"); keep that proof two levels per layer (a bound on `C::Deps` rather than a trait on components) and keep the chunked `Upstream` impls. Profile with `-Zself-profile` and `summarize` (README, "Inspecting compiler output") before changing anything for speed.
