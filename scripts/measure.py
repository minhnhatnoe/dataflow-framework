#!/usr/bin/env python3
"""Measurement harness for the generated market app (bench/gen_app.py).

usage: scripts/measure.py [--sizes tiny,small,typical] --mode 1a|enum|req [--lib CHECKOUT] [--out DIR] [--steps gen,release,run,dev,mono]
                          [--cpu 2] [--procs 5] [--cap-small 60] [--cap-big 600] [--sink-scan 8] [--keep-going]

For each size it generates the workspace under DIR/gen/<size>-<mode>/ and records CSVs under DIR/csv/:

  build_<size>-<mode>.csv   step, seconds, status (ok | >cap | error), note  (also the dev incremental sequence)
  phases_<size>-<mode>.csv  step, crate, phase, seconds   (-Ztime-passes; top phases of each timed build)
  mono_<size>-<mode>.csv    crate, mono_items            (-Zprint-mono-items=yes, release)
  run_<size>-<mode>.csv     side, proc, min_ns, median_ns, then summary rows (proc = all)
  meta_<size>-<mode>.csv    markets and component counts
and merges everything into DIR/csv/all_*.csv. Raw logs are in DIR/logs/.

Steps:
  gen      generate the workspace
  release  (b) prebuilt library deps + marketlib (timed), then a scratch release build of `app` (timed, -Ztime-passes)
  mono     (d) -Zprint-mono-items counts per crate (release; forces one extra rebuild of marketlib and app)
  run      (a) ns/packet, framework vs baseline, pinned with taskset, alternating rounds inside each process,
           `procs` processes (the protocol of scripts/bench.sh)
  dev      (c) debug-profile incremental rebuild of `app` after editing the big component plus small ones
           (their `let tweak ... // EDIT` lines): initial, no-op, big only (same length), big + 3 small (same
           length), big + 3 small (different length); each with -Ztime-passes -Zincremental-info

Caps: a build slower than --cap-small (tiny/small/typical) or --cap-big (perps/full, one attempt) is killed
and recorded as ">cap". A failed build records the first rustc error. If the framework side fails, the
baseline side is still built and run (baseline-only numbers).
Every cargo command runs under `timeout`. Wall times are noisy: other processes share the CPU.
"""
import argparse, csv, glob, os, re, statistics, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(HERE)
BIG = {"perps", "full"}


def sh(cmd, cwd, cap, log, env=None):
    """Run under `timeout cap`; returns (seconds, status, output). status: ok | >cap | error."""
    e = dict(os.environ, CARGO_TERM_COLOR="never", **(env or {}))
    t = time.time()
    p = subprocess.run(["timeout", "-k", "10", str(cap)] + cmd, cwd=cwd, env=e, capture_output=True, text=True)
    dt = time.time() - t
    out = p.stdout + p.stderr
    with open(log, "w") as f:
        f.write(f"$ {' '.join(cmd)}\n# wall {dt:.2f}s exit {p.returncode}\n\n{out}")
    if p.returncode in (124, 137):
        return dt, ">cap", out
    return dt, ("ok" if p.returncode == 0 else "error"), out


def first_error(out):
    """First rustc error line (message) and how many errors there were."""
    lines = out.splitlines()
    errs = [l for l in lines if re.match(r"error(\[E\d+\])?:", l) and "could not compile" not in l and "aborting" not in l]
    if not errs:
        return next((l for l in lines if l.startswith("error")), "")
    return f"{errs[0]} | {len(errs)} errors"


def phases(out, top=12):
    rows = []
    for l in out.splitlines():
        m = re.match(r"\s*time:\s+([\d.]+);.*\)\s+(\S+)\s*$", l)
        if m:
            rows.append((m.group(2), float(m.group(1))))
    rows.sort(key=lambda r: -r[1])
    return rows[:top]


class Rec:
    def __init__(self, outdir, tag):
        self.dir, self.tag = outdir, tag
        os.makedirs(f"{outdir}/csv", exist_ok=True)
        os.makedirs(f"{outdir}/logs", exist_ok=True)
        self.tables = {"build": [], "phases": [], "mono": [], "run": [], "meta": []}
        self.heads = {"build": "step,seconds,status,note", "phases": "step,crate,phase,seconds",
                      "mono": "crate,mono_items", "run": "side,proc,min_ns,median_ns", "meta": "key,value"}

    def add(self, table, *row):
        self.tables[table].append(row)
        self.flush(table)

    def flush(self, table):
        with open(f"{self.dir}/csv/{table}_{self.tag}.csv", "w", newline="") as f:
            w = csv.writer(f)
            w.writerow(self.heads[table].split(","))
            w.writerows(self.tables[table])


