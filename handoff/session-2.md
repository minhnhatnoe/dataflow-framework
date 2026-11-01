# Session 2 (2026-11-01, cloud container): what was done, what is next

This updates `handoff/README.md` (sections 4–6). Numbers come from this machine (4 cores, ~14 GB, noisy VM).
It is much slower than the old one, so only compare numbers within this file.

## Environment

- No systemd here. Builds use `handoff/agent-env/cap.sh`, which applies a cgroup memory cap and a machine-wide lock
  (`build` takes it shared, `measure` exclusive). The subagent rules block is `handoff/agent-env/rules.md`,
  now with rule 8 (disk budget). Both hard-code this session's scratchpad path, so update it in a new session.
- The disk allowance is about 39 GB for the whole session. It filled once, from finished agents' generated apps and `target/` dirs (about 30 GB).
- `perf` is not installable. valgrind and objdump work.

## Merged to master

| Branch | What | Verified |
|---|---|---|
| `plan-linear` | Linear push-program builder plus `mk_hrun` bucketing by key. Handler N=256/512 builds with no `long_running_const_eval` allow. | Agent: `cargo test`, `check_errors` 30/30. Main agent: exchanges example output identical to master. |
| `runtime-gap` | Harness only. Cores build a local on both sides, plus `--kind E:B`, `--big-inline`, and `bench/sfb/sfb.rs` (store-forwarding repro). | Main agent re-ran `sfb`: field 16.0 ns, local 7.5 ns. |
| `mono-items` | Mono items per component −5% (typical 63.6 → 60.3): run body as macro, `Empty` as const, the 22 plain `inline(always)` made release-only. | Agent: `cargo test`, `check_errors` 30/30. **Main agent's own re-check (test + check_errors + exchanges) was interrupted. Run it first.** |

plan-linear vs master before it (`scripts/measure.py`): typical release app crate 31.3 → 28.5 s, runner 47.3 → 38.0 s,
dev initial 55.8 → 43.1 s. Edit rebuilds and run time unchanged within noise.

## Run-time gap (handoff step 2): mostly closed

With the fair harness (median of per-process minimum, 7 procs × 101 rounds, `taskset -c 2`):

| Case | Framework | Hand | Gap |
|---|---|---|---|
| tiny, on | 70.0 | 67.4 | +2.6 (was +9) |
| typical, on | 182.4 | 180.7 | +1.7 (was +6) |
| typical, enum | 181.1 | 181.7 | −0.6 |

- Cause was store forwarding in the harness cores: 8-byte field stores, then a 16-byte copy of the output.
- `target-cpu=native`, `-x86-disable-avoid-SFB` and `-x86-sfb-inspection-limit=200` change nothing.
- Why LLVM misses it, checked in `sfb` asm: X86AvoidStoreForwardingBlocks scans only the load's block and its *immediate*
  predecessors. In the repro, the half of the copy whose store sits one block back *is* split; the half whose stores are two
  blocks back is not.
- Possible upstream fixes: deeper bounded predecessor search in that pass, or an IR-level split or forward of a memcpy after narrow
  stores. An LLVM issue was not drafted or filed.
- What remains: about 2–3 ns, all on binance depth. Suspects: the root packet copy and `Frame` copies (tiny shows about 17 extra stores per packet).
  It is within noise. Taking the packet by reference would change the API.
- API decision unchanged: keep `update -> Output`. Plan: document "build a local, then return it" on `Component`.
  An out-param or `#[output]` field would avoid the copy but not make the stall impossible.

## Not merged

- **`blk-storage`** (`98db766` = cc27cb6 on bf316c8). The old incremental-check regression is gone (about 0.7 → 1.0 s at about 3k components).
  The trade-offs are why it stays on its branch:
  - Better: typical dev initial 50.3 s / 6.2 GB → 30.9 s / 2.1 GB, dev big edits 4.9/4.3 → 1.7/1.8 s.
  - Worse: cold `cargo check` at 2,649 components 21.6 → 55.1 s, all of it `type_check_crate`. The dev "longer edit" 4.6 → 13.1 s,
    of which 12 s is `generate_crate_metadata` (one sample).
  - Unmeasured: release peak at about 4k components (old machine 12.9 → 6.0 GB). This decides whether 30k scale fits.
- **`blk-cost`** (`7ce593f`): `blk-storage` with master (incl. plan-linear) merged in cleanly. Untested.
  - It was stopped before any measurement.
  - Planned: tests; 4k release time and peak, master vs blk at a 10 GB cap; dev edits ×3 with `.rmeta` sizes;
    the cause of the type-check slowdown.
  - Unmeasured suspects: `Blk<R,..>` carries the whole root type `R` in its name, and its field type goes through five trait projections.
  - Its scripts were in the session scratchpad, which is lost with the container.

## Next, in order

1. Re-verify master after the mono-items merge: `cargo test`, `scripts/check_errors.sh`, the exchanges example.
2. `blk-cost`: the measurements above, then a keep/drop decision on `Blk` (it gates step 6).
3. Mono items: the main remaining cost is storage descent, about 35 of 60 items per component (`Access::get/get_mut`,
   `Slot`, `AtPos`: one instance per tree level per position per storage view).
   - Next experiment: `ARITY` 8 → 16/32 in `dataflow-macros` (and the literal 8s in `flat.rs`), with the `wide.rs` tests.
   - Dropped attempts: a shared per-edge accessor (GCE "cycle detected when building an abstract representation"),
     and removing `Exec`/`Source` (recursion overflow in `Sinks::make`).
4. Per-component caching (handoff step 4).
5. 30k scale (step 6), plus the cost of `collect_requests`, which is still unmeasured.
   - A single build can use at most 10 GB here.
   - The typical dev build on master peaked at 6.2 GB.
   - The mono-items agent's typical dev build failed at its 5 GB cap on both sides.
6. Docs rewrite (step 7), including the build-a-local guidance.
