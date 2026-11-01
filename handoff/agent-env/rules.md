RULES (from the user; they override defaults; read all of them)

Machine: cloud container, 4 cores, ~14 GB usable RAM for everyone, no swap, no systemd. Other agents
run builds at the same time as you.

1. EVERY cargo/rustc/bench command goes through the cap wrapper, never bare:
     /tmp/claude-0/-home-user-repo/8e981b07-5287-54fa-a02f-4121b77bfce6/scratchpad/cap.sh <build|measure> <CAP> <SECONDS> <cmd...>
   e.g. .../cap.sh build 4G 600 cargo test
   - CAP: your brief gives your `build` cap. Never exceed it. A `measure` run may use up to 10G (no other
     build runs during it), but only when the measurement needs it. An OOM (rc=137 with peak≈cap) is a RESULT to
     record, never a reason to retry with a bigger cap or uncapped.
   - `build` = shared machine lock, CARGO_BUILD_JOBS=2. Use for correctness work (tests, checks, examples).
   - `measure` = exclusive machine lock (waits until no other agent builds), CARGO_BUILD_JOBS=4. Use for
     EVERY timing you report (build time, run time, rebuild time). Keep measure runs short and batched:
     while you hold it, everyone else waits. Never run a `measure` from inside a `build` or vice versa.
   - One build at a time per agent. The wrapper prints `cap.sh: rc= peak=` on stderr: record peaks.
   - Pin run-time benchmarks with `taskset -c 2` (inside the wrapper).
2. Tokens: you are a single-milestone agent. Hand back when the milestone is done, or when your context
   reaches ~120k tokens, whichever is first, with a 20–30-line report (what changed, commits, numbers,
   what's left, exact next step). Don't continue past that to "one more thing".
   - No polling, no sleep loops. Run long builds with run_in_background and wait for the completion notice.
   - All command output goes to files (in your scratch dir below); read it with grep/tail, never cat a
     whole build log into context.
   - Don't re-read big files you already read; read only the parts you need.
3. Paths: use literal absolute paths in commands (no $VAR paths; the worktree guard refuses them).
   Scratch dir for logs/scripts: /tmp/claude-0/-home-user-repo/8e981b07-5287-54fa-a02f-4121b77bfce6/scratchpad/<your-agent-name>/
4. Git: work only on your own branch in your own worktree. Commit as you go (small commits). No Claude
   attribution lines and no signing in commits. Push your branch with `git push -u origin <branch>`
   (retry on network failure 4x with 2/4/8/16 s backoff). Never push to master, never force-push,
   never delete branches or files you didn't create without a dry-run first.
5. Project invariants that must still hold: #![forbid(unsafe_code)]; no dynamic dispatch at run time;
   plan constants are integer arrays; component identity is TypeId via table.rs; generic_const_exprs only
   inside the library (user crates need no #![feature]); scripts/check_errors.sh must pass (30 cases).
   Root README.md / CLAUDE.md describe the OLD model; the current model and state are in
   handoff/README.md and handoff/event-spec.md (read the sections your brief points to, not everything).
6. Optimization priorities (user's order): (1) release run time, (2) release build of the typical app
   under 1 min, (3) dev incremental rebuild after editing the big component plus a few small ones.
   A change that trades one of these for another stays on its branch and is reported, not merged.
7. Check claims by building small artifacts and measuring, not by reasoning. Report numbers with the
   exact command that produced them. Never invent or estimate a number you didn't measure.
8. Disk: the session has a fixed disk allowance (~39 GB in total, shared by every agent; it filled up
   once). Keep your scratch dir plus your worktree's target/ under 6 GB: delete generated app
   workspaces and their target dirs as soon as you've recorded their numbers, prefer one master
   checkout reused across measurements, and check `du -sh` of your dirs before big builds. On
   "no space left on device", delete only your own build output and report it; never delete other agents' files.
