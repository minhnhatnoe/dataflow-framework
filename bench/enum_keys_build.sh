#!/bin/bash
# usage: bench/enum_keys_build.sh WORKDIR SIDE STEP   (STEP: check | release | dev | devedit)
# One build in a capped scope; prints wall seconds and peak RSS of the largest process; log in WORKDIR/logs.
W=$1; S=$2; STEP=$3
mkdir -p "$W/logs"
cd "$W/$S" || exit 1
case $STEP in
  check) A="cargo check" ;;
  release) A="cargo build --release" ;;
  dev) A="cargo build" ;;
  devedit) sed -i '0,/self\.n += 1;/s//self.n += 2;/' src/main.rs; A="cargo build" ;;
esac
export CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR="$W/$S/target"
systemd-run --user --scope -q -p MemoryMax=12G -p MemorySwapMax=0 -- timeout ${TL:-900} /usr/bin/time -f "RESULT side=$S step=$STEP wall=%es maxrss_kb=%M exit=%x" $A > "$W/logs/$S-$STEP.log" 2>&1
tail -n1 "$W/logs/$S-$STEP.log"
