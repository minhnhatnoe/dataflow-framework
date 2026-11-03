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

## Closed by the user (2026-11-01)

No LLVM issue. No rewrite of the Claude-authored commits. No headless restart job.

## Handler API direction (user, 2026-11-01, not built)

"Oms should be a component btw. I think the annotation should still be subscription": one derive and `#[subscribe]`, no `#[derive(Handler)]` / `#[handles]`. Catalog lists use a plain tuple plus `_`. Queued after the optimization work.

## `update`-only output via reflection (2026-11-01, not built; needs the user's decision)

`handoff/experiments/update-only/`: the author writes only `update -> &Output`. The library finds the field of type `Output` at compile time with `#![feature(type_info)]`, and reads it with one `unsafe` block (so it breaks `forbid(unsafe_code)` unless the user relaxes it).
Miri clean under Stacked and Tree Borrows on a plain component. Two attacks from safe user code gave UB on the first version (lifetime laundering, because type IDs erase lifetimes; a packed struct at an unaligned address). Fixed by `C: 'static`, a check of the struct's alignment, and removing the nested-field search. Not integrated with the framework's storage or dispatch.

## Scale limit (2026-11-01)

The stack-frame error at about 2,000 components is fixed on master (`excl_group` loops instead of recursing). The next wall is the `long_running_const_eval` lint at about 2,000 components: it can only be allowed from the crate that evaluates the plan. With the allow, 4,079 components builds in release in 6:10 at a 14.6 GB peak. Open decision: rustflag or attribute in user apps, or split `Planned::PLAN` into chained consts.

**Decided by the user:** option (a). Apps above about 2,000 components use the allow or the rustflag; the plan stays one constant. Document it in the docs rewrite.

**Decided by the user (2026-11-01):** the `update`-only reflection design is stashed. `Blk`: re-test at 4,079 components, merge if the result looks good.

## `Blk` re-test (2026-11-01, not merged)

Clean numbers; they replace the 6:10 above, which was measured with other builds running.

| Case | Master | `Blk` |
|---|---|---|
| 4,079 components, release build | 3:52, 14.6 GB | 3:34, 6.2 GB |
| 2,044 components, `cargo check` | 18.5 s, 1.06 GB | 22.5 s, 1.26 GB |
| 2,704 components, cold `cargo check` | 26.5 s, 1.32 GB | 43.6 s, 1.66 GB |
| 2,704 components, check after an edit | 0.41 s | 0.49 s |
| 823 components, framework run time | 108.6 ns | 108.3 ns |
| 823 components, release app / runner | 13.3 / 18.4 s | 14.0 / 19.6 s |
| 823 components, dev edit, same length (big, 4 rounds) | 1.7-2.1 s | 0.8-0.9 s |
| 823 components, dev edit, same length (big + 3 small, 4 rounds) | 1.8-1.9 s | 0.8-0.9 s |
| 823 components, dev edit, length changes (big + 3 small, 4 rounds) | 2.3 s | 4.9-5.0 s |

On `blk-4k`: `cargo test` 18/18 suites, `check_errors.sh` 30/30, exchanges example ok.
The length-changing edit is the one failed merge condition. It is real, not noise, and its cause is unknown.

12,412 components on `Blk`: fails in the plan with "the push program is too large" (`MAX_LST = 32768` in `plan.rs`), not on memory. Exit 143 about 4 minutes in. The limit is not specific to `Blk`; master not run at this size. The cost of a larger table to small apps is unmeasured.

Query-level reuse has never been measured: the only self-profile attempt could not be read (reader `decodeme` 10.1.3 expects file format 8, this toolchain writes 9).

`patches/plan-linear-uncommitted.patch` and `patches/runtime-gap3-uncommitted.patch` hold uncommitted changes saved from worktrees before removing them.

## Blk result (2026-11-01): not merged, branch `blk-4k`

4,079 release: master 3:52, 14.63 GB; Blk 3:33, 6.17 GB. Typical run time equal (107.5). Typical release app 13.3 vs 14.0 s. Dev initial 17.8 vs 11.3 s. Same-length edit about 1.9 vs 0.85 s. Longer edit (6 alternating rounds) about 2.3 vs 5.05 s. Cold check at 2,704: 26.7 vs 43.6 s. Fails the "no worse on dev rebuilds" condition because of the longer edit. Perps (12,412) on Blk stops at "push program is too large: raise the list pool" (plan.rs), so a pool cap has to be raised to try that size.

## Caching and the `Blk` slowdown (2026-11-01)

Self-profile on the typical app (823 components, dev profile). Flags must be identical on every build, including the profile dir: changing any `-Z` flag forces a near-full rebuild of about 10 s. The reader needs `analyzeme`/`decodeme` 12.0.3 for file format 9. Each case was profiled once.

- **Body edit in the app crate: reused, per component.** The plan is not re-evaluated (`eval_to_allocation_raw`: 0 executed) and trait resolution does not re-run. About 9-10k queries execute against 168k loaded on master (231k on `Blk`), `mir_borrowck` runs once per edited file, and 1-4 of 82 codegen units recompile.
- **Body edit in the library crate: not reused.** About 130-178k `eval_to_allocation_raw` re-execute with only 1 codegen unit recompiled. Profiled, that is 8.2 s on master and 7.5 s on `Blk` (about 3.7 s self), but profiling inflates it: unprofiled, the same edit takes 3.65-3.94 s on master and 2.9 s on `Blk`. Cause not investigated. `typeck_root` shows only 2 executions in these profiles, so nothing is claimed about per-component type-check in this case.
- **Not tested:** reuse across different roots or apps.

**Why `Blk` is slower on a length change:** every length change re-runs `exported_generic_symbols`, part of sharing generics between crates. On master it costs about 0.46 s. On `Blk` it takes 2-5.5 s inclusive, because of the normalization and const evaluation it triggers (`try_normalize_generic_arg_after_erasing_regions`, `eval_to_allocation_raw`). Why a length change triggers it, and why `Blk`'s types are so costly there, are not established. Profiled `Blk` walls also differ by edit (6.5 s big only, 3.0 s big + 3 small), unexplained.

Confirmed by turning it off with `RUSTFLAGS=-Zshare-generics=n` (dev rebuilds, 3 rounds, seconds):

