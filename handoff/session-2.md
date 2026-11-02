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
