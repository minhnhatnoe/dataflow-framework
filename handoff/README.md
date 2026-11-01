# Handoff: state of the dataflow framework work (2026-11-01)

The root `README.md` and `CLAUDE.md` still describe the **old** model. This file and `handoff/event-spec.md` are the current picture.

`event-spec.md` is the source of truth for the model. It was copied here from the session scratchpad in `/tmp`, which doesn't survive a reboot.

Contents:
1. [The model as built](#1-the-model-as-built)
2. [Rules and preferences](#2-rules-and-preferences)
3. [What is on master](#3-what-is-on-master-df22e3d)
4. [Branches](#4-branches)
5. [Measurements](#5-measurements)
6. [Pending work](#6-pending-work-in-order)
7. [Open design questions](#7-open-design-questions)
8. [Stashed](#8-stashed-not-to-be-built-unless-asked)
9. [Tooling](#9-tooling)

---

## 1. The model as built

Event-driven and push-based, with a single root.

**The root.**
- The user defines it with `#[derive(Root)]` and feeds it with `p.push(value)`.
- One push is one step.
- The anchor type is `Pipeline<R, Sinks>`, where `Sinks` is a tuple of up to 7 sink components.

**Components.**
- `#[derive(ComponentSubscription)]` with `#[subscribe(name: Type, ...)]`.
- `new(config, init) -> (Self, Output)` receives the initial outputs of its subscriptions.
- `update(&mut self, input) -> Output`.
- **Outputs are stored and persist between steps** (the "push value" model).
- The framework never invents `Option<&T>`. A component that wants "no value yet" sets `Output = Option<T>` itself.

**Firing.**
- **OR rule:** a component runs if any upstream fired this step. A component that runs always fires.
- **Order:** post-order within a step.
- Only the affected cone runs. It is computed at compile time and dispatched through a decision trie.

**Demuxing lives on subscription edges**, not in nodes.
- Example: `#[subscribe(upd: BinanceWs => Some, payload::Depth, On<Btc>)]`.
- **Selectors** are ZSTs implementing `Select<In> { type Out; const KEY: u32; fn select; fn key_of }`.
- `#[derive(Demux)]` generates one selector per enum variant. `Some`/`None` are built in. Guard selectors are written by hand (`KEY = GUARD = u32::MAX`).
- **Joins** absorb implied terms. Whatever remains must be pairwise exclusive (same trie place, different key), otherwise it is a compile error.
- **Rule 5:** a selected edge's value must be implied by the reader's condition.

**Merged listeners.**
- `Symbol { KEY, NAME }`, `Keyed`, and the selector `On<S>`.
- A component marked `#[listener]` gets `Listener::subscribe(&keys)` once at build time, with the deduplicated keys the graph requests. So one websocket carries every symbol the graph uses.

**Handler components** are an alternative definition style that coexists with `Component`.
- Written as `#[derive(Handler)]` with `#[handles(Quoter<Btc>, Quoter<Eth>, ...)]` and `#[subscribe(...)]` for plain inputs.
- They implement `Collect { new, begin -> Buf, finish(buf, input) -> Output }` and `UpdateHandler<E>::on_update(&mut self, buf, v)`.
- Only edges that fired are delivered.

**Compile-time plan.**
- One const evaluation of integer arrays (`Planned::PLAN`): decisions, terms, conditions, flags, listener keys.
- From those it builds the push program: `Seq`, `Run`, `RunIf`, `Switch`, plus `PG_HRUN` for handlers.
- A `Switch` becomes a binary decision tree over sorted keys (`BstExec`), which LLVM turns into jump tables.
- Plan errors are deferred (`Plan::fail` / `Plan::check` / `Planned::CHECK`). A failed plan gets an empty program, because a const panic inside the plan once caused a 13 GB hang lasting minutes.

**Storage.**
- Flat balanced 8-ary tuples indexed by execution position, with `Nth` halving descent.
- `inline(always)` is release-only: `#[cfg_attr(not(debug_assertions), inline(always))]`.

## 2. Rules and preferences

**From the user:**
- Optimization priorities, in order:
  1. release run time;
  2. release build of the typical app (one NIC, several exchanges × symbols, hundreds of components, mostly ≤ 7 subscriptions each, a few big ones) under 1 min;
  3. dev incremental rebuild after editing the big component (in its own file) plus a few small ones.
- **Do every optimization task before any other kind of work** (docs, features). Said 2026-11-01.
- The main Opus agent never writes code. All code changes go to Sonnet subagents. The main agent reads, measures, designs and verifies.
- Commit work to master directly; experiments go on their own branches. Changes with trade-offs stay on a branch and are reported.
- No Claude attribution in commits, no commit signing.
- Dry-run before anything that deletes or overwrites.

**RAM.** The machine has 30 GB and the desktop uses about 7 GB. An uncapped rustc once triggered the OOM killer, and systemd then killed VS Code, Claude Code and every agent.
- Every build runs as `systemd-run --user --scope -q -p MemoryMax=<cap> -p MemorySwapMax=0 -- timeout <s> ...`, with `CARGO_BUILD_JOBS=4` and one build at a time per agent.
- The caps of concurrent agents sum to about 23 GB or less.
- An OOM inside a scope is a result to record, never a reason to retry uncapped.

**Tokens** (also in `~/.claude/CLAUDE.md`):
- one milestone per agent, handing back at about 120k tokens of context with a 20–30-line report;
- no polling: background builds plus the completion notification;
- output goes to files, read with grep or tail;
- write the rules into every brief, because agents started in an old session don't see CLAUDE.md edits.

**Project invariants that still hold:**
- `#![forbid(unsafe_code)]`;
- no dynamic dispatch at run time;
- plan constants are integer arrays;
- component identity is `TypeId` via `table.rs`;
- `generic_const_exprs` is confined to the library;
- `scripts/check_errors.sh` must pass (30 cases on master).

## 3. What is on master (`df22e3d`)

| Commit | What |
|---|---|
| 772fac7, b2891dc | Event model phase 1a (root, components, sinks, push, stored outputs) and phase 1b (selector chains, `Demux`, conditions, conditional push) |
| 57db1a4 / 9474507 | Keyed `On<S>`, `Symbol`/`Keyed`, `#[listener]` requests collected by the plan |
| 29bccbb | Exchanges example: NIC, two exchanges, merged listeners, exclusive joins |
| 57e047e / f39fd10 | Benchmark harness: `bench/gen_app.py`, a generated multi-crate app plus a fair hand-written baseline |
| fdf6f71 … 215d879 | Trie dispatch: the push program is built in the plan, with jump-table switches |
| 869a797, edd5783 / 7ff5191 | Handler components; `MAX_TERMS` raised to 256 |
| 5dca8e7, d395308, 0411655, 7aae9c1, f42a3ea | scale-typical work, listed below |
| 87bd967 / df22e3d | stable-spans: the derives re-span generated tokens to the struct ident |

The scale-typical commits:
- the derive no longer generates a per-component module with a glob import, which made name resolution quadratic;
- caps raised to 32k components;
- hashed decision tables;
- sub-quadratic absorption and exclusivity checks;
- `Fired<N>` sized by the plan's highest needed flag;
- `gen_app.py` adds `#![allow(long_running_const_eval)]` to the app and runner crates.

Verified on master: `cargo test`, `check_errors.sh` 30/30, and the exchanges example.

## 4. Branches

| Branch | Head | State |
|---|---|---|
| `plan-linear` | 08966ac, plus **uncommitted** `plan.rs` changes (+163/−16) in worktree `.claude/worktrees/agent-a7f1e4b41c96a31c8` | Linear push-program builder. Details below. **Not merged.** |
| `runtime-gap` | 05a05da | `gen_app.py --big-inline`, a user-side hint. Not merged; it hurts typical by about 2 ns. |
| `runtime-gap3` | df22e3d, plus **uncommitted** `gen_app.py` changes (+8/−3) in worktree `.claude/worktrees/agent-ad6370a9d202b7f4b` | Run-time investigation. The agent was interrupted. The diff is probably the "cores build a local" harness change. |
| `scale-30k` | cc27cb6 (on an older master) | `Blk<R,M,LO,HI>` named storage nodes. Details below. |
| `build-cost` | f42a3ea | No changes. The span work moved to `stable-spans`, which is merged. |
| `flat-experiment`, `shape-experiment`, `trie-dispatch`, `keyed-listener`, `exchanges-example`, `bench-harness`, `handlers`, `scale-typical`, `stable-spans` | | Old or already merged |

**`plan-linear`:**
- Changes:
  - a per-place context table makes `held`/`conflicts` O(1);
  - candidate terms are carried into switch branches;
  - `find_decision` is hashed;
  - the allow is removed from `gen_app.py`;
  - a `program_shapes_are_stable` digest test.
- Results:
  - typical builds without the allow (release 19.4 s; to check against about 15.5 s before);
  - gen_handler N=256 component: 33.7 → 27.4 s;
  - N=256 handler: 23.7 → 21.2 s;
  - N=512 component builds (369 s, almost all codegen);
  - N=512 handler still trips the lint in `mk_hrun` → `ran_state`.
- `cargo test` passed at 08966ac.
- The uncommitted diff is the follow-up agent's work in progress: the `mk_hrun` fix and/or verification.

**`scale-30k` (`Blk` storage):**
- Gains:
  - release peak memory at 4k components 12.9 → 6.0 GB;
  - at typical, dev initial build 16.7 → 9.8 s and edits 2.4 → 0.8 s.
- Regression: incremental `cargo check` on a synthetic 3k graph, 31 s → over 110 s, cause unknown. Measured before the stable-spans fix, so it may be the same span problem; re-measure.

## 5. Measurements

Harness: `bench/gen_app.py`. The typical app has 823 components, small 415, tiny 111. Times are ns/packet, pinned with `taskset -c 2`.

| Case | Framework | Hand-written | Notes |
|---|---|---|---|
| tiny, on mode | 55–57 | 47.5–48 | 1.17× |
| tiny, `Big` inlined | 50.5–51 | 47.3 | `Big::update` not inlined costs about 4.5 ns at tiny only |
| small, on/enum | 93–95 | 85.5 | |
| typical, on/enum | 112–115 | 100.7–103 | 1.12×; was 1.4× before trie dispatch |
| gen_handler N=8/64/128, handler | 4.7 / 9.0 / 10.3 | 4.65 / 5.2 / 5.2 | an ordinary component sink: 5.4 / 36.6 / 56.0 |

**Run-time findings so far:**
- Instruction counts are equal (callgrind: tiny 595 vs 603, typical 1071 vs 1090). The dispatch is jump tables; `fired` flags and bounds checks are not the cause. So the gap is latency.
- `perf` is unavailable (`perf_event_paranoid=4`). valgrind, objdump and gdb work.
- On tiny, trade packets carry the gap, even for a single predictable key (binance key2 trade +8.4 ns, depth +2.2).
- **Store-forwarding stall.**
  - Harness cores do `self.top.x += ..; self.top`: scalar stores, then a 16-byte load when the output is copied into its slot.
  - Rewriting a core to build a local closes only about 1.4 ns at tiny, and the baseline gains 1.1 ns from the same rewrite.
  - So the stall is real but small, and **most of the gap is still unexplained**.
- No `target-cpu` is set anywhere, so builds target generic x86-64.

**Build:**
- typical release build about 15.5 s (app crate); the runner build about 23–25 s;
- typical dev edit rebuild 1.8–2.3 s;
- after stable-spans (m10-80-20, 1,129 components, `cargo check`): a declaration shift 11.8 → 0.22 s, an edit inside `#[subscribe]` 13.2 → 0.20 s.

**Plan const eval:** before plan-linear, the typical plan passed rustc's `long_running_const_eval` limit of about 2M steps. The cost was N² scans in the program builder, mainly `conflicts`: 545k inner iterations at typical.

**Scale limits on master:**
- m15-150-35 (2,044 components) and m30-300-70 (4,079) fail with E0080 "reached the configured maximum number of stack frames" in `Anchor<Wire,Sinks>::PLAN`.
- 12,190 components was killed at 18 GB of memory (before later fixes).
- 29,650 has never been run.
- Initial check scales mildly superlinearly (10.2 s at 823, 15.4 s at 1,129).

**Token usage, Oct 30 – Nov 1:**
- about 99% of tokens were cached context re-reads: 131 M in the main session (Opus), 343 M across 74 Sonnet subagents;
- long single-turn agents reached 300–380k of context;
- which led to the token rules in section 2.

## 6. Pending work, in order

Two agents were interrupted when the session ended:
- the `plan-linear` verifier (worktree `agent-a7f1e4b41c96a31c8`, uncommitted `plan.rs` diff);
- the `runtime-gap3` investigator (worktree `agent-ad6370a9d202b7f4b`, uncommitted `gen_app.py` diff).

Their transcripts are saved. Resume them, or inspect the diffs and continue with fresh agents.

1. **Finish `plan-linear`**, then merge.
   - Review the uncommitted diff.
   - Fix `mk_hrun`: bucket handled edges by key at the switch place, as candidate terms already are. Re-test handler N=256/512 without the allow.
   - Run `check_errors.sh` and the exchanges example. Check that typical enum mode builds without the allow.
   - Measure against master: typical release build time (19.4 s vs about 15.5 s, regression or a different measurement?), run time at tiny and typical in on/enum mode plus gen_handler 64, and dev rebuild with `scripts/measure.py`.
   - Then measure `collect_requests`, which is listeners × decisions.
2. **Run-time gap (`runtime-gap3`).** The user chose to keep the old API, `update(&mut self, input) -> Output`.
   1. Harness cores build a local on both sides, so the comparison stays fair.
   2. Break typical down per packet kind (the `kinds.rs` tool from earlier agents).
   3. Find the rest of the 12 ns: memory footprint, the separate `states`/`outs` arrays vs the baseline's per-market struct, D1/LL misses, listener `Frame` copies (about 2.5 ns at tiny), branch layout.
   4. Measure with `-C target-cpu=native` on both sides.
   5. Check whether X86AvoidSFB can see the pattern (are the stores and the copy in the same basic block?); try `-C llvm-args=-x86-disable-avoid-SFB`.
   6. **User request:** if the compiler is confirmed to mishandle this, write a minimal standalone repro in `bench/sfb/`: under about 100 lines, no framework dependency, field-wise `+=` then copy vs build-a-local, with the asm and numbers in a top comment.
3. **Mono items per component**, about 50–79 today (`-Zprint-mono-items`). Reduce them without hurting run time.
4. **Caching per-component resolution** (spec ideas 1–4).
   - Prove reuse with query counts before and after a body edit.
   - Non-generic cores and `-Zshare-generics`.
   - Dev settings.
   - The 22 plain `#[inline(always)]` in `gather.rs`/`select.rs`/`handler.rs` should become release-only unless debug needs them.
5. **`Blk` storage.** Cherry-pick `cc27cb6` onto master. Re-measure the incremental-check regression after stable-spans; keep `Blk` if the trade-off is gone.
6. **30k scale.** perps (12,190) and full (29,650) within an 18 GB cap: release time, peak memory, run time. This needs plan-linear, and probably the stack-frame limit fixed at 2k+ components. Fragments (spec idea 5) only if 1–5 aren't enough, on a separate branch.
7. **After all optimization:** rewrite `README.md` and `CLAUDE.md` for the new model (invariants 3, 4, 6 and 8 changed, plus the new rules), and move `event-spec.md` into the repo properly.
8. **Housekeeping.**
   - The hourly heartbeat cron job (063a3636) was in the old session. Check `CronList` and delete it when everything is done.
   - The headless restart job (a systemd timer running `claude --continue -p`) awaits the user's decision; option (a), a narrow allow-list, was suggested. Don't set it up without approval.
   - Remove stale worktrees under `.claude/worktrees/` once merged.

## 7. Open design questions

- **Run-time API (decided for now):** keep `update -> Output`. Two alternatives were discussed and are not adopted:
  - (A) `update(&mut self, input, out: &mut Output)`, writing into the output slot;
  - an `#[output]` field, so readers get `&self.field` straight into the component.

  Revisit only if the investigation shows the copy matters.
- **Large handlers:** a handler over 256+ edges is limited by codegen time; at N=512, about 369 s, mostly LLVM. Instance lists must be written out (no alias expansion in the derive).

## 8. Stashed (not to be built unless asked)

- zero-copy views, handles, buffer-owning components and retention;
- "which inputs fired" flags passed to components;
- set subscriptions;
- multiple roots;
- AllNic (dropped);
- FlatPipeline as-is (its layout was adopted into the redesign).

## 9. Tooling

**Repo scripts:**
- `scripts/check_errors.sh` (30 cases);
- `scripts/measure.py` (release, run and dev measurements on the harness);
- `scripts/gen_selbench.py` and `scripts/gen_keybench.py`;
- `bench/gen_app.py --size tiny|small|typical|perps|full|m<L>-<B>-<H> --mode 1a|enum|on [--lib CHECKOUT]`;
- `bench/gen_handler.py N OUT`.

**Scratch** (`/tmp/claude-1000/-home-noe-TGT-dataflow-framework/093ee37d-6bfd-475e-b7a4-1d29416bf0b3/scratchpad/`, which doesn't survive a reboot):
- `bc/s/chk3.sh` and `bc/t/` (incremental edit tables);
- `pl/` (plan profiling: `nb.sh`, `prof.patch`);
- `kinds/` (per-packet-kind timing);
- `cgsum.py` (callgrind summary);
- `tokens.py` / `tok2.py` (token-usage analysis of transcripts).

**Notes:**
- `cargo check` misses post-monomorphization errors, so take depth limits from real builds.
- This nightly writes self-profile format v9; no offline decoder reads it. Use `-Ztime-passes`.
- Use literal absolute paths in agent commands; `$VAR` paths were refused by the worktree guard.