| Edit | Master | `Blk` | Master, flag | `Blk`, flag |
|---|---|---|---|---|
| Big, same length | 1.8-1.9 | 0.8 | 1.6 | 0.7-0.8 |
| Big + 3 small, same length | 1.8 | 0.8-0.9 | 1.6-1.7 | 0.8 |
| Big + 3 small, length changes | 2.2-2.5 | 4.8-4.9 | 1.6-1.7 | 0.8 |
| First dev build (1 run) | 18.2 | 11.2 | 17.1 | 11.3 |

With the flag on the typical app (one run each unless noted): release app/runner 13.3/18.3 s on master and 13.9/19.4 s on `Blk`, against 13.2/18.4 and 13.9/19.6 without it. The release runner binary is the same size with and without it (36,064,712 and 35,925,760 bytes). Framework run time 107.4 and 108.0 ns, against 108.4 and 108.8 without it, within noise. Library edit (2 real edits each) 2.92-2.96 s on master and 2.81-2.83 s on `Blk`, against 3.65-3.94 and 2.89-2.90 without it. On `blk-4k` with the flag: `cargo test` 18/18 suites, `check_errors.sh` 30/30.

Build folder on this app: 3.6-3.7 GB on master, about 0.8 GB on `Blk`. Peak memory on a library edit: about 2.0 GB on master, 0.8 GB on `Blk`. `Blk` remains unmerged pending the user's decision.

**Merged (2026-11-01, 754044a):** the user approved `Blk` plus the flag. `.cargo/config.toml` sets `-Zshare-generics=n`, and `bench/gen_app.py` writes the same file into generated apps. Verified through the config file: `cargo test` all suites, `check_errors.sh` 30/30, exchanges ok, typical 108.1 ns, release 13.8/19.5 s, first dev build 11.2 s, all three dev edits 0.8 s.

## `cargo check` growth at scale (2026-11-01, master before `Blk`)

12,412 components passes after raising `MAX_LST` to `1 << 16` (no measurable cost on typical or 2,044), but takes 15:24 at 6.1 GB. Profile, self time at 4,079 and 12,412, with call counts growing about linearly, so cost per call is what grows:

| Step | 4,079 | 12,412 | Cost per call |
|---|---|---|---|
| `check_well_formed` (946 of 948 s inclusive) | 9.8 s | 330 s | 16 ms to 209 ms |
| `resolve_instance_raw` | 2.4 s | 286 s | 3.9 us to 142 us |
| `eval_to_allocation_raw` | 35 s | 272 s | 40 us to 95 us |
| `evaluate_root_goal_for_proof_tree_raw` | 2.1 s | 49 s | about 15 us to 104 us |

Which constant or type is responsible is unknown. `query-keys` at 2,044 made a 26 GB profile; use a few hundred components or a streaming filter.

## Enum variants vs `Frame` keys (2026-11-01, branch `enum-keys`)

827 symbols, per packet, identical `Book<S>` bodies:

| | `Frame` + `On` | Enum + markers | Hand-written `match` |
|---|---|---|---|
| Random keys | 3.97 ns, 21.5 cycles, 0.01 mispredicts | 11.89 ns, 65.0 cycles, 0.95 | 10.62 ns, 58.4 cycles, 1.00 |
| Sorted keys | 3.94 ns, 21.3 cycles | 5.91 ns, 32.3 cycles | 4.50 ns, 24.9 cycles |

Compile time is tied. The wire-number `match` folds into storing the tag. The gap is one indirect-jump mispredict: `Frame`'s 827 arms merge into a table and a shared tail, the enum's stay as a jump table, cause unknown. Not tested with bodies that differ per symbol, which could erase `Frame`'s lead. The enum design is on hold; `Frame` and `On` stay meanwhile.

## Run-time gap: state and next step (2026-11-01)

This replaces "mostly closed" above: +1.7 ns came from the cloud VM. It is about 8% on the local machine (107.6 vs 99.6 ns, `measure.py` medians).

Typical harness, 823 components, `--mode on`, `taskset -c 2`, per packet:

| | Cycles | Mispredicts | `ls_stlf` | `load_not_complete` |
|---|---|---|---|---|
| Framework, random keys | 603.8 | 2.20 | 12.80 | 29.2 |
| Baseline, random keys | 557.9 | 2.24 | 3.21 | 18.6 |
| Framework, sorted keys | 491.9 | 0.90 | 14.84 | |
| Baseline, sorted keys | 488.9 | 0.89 | 3.85 | |

About 46 cycles remain at equal mispredict counts, nearly all only with random keys. Marginal cycles per mispredict are about 85 vs 49.

Ruled out as the main cause: compare tree (the flat chain took about 8 cycles and is merged), mispredict count, code footprint, alignment. Packet copy is at most about 12 cycles. Key in a register is about 9 cycles, not built: it needs a plan table.
Open suspect, unconfirmed: about 7-9 extra store forwards per packet, from outputs stored and reloaded by readers.

**Closed: gap accepted (2026-11-01), by the bounded rule of at least about 15 cycles.** The suspect was wrong. A `1:t`/`1:d` trace of one key-0 packet per side found no component output stored and reloaded by a reader; the extra close store/load pairs (about 9-14) are the packet copy (about 4-5) and the input-reference array `Big::update` receives (about 5-10). Whether each pair actually forwards was not checked.

`--big-inline` (per packet, same session, 5 rounds) roughly halved `ls_stlf`, but cycles did not follow with random keys:

| Keys | Variant | Cycles | Mispredicts | `ls_stlf` | `load_not_complete` |
|---|---|---|---|---|---|
| Random | Framework | 602.8 | 2.21 | 12.81 | 28.8 |
| Random | `--big-inline` | 609.4 | 2.23 | 6.90 | 29.7 |
| Random | Baseline | 556.2 | 2.24 | 3.22 | 18.1 |
| Sorted | Framework | 491.6 | 0.90 | 14.85 | 4.6 |
| Sorted | `--big-inline` | 482.7 | 0.90 | 6.51 | 4.2 |
| Sorted | Baseline | 489.0 | 0.89 | 3.87 | 3.4 |

