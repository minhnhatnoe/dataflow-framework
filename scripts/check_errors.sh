#!/usr/bin/env bash
# Verifies that each class of mistake fails the way the README says.
#
#   scripts/check_errors.sh            # all cases
#   scripts/check_errors.sh cycle      # one case
#
# Each case is a tiny standalone crate under target/error-cases/<name>,
# depending on dataflow-flat by path. We match on error codes / our own panic
# messages (stable across compiler versions), not on full stderr snapshots.
# Every cargo invocation has a timeout so a regression can't hang the run.
set -u
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CASES_DIR="$ROOT/target/error-cases"
export CARGO_TARGET_DIR="$ROOT/target/error-cases-target"
TIMEOUT="${TIMEOUT:-120}"

PRELUDE='use dataflow_flat::{Component, ComponentSubscription, Configs, Deps, Pipeline};

#[derive(ComponentSubscription)]
pub struct Src;
impl Component for Src {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self { Src }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 1 }
}
'

# name | cargo command (check/build/run) | regex that must appear | body
declare -A CMD PATTERN BODY

CMD[field_typo]=check
PATTERN[field_typo]='E0609'
BODY[field_typo]='
#[derive(ComponentSubscription)]
#[subscribe(value: Src)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self { A }
    fn update(&mut self, input: Self::Input<'"'"'_>) -> u64 { *input.valeu }
}
fn main() {}'

CMD[not_a_component]=check
PATTERN[not_a_component]='E0277.*Component|Component.*not (satisfied|implemented)'
BODY[not_a_component]='
#[derive(ComponentSubscription)]
#[subscribe(value: String)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self { A }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
fn main() {}'

CMD[duplicate_name]=check
PATTERN[duplicate_name]='duplicate subscription name'
BODY[duplicate_name]='
#[derive(ComponentSubscription)]
#[subscribe(value: Src, value: Src)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self { A }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
fn main() {}'

CMD[wrong_input_type]=check
PATTERN[wrong_input_type]='E0308|mismatched types'
BODY[wrong_input_type]='
#[derive(ComponentSubscription)]
#[subscribe(value: Src)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self { A }
    fn update(&mut self, input: Self::Input<'"'"'_>) -> u64 { let s: &String = input.value; s.len() as u64 }
}
fn main() {}'

CMD[cycle]=check
PATTERN[cycle]='E0275|AcyclicList'
BODY[cycle]='
#[derive(ComponentSubscription)]
#[subscribe(b: B)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self { A }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
#[derive(ComponentSubscription)]
#[subscribe(a: A)]
pub struct B;
impl Component for B {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self { B }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
fn main() { let _ = Pipeline::<Deps![A]>::build(Configs::new()); }'

# Same type written two different ways: the derive can only dedupe identical
# spellings, so the compile-time plan catches it. This is a post-monomorphization
# error: `cargo check` does NOT report it, `cargo build` does.
CMD[duplicate_dep_via_alias]=build
PATTERN[duplicate_dep_via_alias]='E0080|same dependency twice'
BODY[duplicate_dep_via_alias]='
type AlsoSrc = Src;
#[derive(ComponentSubscription)]
#[subscribe(x: Src, y: AlsoSrc)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self { A }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
fn main() { let _ = Pipeline::<Deps![A]>::build(Configs::new()); }'

CMD[root_listed_twice]=build
PATTERN[root_listed_twice]='E0080|listed twice'
BODY[root_listed_twice]='
fn main() { let _ = Pipeline::<Deps![Src, Src]>::build(Configs::new()); }'

CMD[keep_input_reference]=check
PATTERN[keep_input_reference]='lifetime may not live long enough|borrowed data escapes|E0521'
BODY[keep_input_reference]='
#[derive(ComponentSubscription)]
#[subscribe(value: Src)]
pub struct A { kept: Option<&'"'"'static u64> }
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self { A { kept: None } }
    fn update(&mut self, input: Self::Input<'"'"'_>) -> u64 { self.kept = Some(input.value); 0 }
}
fn main() {}'

CMD[name_input_struct_outside]=check
PATTERN[name_input_struct_outside]='E0412|E0433|cannot find'
BODY[name_input_struct_outside]='
fn main() { let _x: Option<SrcInput<'"'"'static>> = None; }'

# Runtime (at Pipeline::build), not compile time.
CMD[unused_config]=run
PATTERN[unused_config]='BuildError \{ unused: \[[^]]*Unused'
BODY[unused_config]='
#[derive(ComponentSubscription)]
pub struct Unused;
impl Component for Unused {
    type Config = ();
    type Output = ();
    fn new((): ()) -> Self { Unused }
    fn update(&mut self, _: Self::Input<'"'"'_>) {}
}
fn main() {
    match Pipeline::<Deps![Src]>::build(Configs::new().with::<Unused>(())) {
        Ok(_) => println!("unexpectedly built"),
        Err(e) => println!("{e:?}"),
    }
}'

ORDER=(field_typo not_a_component duplicate_name wrong_input_type cycle
       duplicate_dep_via_alias root_listed_twice keep_input_reference
       name_input_struct_outside unused_config)

run_case() {
    local name=$1 dir="$CASES_DIR/$1"
    mkdir -p "$dir/src"
    cat > "$dir/Cargo.toml" <<EOF
[package]
name = "case_$name"
version = "0.0.0"
edition = "2021"

[dependencies]
dataflow-flat = { path = "$ROOT/dataflow-flat" }

[workspace]
EOF
    printf '#![allow(dead_code, unused_imports, unused_variables)]\n%s\n%s\n' "$PRELUDE" "${BODY[$name]}" > "$dir/src/main.rs"

    local out
    out=$(cd "$dir" && timeout "$TIMEOUT" cargo "${CMD[$name]}" -q 2>&1)
    local rc=$?
    if [ $rc -eq 124 ]; then
        echo "TIMEOUT  $name"; return 1
    fi
    if echo "$out" | tr '\n' ' ' | grep -Eq "${PATTERN[$name]}"; then
        echo "ok       $name"
    else
        echo "FAIL     $name  (cargo ${CMD[$name]}, exit $rc; expected /${PATTERN[$name]}/)"
        echo "$out" | grep -E "^(error|warning)|Err|unused" | head -5 | sed 's/^/           /'
        return 1
    fi
}

failed=0
if [ $# -gt 0 ]; then
    for c in "$@"; do run_case "$c" || failed=1; done
else
    for c in "${ORDER[@]}"; do run_case "$c" || failed=1; done
fi
exit $failed
