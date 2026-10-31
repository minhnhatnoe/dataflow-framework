#!/usr/bin/env bash
# usage: scripts/bench.sh [layers=20] [wide]
# Regenerates bench/src/main.rs (full mode), builds it in release, runs the binary
# BENCH_PROCS (default 5) times pinned to CPU ${BENCH_CPU:-2}, prints each process's
# min/median ns/run per side and an overall summary, then regenerates `full 20` so the
# committed file stays the default.
set -euo pipefail
cd "$(dirname "$0")/.."
layers=${1:-20}; wide=${2:-}
procs=${BENCH_PROCS:-5}; cpu=${BENCH_CPU:-2}
trap 'python3 bench/gen_bench.py full 20 >/dev/null' EXIT
python3 bench/gen_bench.py full "$layers" $wide
timeout 300 cargo build --release -p bench 2>&1 | grep -E "^(error|\s+Finished)" || true
bin=${CARGO_TARGET_DIR:-target}/release/bench
out=$(mktemp); trap 'rm -f "$out"; python3 bench/gen_bench.py full 20 >/dev/null' EXIT
for i in $(seq "$procs"); do
    echo "process $i:"
    taskset -c "$cpu" "$bin" | tee -a "$out" | sed 's/^/  /'
done
python3 - "$out" <<'P'
import re, statistics, sys
side = {}
for line in open(sys.argv[1]):
    m = re.match(r"(flat|hand-written):\s+min\s+([\d.]+)\s+median\s+([\d.]+)", line)
    if m: side.setdefault(m[1], []).append((float(m[2]), float(m[3])))
print("summary over processes (min of mins / median of medians / spread of medians):")
for k, v in side.items():
    mins = [a for a, _ in v]; meds = [b for _, b in v]
    print(f"  {k:<13} min {min(mins):7.1f}  median {statistics.median(meds):7.1f}  medians in [{min(meds):.1f}, {max(meds):.1f}] ns/run")
P