So store forwards are not the main cost. The remaining gap is about 46 cycles with random keys only, cause unknown; marginal cost per mispredict is about 85 vs 49 cycles. Removal of the array in the `--big-inline` build was not verified directly. Branch `reg-outputs` has no commits.

Measurement caveat: changing `-Z` flags on only the timed build forces a near-full rebuild of about 12 s on both sides.

Open question, unanswered by the user: passing the root packet by reference (about 2-3 ns on a slower machine, changes the API). Left as is.

## Handler API direction (after optimization, not built, syntax not confirmed)

Replace `#[derive(Handler)]`/`#[handles]`. Agreed: Oms is a normal component and the annotation stays `#[subscribe]`. The user asked for "more rust syntax" and has not confirmed this form, so show it to them first:

```rust
#[subscribe(
    limits: Limits,
    impl Quoter<Btc>,
    for S in AllSymbols { impl Quoter<S> },
    for (X, S) in Product<Venues, Coins> { impl Book<X, S> },
)]
```

With `impl<S> UpdateHandler<Quoter<S>>` plus plain `Component::update`, the buffer as a field on `self`, no `Collect`/`begin`/`Buf`. Catalogs are plain nested tuples with `M<X, S>` pairs and `Product`/`Concat`/`Zip`, no filtering.

## Results after the `Blk` merge (2026-11-01)

**Small sizes, `Blk` + flag vs before.** 111 components: release 1.63–1.68 → 1.77 s, first dev build 1.37–1.43 → 1.49 s, edits 0.16 s both. 415: release 5.21–5.25 → 5.65 s, first dev 5.77 → 4.90 s, edits 0.51 → 0.37 s. Run time at 415 (4 rounds): 90.9–91.7 vs 90.7–91.4 ns, no change. At 111 (5 rounds): before 45.4–48.05 (middle 46.5), after 47.2–48.5 (middle 47.7), baseline unchanged; maybe about 1 ns real.

**12,412 components.** Pre-`Blk` `cargo check` 923 s, 6.1 GB. On `Blk` it did not finish within 1500 s (exit 124, no errors in the log). Release at that size is untested.

**Cold `cargo check`, same command** (`cargo check -p app --offline`, target dir removed, 4 jobs, two runs each). `bench/gen_app.py` defaults to `--mode 1a`, which has no selectors. Pass `--mode on` or `--mode enum` for anything meant to resemble a real app.

| Library | `1a`, 2,044 | `1a`, 4,079 | `on`, 2,044 | `on`, 4,079 |
|---|---|---|---|---|
| Pre-`Blk` master `0ad5cdf` | 8.8 s, 0.93 GB | 18.2–18.4 s, 1.75 GB | 18.5–18.7 s, 1.16 GB | 50.1–51.4 s, 1.89 GB |
| `Blk` master `86c7c97` | 12.4–13.0 s, 1.05 GB | 28.4–30.0 s, 2.02 GB | 23.6–23.8 s, 1.31 GB | 79.4–79.7 s, 2.24 GB |
| ARITY 16 master `64b2d0f` | | | 22.0–22.5 s, 1.22 GB | 61.6–64.8 s, 2.15 GB |
| `Blk` + `Field` (branch `check-descent`, f8a0af3) | 10.6–12.4 s, 0.72 GB | 52.2–55.7 s, 1.28 GB | | |

- With selectors, `Blk` is 1.3x slower at 2,044 and 1.6x at 4,079. ARITY 16 brings that to 1.2x and 1.2x.
- Selectors cost more than `Blk`: `on` is 2–2.8x `1a` on the pre-`Blk` library. From 2,044 to 4,079, `1a` pre-`Blk` grows 2.1x; `on` grows 2.7x pre-`Blk`, 3.4x on `Blk`, 2.8x on ARITY 16.
- The older figures (18.5 s at 2,044, about 52 s at 4,079) match `on` mode, which explains that conflict. The self-profile claim that `Blk` is 1.5x faster compared `on` figures to a `1a` run, so it is wrong.
- `1a` only: removing `-Zshare-generics=n` changes nothing (8.9/18.5 s and 13.2/31.3 s), and seven commits from `bf316c8` to `0ad5cdf` all cost 17.8–18.8 s at 4,079.
- `perps` (12,412) was generated with `--mode on`.

**ARITY 16 (merged, `64b2d0f`).** Typical, ARITY 8/16/32: mono items 69,563/58,427/48,355, release app 13.9/12.8/12.1 s, first dev build 11.3/10.4/10.0 s, run time 108.0/107.2/108.7 ns with identical instruction counts. 4,079 release: 216 s, 6.16 GB / 198 s, 5.94 GB / 201 s, 6.31 GB. 32 costs more at 2,044 and 4,079 and in `dataflow-flat` (5.3 s vs 2.1 s release). `perps` is untested on 16.

**`Demux` fix (merged, `257c668`).** One shared key function per enum, so output is linear in variants. Enum check at 827: 18.4 → 12.6 s; at 2,214: 124.5 s, 9.39 GB → 67.9 s, 3.02 GB (`Frame` 75.0 s, 2.98 GB). Release at 2,214 times out at 1200 s on both enum and `Frame`, so the limit is the framework. 827 symbols releases in about 80 s against about 14 s for the 823-component typical app; a wide dispatch may be costly (guess).

**Enum slowdown on typical (`on` vs `enum`, branch `enum-typical`).** Random keys 607.7 vs 624.4 cycles, sorted +4; 16 of the 17 are framework-side. Same instructions and mispredicts, 62 more frontend-starved slots. It sits mostly in Binance's 60-way dispatch. Size varies with layout: +1.2% at 58 Binance markets, +3.5% at 60, +3.0% at 61; always positive. No mitigation found. Tried: scalar copy, matching `On`'s layout to the enum's, `repr(u8)`/`repr(u32)`, padding, alignment flags, no jump tables, range check in `key_of`. PGO is untested because `llvm-tools` isn't installed.

**`Blk` profile, `1a` mode, so not representative of selector apps (query-keys at 415, 823, about 1,370).** `Planned::PLAN` is linear (0.47, 0.75, 1.16 s) and not the cause. `check_well_formed` is nearly all `app::build` (1.49, 3.39, 6.33 s). `resolve_instance_raw` of `store::SelP::POS`: about 105 calls per component, 4.9 to 8.8 µs per call.