def merge(outdir):
    for table in ("build", "phases", "mono", "run", "meta"):
        rows, head = [], None
        for p in sorted(glob.glob(f"{outdir}/csv/{table}_*.csv")):
            tag = os.path.basename(p)[len(table) + 1:-4]
            r = list(csv.reader(open(p)))
            head = ["size-mode"] + r[0]
            rows += [[tag] + x for x in r[1:]]
        if head:
            with open(f"{outdir}/csv/all_{table}.csv", "w", newline="") as f:
                w = csv.writer(f)
                w.writerow(head)
                w.writerows(rows)


def edit(gen, files, n, longer):
    """Rewrite the `let tweak ... // EDIT` line of each file. Same length: a different 4-digit number.
    Longer: an expression."""
    for rel in files:
        p = f"{gen}/{rel}"
        s = open(p).read()
        val = f"{1000 + n} + {n}u64 * 3 - 1" if longer else f"{1000 + n}"
        s2 = re.sub(r"let tweak: u64 = [^;]*; // EDIT", f"let tweak: u64 = {val}; // EDIT", s)
        assert s2 != s or longer is None, p
        open(p, "w").write(s2)


def do_size(a, size):
    tag = f"{size}-{a.mode}"
    gen = f"{a.out}/gen/{tag}"
    logs = f"{a.out}/logs/{tag}"
    os.makedirs(logs, exist_ok=True)
    rec = Rec(a.out, tag)
    cap = a.cap_big if size in BIG else a.cap_small
    steps = a.steps.split(",")
    cargo = ["cargo"]
    off = ["--offline"]

    def timed(step, args, crate, env=None, log=None, cap_=None, profile_phases=False):
        dt, st, out = sh(cargo + args, gen, cap_ or cap, f"{logs}/{log or step}.log", env)
        note = first_error(out) if st == "error" else ""
        rec.add("build", step, f"{dt:.2f}" if st != ">cap" else f">{cap_ or cap}", st, note)
        if profile_phases and st != ">cap":
            for ph, secs in phases(out):
                rec.add("phases", step, crate, ph, f"{secs:.3f}")
        print(f"[{tag}] {step}: {dt:.1f}s {st} {note}", flush=True)
        return st, out

    if "gen" in steps:
        gcmd = [sys.executable, f"{REPO}/bench/gen_app.py", "--size", size, "--mode", a.mode, "--out", gen,
                "--sink-scan", a.sink_scan] + (["--lib", a.lib] if a.lib else [])
        r = subprocess.run(gcmd, capture_output=True, text=True)
        print(r.stdout.strip(), r.stderr.strip())
        for kv in re.findall(r"(\w+)=(\[[^\]]*\]|\S+)", r.stdout.split(": ", 1)[1]):
            rec.add("meta", *kv)
    # Which build profile the numbers use: the generated workspace's [profile.release] and rustflags.
    try:
        toml = open(f"{gen}/Cargo.toml").read()
        prof = toml.split("[profile.release]", 1)[1].split("[", 1)[0].split() if "[profile.release]" in toml else []
        prof = " ".join(prof).replace(" = ", "=") if prof else "default"
        flags = re.search(r"rustflags = (\[.*\])", open(f"{gen}/.cargo/config.toml").read())
    except OSError:
        prof, flags = "unknown", None
    rec.add("meta", "profile_release", prof.replace(",", ";"))
    rec.add("meta", "rustflags", (flags.group(1) if flags else "none").replace(",", ";"))
    print(f"[{tag}] profile.release: {prof}; rustflags: {flags.group(1) if flags else 'none'}", flush=True)
    fw_ok = True

    if "release" in steps:
        timed("release_deps_dataflow-flat", ["build", "--release", *off, "-p", "dataflow-flat"], "dataflow-flat")
        st, _ = timed("release_lib_marketlib", ["rustc", "--release", *off, "-p", "marketlib", "--", "-Ztime-passes"],
                      "marketlib", profile_phases=True, log="release_lib")
        st, _ = timed("release_app_scratch", ["rustc", "--release", *off, "-p", "app", "--", "-Ztime-passes"],
                      "app", profile_phases=True, log="release_app")
        fw_ok = st == "ok"

    if "mono" in steps:
        for crate in ("marketlib", "app"):
            if crate == "app" and not fw_ok:
                continue
            subprocess.run(["touch", f"{gen}/{crate}/src/lib.rs"])
            dt, st, out = sh(cargo + ["rustc", "--release", *off, "-p", crate, "--", "-Zprint-mono-items=yes"], gen, cap,
                             f"{logs}/mono_{crate}.log")
            n = sum(1 for l in out.splitlines() if l.startswith("MONO_ITEM"))
            rec.add("mono", crate, n if st == "ok" else st)
            print(f"[{tag}] mono items {crate}: {n} ({st}, {dt:.1f}s)", flush=True)
            if crate == "app":
                fw_ok = fw_ok and st == "ok"

    if "run" in steps:
        feats = [] if fw_ok else ["--no-default-features", "--features", "base"]
        st, _ = timed("release_runner", ["build", "--release", *off, "-p", "runner", *feats], "runner", log="release_runner",
                      cap_=a.cap_big)
        if st == "ok":
            bin_ = f"{gen}/target/release/runner"
            sides = {}
            for i in range(a.procs):
                dt, st2, out = sh(["taskset", "-c", str(a.cpu), bin_, "--all"] if a.mode == "1a" and size not in BIG else
                                  ["taskset", "-c", str(a.cpu), bin_], gen, 900, f"{logs}/run_{i}.log")
                if st2 != "ok":
                    rec.add("build", "run", "", st2, "; ".join(out.strip().splitlines()[-3:])[:300])
                    print(f"[{tag}] run failed: {out[-300:]}")
                    break
                for m in re.finditer(r"RESULT (\S+) min=([\d.]+) median=([\d.]+)", out):
                    sides.setdefault(m[1], []).append((float(m[2]), float(m[3])))
                    rec.add("run", m[1], i, m[2], m[3])
                if "VERIFIED" in out:
                    sides.setdefault("verified", []).append(1)
            for side, v in sides.items():
                if side == "verified":
                    continue
                mins = [x for x, _ in v]
                meds = [y for _, y in v]
                rec.add("run", side, "all", f"{min(mins):.2f}", f"{statistics.median(meds):.2f}")
                print(f"[{tag}] {side:<13} min {min(mins):8.2f}  median {statistics.median(meds):8.2f}  "
                      f"medians in [{min(meds):.2f}, {max(meds):.2f}] ns/packet", flush=True)
            rec.add("meta", "verified_processes", len(sides.get("verified", [])))

    if "dev" in steps:
        incr = ["-Ztime-passes", "-Zincremental-info"]
        timed("dev_deps_dataflow-flat", ["build", *off, "-p", "dataflow-flat"], "dataflow-flat")
        st, _ = timed("dev_lib_marketlib", ["rustc", *off, "-p", "marketlib", "--", "-Ztime-passes"], "marketlib",
                      profile_phases=True, log="dev_lib")
        if st == "ok":
            small = ["app/src/risk.rs", "app/src/lighter/sender.rs", "app/src/hyper/sender.rs"]
            big = ["app/src/big.rs"]
            seq = [("dev_app_initial", None, None), ("dev_app_noop", None, None),
                   ("dev_app_edit_big_same_len", big, False),
                   ("dev_app_edit_big_plus3_same_len", big + small, False),
                   ("dev_app_edit_big_plus3_longer", big + small, True)]
            n = 0
            for step, files, longer in seq:
                if files:
                    n += 1
                    edit(gen, files, n, longer)
                st, out = timed(step, ["rustc", *off, "-p", "app", "--", *incr], "app", profile_phases=True, log=step)
                with open(f"{logs}/{step}.incr", "w") as f:
                    f.write("\n".join(l for l in out.splitlines() if "incremental" in l.lower()))
                if st != "ok":
                    break
    merge(a.out)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--sizes", default="tiny,small")
    ap.add_argument("--mode", required=True, help="1a | enum | req (see bench/gen_app.py)")
    ap.add_argument("--lib", help="dataflow checkout to build against (default: this repo)")
    ap.add_argument("--out", default=os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "target", "harness"))
    ap.add_argument("--steps", default="gen,release,mono,run,dev")
    ap.add_argument("--cpu", type=int, default=2)
    ap.add_argument("--procs", type=int, default=5)
    ap.add_argument("--cap-small", type=int, default=60)
    ap.add_argument("--cap-big", type=int, default=600)
    ap.add_argument("--sink-scan", default="8")
    a = ap.parse_args()
    a.out = os.path.abspath(a.out)
    for s in a.sizes.split(","):
        do_size(a, s)


if __name__ == "__main__":
    main()
