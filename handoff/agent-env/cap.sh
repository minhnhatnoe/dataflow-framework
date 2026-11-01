#!/bin/bash
# Usage: cap.sh <build|measure> <cap, e.g. 4G> <timeout-seconds> <cmd...>
#
# Memory cap: runs <cmd> in a fresh cgroup-v1 memory child with memory.limit_in_bytes = cap
# (no swap on this machine). An OOM kill inside the cap is a RESULT to record (exit 137), never
# a reason to retry uncapped.
#
# Machine lock (shared by all agents): `build` takes it shared, `measure` exclusive. So timing
# measurements (build time, run time) never overlap another agent's build or measurement, while
# ordinary builds/tests from several agents may run together.
# Sets CARGO_BUILD_JOBS (build: 2, measure: 4) unless already set. Machine: 4 cores, ~14 GB usable.
set -u
mode=$1 cap=$2 secs=$3; shift 3
lock=/tmp/claude-0/-home-user-repo/8e981b07-5287-54fa-a02f-4121b77bfce6/scratchpad/machine.lock
case $mode in build) lk=-s ;; measure) lk=-x ;; *) echo "mode must be build|measure" >&2; exit 2 ;; esac
rel=$(awk -F: '$2=="memory"{print $3}' /proc/self/cgroup)
base=/sys/fs/cgroup/memory$rel
cg=$base/cap.$$.$RANDOM
mkdir "$cg" || { echo "cap.sh: cannot create cgroup" >&2; exit 2; }
bytes=$(numfmt --from=iec "$cap")
echo "$bytes" > "$cg/memory.limit_in_bytes"
echo "$bytes" > "$cg/memory.memsw.limit_in_bytes" 2>/dev/null
if [ $mode = build ]; then export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-2}; else export CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-4}; fi
flock $lk "$lock" bash -c 'echo $$ > "$1/cgroup.procs"; shift; exec timeout "$@"' _ "$cg" "$secs" "$@"
rc=$?
peak=$(cat "$cg/memory.max_usage_in_bytes" 2>/dev/null)
oom=$(cat "$cg/memory.failcnt" 2>/dev/null)
echo "cap.sh: rc=$rc peak=$(numfmt --to=iec ${peak:-0}) cap=$cap limit_hits=${oom:-?}" >&2
# move ourselves out isn't needed (we never joined); remove the empty cgroup
rmdir "$cg" 2>/dev/null
exit $rc