**`Field` (direct child index, `check-descent`, not merged; `1a` mode only).** Valid, and `cargo test` passes. `SelP::POS` work falls about 30x at both sizes (`resolve_instance_raw` at 4,079: 440k calls, 10.2 s → 49k calls, 0.31 s), but `check_well_formed` of `app::build` rises from 30.8 to 55.7 s. Cause unknown. Memory falls 31–37%. `check_errors.sh`, run time and release were not run on it. A variant with the index as an associated const on a struct gave the same time. Scripts and apps: `.claude/worktrees/agent-ad73c0142f3b83542/target/w/`.

**Enum vs `Frame`, second test (branch `enum-keys`, 827 symbols, 5 rounds).** Cycles per packet:

| | `Frame` | Enum | Hand `match` |
|---|---|---|---|
| Different bodies, random | 73.4 | 72.8 | 62.4 |
| Different bodies, sorted | 33.4 | 35.1 | 35.5 |
| Identical bodies, random | 21.6 | 64.9 | 58.4 |

`Frame`'s lead with identical bodies comes from LLVM sinking its 827 identical arms into one block, which also fails with `-C codegen-units=1`. The enum misses it because three address calculations were hoisted out of the last switch group's 52 arms. Whether real apps get that merge is untested; check the typical app before committing to the enum syntax. Raw data: `.claude/worktrees/agent-a4e0d2d85fdaec8bc/target/enumwork2/`.

## Cold check at `perps`: the cost is incremental compilation (2026-11-02)

**`excl_group` fix (merged, `9fa5fc8`).** Split on the root-most unused place. The verdict is independent of the place: every pair is checked once either way (reasoned), and an old-vs-new scratch differential found no mismatch across tests, error cases and generated apps. `.text` and `.data` byte-identical at typical. Cold check, `on`: typical 8.3 → 7.6 s, 2,044 21.8 → 17.3 s, 4,079 59 → 43–51 s.

**Cause of the growth.** At large sizes most time is kernel page faults from `memset` in rustc's `ReadsRecorder::seed` (`rustc_middle::dep_graph::edges`). It resizes and zeroes a `Vec<u8>` sized to the dep graph per query task (reading of the disassembly), so tasks × graph size is quadratic: 80 GB zeroed at 2,044, 319 GB at 4,079. Largest source at 4,079: 61k `resolve_instance_raw` calls from `const_eval_resolve`, about 15 per component (39% of bytes). Also present on `nightly-2026-10-01`, so not a recent regression. Found with an `LD_PRELOAD` memset shim: `.claude/worktrees/agent-ab026fb31d6a1c588/target/ms/`.

| Cold `cargo check`, `on` | Incremental on | `CARGO_INCREMENTAL=0` |
|---|---|---|
| 4,079 | 46.5 s, 8.07M faults | 25.1 s, 0.89M faults |
| `perps` (12,412) | 2004 s, 7.5 GB, with self-profile | 81.8 s, 6.73 GB, 2.69M faults |

With incremental off, 4,079 to 12,412 is about linear.

| Dev build | Typical on | Typical off | 4,079 on | 4,079 off |
|---|---|---|---|---|
| First build | 9.8 s | 8.7 s | 72.2 s | 46.2 s |
| Edit | 0.7 s | 8.5 s | 3.6 s | 46 s |

**`perps` release:** OOM-killed at the 18 GB cap, reaching `Compiling app`, about 2:40 CPU. Which phase hit the cap is unknown.

**Worktree cleanup (2026-11-02).** Finished worktrees were removed; branches were kept. Raw data under the `.claude/worktrees/agent-*/target/` paths named in these notes is gone, except `agent-ae262aab3ddfbb8d8` and `agent-adba54fa14ed56a76`. Small tools were saved to `handoff/tools/`: the `LD_PRELOAD` memset shim (`memset-shim/`), the self-profile aggregators (`profile-reader/`: `agg`, `keys`, `selfw`), the bound probes (`pv.sh`, `probes-scratch.diff`), and the `excl_group` differential and counters. Not saved: the reader's plain `src/main.rs`.

**`perps` release memory (2026-11-02).** Stopped: it can't fit on this machine as used. RSS was 6.9 GB after analysis, 14.4 GB after `monomorphization_collector_graph_walk` (+7.5 GB, 49.7 s), about 15 GB at `find_cgu_reuse`, and 17.5 GB when killed machine-wide, about 2.5 minutes in. Next step when this is resumed: mono collector memory. Not measured: flags (fewer jobs, codegen units), run time at this size, and a clean incremental check at `perps`.

## Syntax work (2026-11-02)

Optimization paused by the user's agreement; the parked list is in `MEMORY`'s scale target and the sections above.

- **Patterns (`b3f0804`), listeners (`35619c9`), unit selectors and `=>` removal (`1c00bbc`) are merged.** `.text` is byte-identical at typical for patterns and for `=>` removal. Listeners: +0.13% `.text` in enum mode, run time 108.3–108.8 → 108.8–109.5 ns (probably layout, unproven), release memory 1.31 → 1.46 GB.
- `On`, `KeyListener`, `Keyed`, `Symbol` and `collect_requests` are removed on branch `remove-on` (enums are the only keyed form); branch `keep-on-mode` keeps them for the run-time comparison.
- **Unknown limits:** one component can't read two variants of one listener (existing plan rule). `Data: E` on a data variant compiles and ignores the payload.
- `handoff/README.md` and `handoff/event-spec.md` still show `=>` examples.

## Allocator setting, caching, release memory (2026-11-02)

`MALLOC_CONF=dirty_decay_ms:-1,muzzy_decay_ms:-1` is set in `.cargo/config.toml` `[env]` (`_RJEM_MALLOC_CONF` is ignored on this nightly; results above naming it ran on defaults). Cold check, `enum`, incremental on: 4,079 48.9 s, 12.7M faults, 1.33 GB → 30.0 s, 0.68M, 1.49 GB; `perps` 125.9 s, 5.29 GB, 1.68M faults. It raises release peak: typical full profile 0.90 → 1.42 GB, so use a shell override such as `MALLOC_CONF=dirty_decay_ms:10000,muzzy_decay_ms:0` for large release builds.

