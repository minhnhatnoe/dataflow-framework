# Working on this repo

Read `README.md` first. It covers the requirements, the full design, why each alternative was rejected, and the known limitations. The person cares about the concept more than polish, so check claims by building small artifacts rather than reasoning about them.

## Toolchain and commands

`rust-toolchain.toml` selects nightly. Without rustup, use `RUSTC_BOOTSTRAP=1`.

Always run cargo under a timeout: `timeout 120 cargo ...`. Large graphs compile slowly, and past bugs made builds exponential. Run long checks one at a time and write results to a file, rather than chaining many slow commands in one shell call.

```sh
timeout 120 cargo test                           # plan assertions + runtime guarantees
timeout 300 scripts/check_errors.sh              # or pass case names to run a subset
timeout 120 cargo run -p example
python3 bench/gen_bench.py flat 20 && timeout 200 cargo run --release -p bench
```

`bench` is excluded from default members because its release build takes 30 to 70 s. `python3 bench/gen_bench.py <full|flat|hand> [layers]` regenerates `bench/src/main.rs`.

## Invariants: don't break these without discussing

1. **No dynamic dispatch while running.** No `dyn`, downcasts or lookups in `run`, `outputs`, or the walk. Type-erased maps are only allowed in `Pipeline::build`.
2. **Compile-time constants contain no references.** `Seq` and `Plan` are plain arrays. A graph of `&'static` constants made incremental builds exponential.
3. **Use `#[inline]`, never `#[inline(always)]`, on the walk** (`DepList::eval`, `inputs`, `eval_node`). Forced inlining in debug builds expands every path through the graph.
4. **Each component is constructed, run and dropped exactly once.** All three walks (`construct`, `eval`, `destroy`) must use the same `Edge::FIRST` flags.
5. **Soundness of the raw-pointer walk** relies on: an acyclic graph (`AcyclicList` bound), `run(&mut self)`, `outputs(&self)`, and `'static` components and outputs. Keep the `// SAFETY:` comments accurate when touching it.
6. **Keep `scripts/check_errors.sh` passing**, and add a case for every new class of user mistake. Match on error codes and on our own panic messages, not on full compiler output.

## Map of the code

| Path | What |
|---|---|
| `dataflow-flat/src/lib.rs` | Traits (`ComponentSubscription`, `Component`), `DepList` (the nested list and walks), compile-time plan (`Seq`, `Plan`, `Described`, `Edge`, `Offset`), `Configs`, `Pipeline`, `plan_trace` test |
| `dataflow-flat/tests/runtime.rs` | Runtime guarantees |
| `dataflow-macros/src/lib.rs` | `#[derive(ComponentSubscription)]` + `#[subscribe(...)]` |
| `example/`, `diamond/` | Usage examples; `diamond` is for reading MIR/IR/asm |
| `bench/` | Generated benchmark; `gen_bench.py` is the source of truth |

Suggested next steps are at the end of `README.md`. The first is named roots: `#[derive(Roots)]` making `outputs()` return a named struct.
