#!/bin/bash
# usage: bench/enum_keys_run.sh WORKDIR [rounds]  -- alternating f/e/h, random and sorted, perf stat per process
W=$1; R=${2:-5}
mkdir -p "$W/logs/run"
EV=cycles,instructions,ex_ret_brn_misp,ls_stlf,ex_no_retire.load_not_complete
for r in $(seq 1 $R); do
  for mode in random sorted; do
    if [ $((r % 2)) = 1 ]; then ORDER="f e h"; else ORDER="h e f"; fi
    for s in $ORDER; do
      perf stat -x, -e $EV -o "$W/logs/run/$s-$mode-$r.perf" taskset -c 2 "$W/$s/target/release/ek_$s" $mode > "$W/logs/run/$s-$mode-$r.out" 2>&1
    done
  done
done
echo RUNDONE
