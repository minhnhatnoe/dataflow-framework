#!/bin/bash
# Profile-guided release build of a generated app (bench/gen_app.py output).
#   scripts/pgo.sh APP_DIR [TRAIN_SEED]
# Builds the runner instrumented, trains it on a packet stream with a different seed than the
# measured one (default 987654321; the runner's own is 0x5EED), merges the profile with llvm-profdata
# from the pinned toolchain, and rebuilds with -Cprofile-use into APP_DIR/target-pgo.
# The runner needs a --seed option (the generated runner hard-codes 0x5EED: change
# `feed_gen::packets(npk, COUNTS, 0x5EED)` to `arg("--seed", 0x5EED) as u64`); without it the
# training input equals the test input, which overstates the gain.
# Measured on typical/enum (Ryzen 9 7900X, cpu 10): framework -5%, hand-written baseline -4%.
# Each build is the usual release profile (+ -Zshare-generics=n -Ctarget-cpu=native), 30 s, 1.4 GB.
set -e
APP=$(realpath "$1"); SEED=${2:-987654321}
PD=$(rustc --print sysroot)/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-profdata
BASE="-Zshare-generics=n -Ctarget-cpu=native"
cd "$APP"
rm -rf target-pgo-gen pgo-prof
RUSTFLAGS="$BASE -Cprofile-generate=$APP/pgo-prof" CARGO_BUILD_JOBS=4 \
  cargo build --release -p runner --target-dir target-pgo-gen
LLVM_PROFILE_FILE="$APP/pgo-prof/t-%m.profraw" taskset -c 10 target-pgo-gen/release/runner --rounds 3 --seed "$SEED"
"$PD" merge -o pgo.profdata pgo-prof/t-*.profraw
RUSTFLAGS="$BASE -Cprofile-use=$APP/pgo.profdata" CARGO_BUILD_JOBS=4 \
  cargo build --release -p runner --target-dir target-pgo
echo "PGO runner: $APP/target-pgo/release/runner"