Release memory, full profile, `enum`, without the setting: 823 0.87 GB 16 s; 2,044 1.86 GB 59 s; 4,079 3.83 GB 226 s, linear at about 0.94 MB per component, peak at the end of `codegen_to_LLVM_IR`. `perps` extrapolates to about 11.7 GB and was killed at a 12 GB cap; `full` to about 28 GB. No single holder; flags tried made no difference.

Caching, `enum`, typical, dev: component body 0.9-1.0 s, 1 of 93 units; big + 3 small 0.95 s; line length or inserted line 0.9 s; subscription list, add or remove component, or sink order 8-9 s (the whole plan re-evaluates); library edit 2.5-3.3 s even for a comment (about 123k queries, 1 unit), 5.9 s right after app-only edits. A second app over the same library reuses nothing. `-Zincremental-ignore-spans` makes a library comment edit 0.95 s and doesn't help real library edits. Default `-Zshare-generics` did not reproduce the 4.9 s regression in the config comment (0.98 vs 0.91 s). Cold dev build at 2,044 was killed at a 3 GB cap, so edit times at 2,044 and 4,079 are unmeasured.

Chain depth: 85 components 2 s, 90 14 s, 95 killed at 2 GB (docs agent). `for` form works but repeats bounds, gives a nonsense rustc suggestion when they're missing, and its body can only be a single node type.

## Run-time gap, depth, allocator, edits, PGO (2026-11-02)

Measure on a cpu with an idle SMT sibling (siblings differ by 12; cpu 10/22 was clean) and compare the min of alternating runs. A busy sibling adds about 17%; that was the "two cycle modes".

Run-time gap, typical, `enum`, full profile: fusing a `Run` with the `Switch` on its output (`PG_RUNSW`) gives about 13 cycles (611.6 to 598.0 min), baseline 568.9. Remaining gap about 25 to 30 cycles, 5.5 ns (108.14 vs 102.60 ns) or 4.5 ns depending on the run. Ruled out: `Big`'s 42-pointer input array (inlining gives the same cycles), stored selector slots (typical has none), the frontend (fewer op-cache misses and stall slots than baseline), store-forward failures (21 vs 0.5 per packet, but inlining `Big` cuts them to 6.4 with no gain), any single load-stall site over 10 cycles. Cause not found.

PGO, trained on a different seed: framework 108.14 to 103.84 ns, baseline 102.60 to 98.90; small -4.8%, tiny -6.6%. `scripts/pgo.sh`. Flags with no gain: see README.

Chain depth is not exponential. Default `recursion_limit` 128: 88 layers 4 s; 89 builds with an overflow warning, 7 s, 0.83 GB; 90 and deeper fail after an error cascade (15 s, 2.1 GB at 90). `#![recursion_limit = "4096"]`: 500 layers 11.7 s, 2.1 GB.

Allocator: keep `-1/-1`. `muzzy_decay_ms:-1` alone is fine up to 4,079 (32.3 s, 0.79M faults) but failed at `perps` (500 s timeout, 256M faults at 460 s; `-1/-1` 119 s, 5.28 GB, 1.68M faults). Release override `dirty_decay_ms:10000,muzzy_decay_ms:0`: typical 0.88 GB vs 1.34 GB, same time.

`perps` release was killed at 12 GB and at 13 GB (183 s, compiling `app`), so the linear estimates of 11.7 GB and of 28 GB for `full` are low.

Dev edits, medians of 3, `enum`. Check at 2,044 / 4,079: body 0.37 / 0.61 s, subscription 0.44 / 0.91 s, add 15.4 / 33.7 s, library 7.8 / 23.3 s. Build at 4,079: first 50.5 s, 7.5 GB; body 3.3 s; subscription 14.9 s (first run 40.2 s); library 19.8 s; add killed at 8 GB. Build at 2,044: first 24.2 s, 3.7 GB; add 20.8 s, 4.4 GB. The check library edit grows 3.1x; `memset` goes from about 1.0 s to about 6.9 s and accounts for about 78% of the excess (callers not recorded).

rustc: page-fault regression bisected to rust-lang/rust#159115 (`c7d7da342`, parent 730k faults, it 2.43M on the reproducer). Not filed; draft is with the user. Reproducer generator: `target/probe-ex/rr/repro.py` (scratch).

## Release at `perps` and `full`, plan limits, rustc fix (2026-11-02)

`perps` release, full profile. 3ee3e0a: 4,348 s, 27.17 GB. After outlining switch leaf arms and construction, and release-only `inline(always)` on lookups (42f26e3, ad0b0d2): 553.6 s, 24.35 GB. After the storage descent (3804207, 8253688): 537.2 s, 12.08 GB. Framework min 117.86 ns vs 117.53 ns before the descent (6 alternating runs, cpu 10), under the threshold. Typical after the first two commits: gap 5.7 to 3.9 ns, build 28.2 to 21.1 s.

Descent, other sizes: typical 1.06 to 0.93 GB, +0.14 ns; 4,079 3.50 to 2.94 GB, +0.95 ns (consistent across two sessions); 6,114 6.17 to 4.33 GB. Not byte-identical at typical (104 bytes of `.text`, in the `Finish` functions).

Gap to hand-written code, `enum`: 823 3.9 ns, 4,079 9.1 ns, `perps` 26 to 27 ns. An agent's earlier split at `perps`: `--keys 64` 7.4, `--keys 179` 17.8, full keys 30, `--sort` 12.4, so most of it appears only when keys vary. No hardware counters at `perps` (`perf_event_paranoid` reset to 4 after restart). On `znver3` 4,079 gains about 1.3 ns; `+prefer-256-bit` nothing.

`full` (generator gives 30,186 components): plan limits raised (33661c2). Cold check with patched rustc, empty `MALLOC_CONF`: 310 s, 10.1 GB, 4.8M faults. Release on 2ebfcdd, 26 GB cap: type check 169 s, 8.4 GB; front end ends at 250 s, 20 GB; RSS falls to about 5 GB; LLVM climbs about 150 MB/s and the rustc was OOM-killed at 432 s (`anon-rss` 27.2 GB), still climbing. At `perps` the same climb goes from 4.4 to 11.8 GB in about 40 s and then stays flat.

