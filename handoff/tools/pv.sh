#!/bin/bash
cd /home/noe/TGT/dataflow-framework/.claude/worktrees/agent-a76d4446a3da733d0/target/prof
sed -i "s/&Pr[ABCD]</\&Pr$1</" m30-300-70/app/src/lib.rs
echo == $1; ./pf.sh m30-300-70 pf$1.txt
