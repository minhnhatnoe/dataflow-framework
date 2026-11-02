#!/bin/bash
W=/home/noe/TGT/dataflow-framework/.claude/worktrees/agent-ab026fb31d6a1c588/target/ms
export MSDIR=${MSDIR:-$W/out}
mkdir -p $MSDIR
LD_PRELOAD=$W/shim.so exec "$@"