rustc: `MAX_POOLED_RECORDERS` 4 to 256 in `compiler/rustc_middle/src/dep_graph/edges.rs` (local stage 1, single runs, default allocator): minimal reproducer 953k to 164k faults; 4,079 cold 54.7-58.3 s, about 12M to 35.2 s, 0.84M, +90 MB; library edit 71.5 s, 37M to 9.05 s, 0.57M. Minimal reproducer: `target/probe-ex/rr/minimal/repro.sh` (scratch). Not filed. Whether a smaller pool suffices and rustc's `tests/incremental` are still open. `muzzy_decay_ms:-1` alone is not a workaround at `perps`.

## Shared tails, wide components, first `full` release (2026-11-02)

Shared tail (79a4241, `PG_TAIL`, `exec_tail`): plain runs that end every branch of a switch compile once. 4,079: framework 97.0 to 93.0 ns, gap 10.7 to 6.7; `perps`: framework min 116.81 to 109.33 (6 alternating runs, cpu 9), gap about 27.5 to about 17, 537 to 472 s, 12.08 to 8.64 GB. It didn't change the climb at `full`.

Wide components (d9d4423, `WIDE_INPUTS` = 24, `run_wide`): the climb was the AlwaysInliner; the IR after it grew quadratically (9.3M, 18.4M, 30.3M lines at 4,964, 7,447, 9,929 components) because the gather of wide sinks was inlined into every arm and tail. LLVM stage peak at 12,385 components 7.2 to 2.95 GB. `perps`: framework min 110.01 to 96.41 (6 alternating runs, cpu 9), baseline min 92.40 and 89.72 in the two binaries, per-round gap 14.8-19.8 to 4.0-7.0, 472 to 416 s, peak 8.64 to 8.75 GB (front end). 4,079: 91.66 to 91.43, no gain there, so the gain only shows at scale. Tests 84 ok, error cases 58 ok on d9d4423.

`full` release (30,186 components, d9d4423, 26 GB cap): first finished build, 1,536.7 s, 19.66 GB. App crate: type check 168 s, 8.4 GB; codegen ends at 247 s, 20 GB; LLVM 497 s, near 7 GB. Runner: fat LTO 602 s. 4 runs on cpu 10: framework min 111.02, baseline min 102.49, gap about 8.5; the baseline slows more than the framework from `perps` to `full`, reason unknown. Still superlinear: ArgumentPromotion at about n^1.9 (6.2, 11.4, 34 s at 4,964, 7,447, 12,385).

rustc report: `MAX_POOLED_RECORDERS` of 64 matches 256 (max live 42 and 26 on two workloads); `tests/incremental` 180 of 180 with and without it. Draft at `target/probe-ex/rr/final/REPORT.md` (scratch). Not filed.

Gap on master d256616 (one binary per size, 6 runs, cpu 9): typical framework 101.01, baseline 101.73, about -0.7; 4,079 91.34 vs 84.16, 7.2. Build typical 20.8 s, 0.90 GB; 4,079 92.3 s, 2.95 GB.

Layout test (`BASE_SOA` in `gen_app.py`, 4,079, baseline only): default 85.1, two arrays like the framework 85.9, one array per field 103.5. Storage is already in execution order, so reordering can win about 1 ns at most.

Attribution of the 7.2 ns at 4,079 (cachegrind on an `x86-64-v3` build, framework / baseline per packet): instructions 708 / 749, stores 63.4 / 49.4, L1d read misses 4.07 / 2.06, L1i misses 15.6 / 27.5, conditional mispredicts 2.56 / 5.74, one indirect branch per packet on both. So instruction count, conditional branches and an extra jump level aren't the cause. `--sort` leaves 3.4 ns with everything hot; `--keys 8` 1.5, 32 3.1, 128 4.7, 300 7. So about 3.6 ns is data footprint and about 3.4 is a floor that lines up with the extra stores; 1 to 2 ns of the floor is unexplained. No fix found within the invariants. The earlier "gap grows with distinct packets" was a bump at 4k to 8k packets where both sides slow: gap 5, 7, 9.5, 10, 7.4, 6.9 at 1k to 64k. Next if wanted: count distinct cache lines per packet on each side, diff the stores of one hot arm. No hardware counters yet (`perf_event_paranoid`=4).

## API: handle components, every entry has on_update, rewrap (2026-11-02)

On master be2c3dc: `#[handle]` and `HandleComponent` (`Handle`, `View<'a>`, `view`), `on_update(&mut self, input, v)` on every entry with `#[derive(UpdateHandler)]` and `ACTIVE`, selectors on handle sources through `SelectView`. By-value `.text` byte-identical at typical and 4,079 through all of it. Tests 107 ok, error cases 73 ok.

Chain, 64 links, by value / handle / hand-written in place: 64 B 103 / 99 / 98 ns; 512 B 274 / 272 / 274; 4 KB 5,480 / 2,513 / 2,548. Switch on a handle source, hand-written / by value / handle: 7 feeds 8.0 / 9.8 / 9.5 ns; 15 feeds 17.4 / 23.4 / 23.5 (instructions +2.5%; gap not explained).

`perps` release on be2c3dc: 416.3 s, 9.06 GB; 6 runs on cpu 9, framework min 96.35, baseline min 91.33, per-run gap 2.9 to 5.7. `full` cold check on afca850: 359 s, 11.53 GB. `full` release not rebuilt since d9d4423: only about 20.7 GB is free with the desktop session running, and it needs about 21 GB or more.

Zero-cost survey at 4,079 (branch `zero-cost`, not merged): no flags, guarded runs or stored slots in generated apps. Extra stores are mostly call overhead from outlined functions, and removing them doesn't change time. At tiny the gap was the baseline's `#[inline(always)]` on `big_digest` that `Big::update` lacks; at 4,079 it follows the wide components through `run_wide`. Rebuilding the same baseline code moves it 2 to 4 ns from layout alone.

One-`Component` prototype: branch `one-component`, patch against 3db775e. Not adopted; the user decides.

## Small graphs and a fair benchmark (2026-11-02)

`gen_app.py` now gives `Sender`, `Risk`, `Big` and their hand-written counterparts the same `#[inline(always)]` (`--user-inline always|none`, default `always`), so earlier framework-vs-baseline numbers from it are superseded. Gap on master before the arm change, then after: tiny +1.9 then +0.7 ns, small +0.9 then +0.3, typical -0.5 then -1.0, 4,079 about +6.3 both (`.text` identical there). Old generator: tiny +4.5, typical -0.8, 4,079 +6.1.

Arms of one plain run, optionally with one more run, are inlined at up to 1,000 components (`prog_tiny_arm`, `TINY_ARM_MAX_COMPONENTS`): `gen_handle_select.py` value direct 7 feeds 9.8-10.1 to 7.7 ns (hand-written 8.2-8.5), value join 15 feeds 23.6 to 19.2-19.7 (17.5). At 4,079 inlining them cost +1.6 ns, hence the limit; `.text` above the limit is byte-identical to master.

Handle source gap is stores, from hardware counters (`perf`, paranoid 1): same retired ops, 51.3 vs 43.6 cycles per push, store-queue stalls about 4x. Adding only the handle store to the hand-written loop reproduces the whole gap, about 0.2 ns per store. These stores are dead when only same-push readers use them. Not fixed; needs handles passed without the `outs` tree, after flag removal. 4,079's gap isn't explained.

## One Component trait (2026-11-02)

`270680b`: `#[handle]`, `#[listener]`, `HandleComponent` and the user's `Listener` trait are gone; one `Component` with `Output`, `Handle`, `View<'a>`, `Requests`, `view`. By value or handle is decided by type (`OutInfo`: the default `Output` is a hidden unsized struct), because a const fails for generic user components (E0283/E0284). `.text` byte-identical at typical and 4,079. Against 7aabc89: typical check -2% time, -3.5% memory, release +0.4% memory; 4,079 check +7.9% memory (cause not found), release +1.4% memory. `full` cold check 367 s, 11.29 GB (359 s, 11.53 GB before, so the 4,079 increase doesn't show there). Tests 109 ok, error cases 77 ok. Worse errors: a `Node` bound on a handle component is E0271 naming the hidden type; cycles name `Anchor<..>: Planned`, not a component. A missing `view` is a post-mono error at any full build.

## The 4,079 gap was AVX-512 gathers (2026-11-02)

Hardware counters at 4,079 (`perf`, separate framework-only and baseline-only builds, per packet, framework / baseline): default flags cycles 497 / 463, ops 985 / 789, microcoded instructions 6.1 / 0.001, microcode ops 280 / 0.1. LLVM turns the wide components' constant-offset loads into `vpgatherdq` under `-Ctarget-cpu=native` on Zen 4. `-Ctarget-feature=+prefer-no-gather`: cycles 457 / 462, ops 752 / 790. Agent's runs, ns framework / baseline: tiny 44.4 / 43.3, small 84.0 / 86.0, typical 100.8 / 103.6, 4,079 83.7 / 85.6, `perps` 90.0 / 90.1. My rerun on ccb6149 plus the flag, repo config only, 6 runs on cpu 10: typical 100.07 / 103.05, per-run gap -2.8 to -4.3; 4,079 83.09 / 85.31, per-run gap -2.1 to -2.5; build 97 s, 3.10 GB. Why the hand-written side doesn't gather is unknown. `gen_handler.py` and `gen_enum_keys.py` don't set the flag yet. Store stalls are nil at this size, unlike the 7 and 15 feed handle graphs.

## Why a release build uses one core (2026-11-02)

Typical release uses 1.04 cores with 8 jobs allowed. `app` at 4,079 (53 s): type checking 19.8 s, metadata 7.8 s, monomorphization 6.9 s, LLVM 22.5 s. `runner` (40.5 s) type-checks again for 19.4 s, because constant evaluation isn't reused across crates (typical: 143,141 evaluations in 3.0 s in `app`, 123,078 in 2.95 s in `runner`; a small crate confirms it, 13 s downstream of a dependency that already evaluated the constant). `PLAN` at typical is 2,808 ms: `build_program` about 1,640 (59%), the walk about 810, `HASH` about 360 (13%), `number_components` and `assign_slots` about 0. Measured by skipping stages in a scratch copy, reading `PLAN`'s time from a profile with query keys; the cut-down builds fail later but still evaluate `PLAN`.

Setups, typical, wall s and framework / baseline ns: current 23.0, 100.4 / 103.4; `-Zthreads=8` 21.8, 100.8 / 103.5; ThinLTO + 16 units 17.9, 104.8 / 104.0; no LTO + 16 units 17.5, 111.8 / 104.4. Fat LTO + 16 units (cold build, no timing split): typical 21.6 to 19.3 s, 0.93 to 1.05 GB, 100.4 to 99.7 ns; 4,079 96.0 to 81.9 s, 3.10 to 3.55 GB, 84.7 to 83.8 ns; `perps` 367 s, 10.37 GB, 91.5 / 93.1 (no same-commit comparison; older runs 416 s, 9.06 GB). Pipeline named only in the binary crate: 4,079 79.7 s, 3.52 GB, 86.0 ns, and 65.9 s, 3.98 GB, 84.2 ns with 16 units. Not adopted yet: `codegen-units = 16` is the user's decision. Newtype around the pipeline: `runner` type checking 19.4 to 0.008 s, build 102.5 s, so no gain.

Parallel constants: rustc's parallel loop covers top-level items only. 32 constants of about 0.4 s each read by one big constant: 8 threads 14.1 s if only the big one is an item, 5.1 s (30 s CPU) if each is an item, 4.9 s with the big one first in the file, 5.0 s with each in its own module. Sub-plans per module were rejected: only about 40% of `PLAN` could split, at most about 8 s at 4,079 if the split holds there, it needs `-Zthreads` and wiring syntax, and modules in dependency crates gain nothing.

Incremental: an unrelated function appended after the constant costs nothing; added before it in the same crate re-evaluates it (13.3 s); a no-change rustc run loses the cached value for the next edit (13.1 s; Cargo doesn't run rustc in that case). Dev edits at typical, profiled, single runs: `runner` 0.3 s; `app/src/risk.rs` 2.4 s and `marketlib/src/lib.rs` 2.4 s, plan reused; `app/src/lib.rs`, which holds `Pipe` and `build`, 7.6 and 11.6 s, plan re-evaluated in `runner` both times and in `app` the second time.

## Set subscriptions (2026-11-02)

`name: for S in Catalog { Node<S> }`, `SetIn` with `get`, `get_ref`, `fold`, `for_each`, `len`. My checks on rebased 33417d7: `sets`, `handlers`, `update_handlers`, `order`, `lifecycle`, `wide`, `generic_user` pass; 82 error cases ok. `.text` of apps without sets is byte-identical to a549f94 (library swapped in one directory): typical 762,006 bytes, 4,079 2,348,038 bytes; builds 21.5 to 21.9 s and 95.0 to 96.7 s. 4,079 run time 84.55 / 85.29 ns. `perps` with Risk and the Senders as sets read with `get`: 436 s, 9.09 GB, 91.69 / 93.03 ns. Agent's numbers not rerun are in the README. Reading all elements with `fold` is measured only at 2,044: the baseline with 300-term sums doesn't build within the 10-minute limit at 4,079.

## Plan evaluation halved (2026-11-02)

`PLAN` at typical: about 2,665 to 1,340 ms (agent's profile, 3 runs each). `build_program` wasn't slow by algorithm: about 244 `emit` calls, 84 switches, about 4,500 `consistent_static` calls, but each interpreted function call costs tens of microseconds (one `consistent_static` loop: 81 µs per call before, 31 after). Fixes: one pass per term instead of several helper calls (`term_status`, `live_dec_at`), inlined candidate cursors, `seg_ends`, keys stored in `ib` during `emit_switch`, and `name_hash` over the length, the last 24 bytes and 24 sampled bytes. My checks on rebased 64082cc, swap in one directory: best of 3 cold builds, typical 22.31 to 19.44 s, 4,079 95.69 to 86.24 s, memory unchanged (3.10 GB); `.text` identical at both; 4,079 run time 84.85 / 85.83 ns; 14 test targets and 82 error cases pass. Remaining at typical: about 867 ms for the walk and filling `Plan`'s arrays (about 7M zeroed elements), about 500 ms left in `build_program`. Probe lengths of the new hash weren't measured directly.

## Why dev edits re-evaluate the plan (2026-11-02)

Source read at rustc 92b5b8c8. `PLAN` is recomputed in a crate iff (A) this compile needs it and (B) the crate's previous compile didn't leave it in the on-disk cache.

(B): `eval_to_allocation_raw` is `cache_on_disk`, but its key is `KeyFingerprintStyle::Opaque` (blanket default, `dep_node_key.rs`), so `dep_kind_vtables.rs` gives it no `promote_from_disk_fn`. `exec_cache_promotions` (`graph.rs`) promotes only nodes with one, and `encode_query_values` (`incremental.rs`) writes what's in memory. A compile that doesn't load the value drops it. The next demand finds nothing to load in `load_from_disk_or_invoke_provider_green` (`execution.rs`) and runs the provider. `runner`'s `query-cache.bin`: 32 MB after computing, 355 KB after loading, 156 KB after a comment-only compile.

(A), `runner`: a changed dependency hash makes `crate_hash` red. Extern `implementations_of_trait((CrateNum, DefId))` is Opaque, so `try_force_from_dep_node` fails and `try_mark_previous_green` returns `None`, and the dependent re-runs. Small crate: `x.clone()` and `x.len()` re-type-check, `x + 1` and a plain function don't, `implementations_of_trait` re-runs 46 times; a function calling the dependency's own trait doesn't re-type-check, unexplained. `typeck_root(main)` holds the pipeline by value, so it needs the layout: `eval_to_valtree(drive::{impl#3}::{constant#0})`, then `flat::N`, then `PLAN`. (A), `app`: `codegen_crate`, `fn_abi_of_instance(app::build)`, `layout_of(Sinks::Store)`, on re-codegen of `build`'s unit, so on a non-comment edit to the file holding it. The failing node wasn't seen directly (no debug logging in release rustc); its identity is inferred. That `PLAN`'s node stays green in a skipped compile is inferred from the source and unchanged node counts.

Sequence at typical (faster plan): comment 0.78 s; function in `lighter` 4.34 s (recomputed); again 2.23 s (loaded); comments 0.81 s; function 4.31 s (recomputed); `risk.rs` 2.26 s, `lighter` 2.29 s (loaded); comment 0.82 s. An earlier 15.5 s comment was my test setup: `RUSTC_BOOTSTRAP` differed between shell calls and `unstable_features` is `[TRACKED]`, which discards the cache (11.8 s each flip, 0.8 s otherwise).

Avoiding the need, agent's scratch copy, before the plan got faster, not rerun by me: `pub struct App(Box<Pipe>)` with the wiring in `app/src/pipe.rs` gives 1.2 s for any edit outside `pipe.rs` (3.2 to 11 s before) and 6.1 s for `pipe.rs`. A newtype without the `Box` doesn't help. Run-time cost and 4,079 not measured.

## Fired flags removed (2026-11-03)

No stored bit at run time: an edge fired by code position, or by asking again from the pushed root and stored outputs (`guard.rs`, `PG_IF`, `PG_GATE`). My checks on 51b0313 against 3a237a5, library swapped in one directory: `.text` byte-identical at typical, 4,079 and `perps` (6,685,526 bytes), so apps with no active handler are unaffected; builds 19.03 to 19.52 s, 81.76 to 82.82 s, 374.5 to 377.6 s; memory 0.90 to 0.92, 3.09 to 3.08, 9.10 to 9.10 GB; `perps` run time 89.64 / 91.32 and 89.72 / 92.57 ns. 18 test targets and 82 error cases pass; agent reports all 21 targets, doc tests, examples and differential seeds 1 to 4 identical to master. Chain with a handler, agent's numbers before the final rebase, flags against guards, ns: 10 links 1.23 / 1.53, 20 1.27 / 2.08, 30 1.54 / 6.24, 40 1.82 / 5.96; 8 independent guards improved earlier, 5.25 to 2.84. Recursion limit: 88 links need 137-138 against 131-134. `PLAN` ms, branch / master: 20 links 265 / 243, 60 661 / 432, 88 1,386 / 775, 120 2,936 / 1,539; `long_running_const_eval` trips between 120 and 160 links, master not at 160. Still roughly quadratic because terms carry every ancestor's decision. `share_tail`'s gate check is conservative: the new test passes without it.
