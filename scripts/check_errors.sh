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

PRELUDE='use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline, Root, Sinks};

#[derive(Root)]
pub struct Poll;

#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct Src;
impl Component for Src {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (Src, 0) }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 1 }
}
'

# name | cargo command (check/build/run) | regex that must appear | body
# FEATURES[name] = dataflow-flat cargo features for that case (default: none)
declare -A CMD PATTERN BODY FEATURES

CMD[field_typo]=check
PATTERN[field_typo]='E0609'
BODY[field_typo]='
#[derive(ComponentSubscription)]
#[subscribe(value: Src)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, input: Self::Input<'"'"'_>) -> u64 { *input.valeu }
}
fn main() {}'

CMD[not_a_component]=check
PATTERN[not_a_component]='E0277.*Node|Node.*not (satisfied|implemented)'
BODY[not_a_component]='
#[derive(ComponentSubscription)]
#[subscribe(value: String)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A, 0) }
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
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A, 0) }
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
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, input: Self::Input<'"'"'_>) -> u64 { let s: &String = input.value; s.len() as u64 }
}
fn main() {}'

CMD[cycle]=check
# The trait solver cannot prove that a component of a cyclic graph is a node (its `Node` impl needs its
# subscriptions to be nodes, which needs itself); it names a component of the cycle.
PATTERN[cycle]='E0275.*overflow evaluating the requirement .*(Upstream|Node|DepList)'
BODY[cycle]='
#[derive(ComponentSubscription)]
#[subscribe(b: B)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
#[derive(ComponentSubscription)]
#[subscribe(a: A)]
pub struct B;
impl Component for B {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (B, 0) }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
fn main() { let _ = Pipeline::<Poll, (A,)>::build(Configs::new()); }'

# A component that subscribes to itself.
CMD[self_cycle]=check
PATTERN[self_cycle]='E0275.*overflow evaluating the requirement .*(Upstream|Node|DepList)'
BODY[self_cycle]='
#[derive(ComponentSubscription)]
#[subscribe(me: A)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
fn main() { let _ = Pipeline::<Poll, (A,)>::build(Configs::new()); }'

# The root type must be a `#[derive(Root)]`.
CMD[root_not_a_root]=check
PATTERN[root_not_a_root]='E0277.*(String: Root|not valid sinks)'
BODY[root_not_a_root]='
fn main() { let _ = Pipeline::<String, (Src,)>::build(Configs::new()); }'

# Sinks are components: the root is not one.
CMD[sink_is_the_root]=check
PATTERN[sink_is_the_root]='E0277.*(Poll: Comp|Poll: Component|not valid sinks)'
BODY[sink_is_the_root]='
fn main() { let _ = Pipeline::<Poll, (Poll,)>::build(Configs::new()); }'

# Generic code over a pipeline needs the `Sinks` bound (and nothing else).
CMD[generic_missing_sinks_bound]=check
PATTERN[generic_missing_sinks_bound]='E0277.*(not valid sinks|Sinks<R>)'
BODY[generic_missing_sinks_bound]='
fn drive<R, S>(p: &mut Pipeline<R, S>, r: R) { p.push(r) }
fn main() {}'

# ...and with it, generic code compiles with no feature gate in the user crate.
CMD[generic_sinks_bound_ok]=check
PATTERN[generic_sinks_bound_ok]='NOERROR'
BODY[generic_sinks_bound_ok]='
fn drive<R, S: Sinks<R>>(p: &mut Pipeline<R, S>, r: R) { p.push(r) }
fn main() { let mut p = Pipeline::<Poll, (Src,)>::build(Configs::new()).unwrap(); drive(&mut p, Poll); }'

# A component that nothing connects to the root. Reported by the plan's
# constant evaluation, so only `cargo build` sees it.
CMD[unreachable_component]=build
PATTERN[unreachable_component]='E0080.*Orphan is not reachable from the root'
BODY[unreachable_component]='
#[derive(ComponentSubscription)]
pub struct Orphan;
impl Component for Orphan {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (Orphan, 0) }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
#[derive(ComponentSubscription)]
#[subscribe(s: Src, o: Orphan)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
fn main() { let _ = Pipeline::<Poll, (A,)>::build(Configs::new()); }'

# A second `#[derive(Root)]` type in the graph.
CMD[second_root]=build
# (The second root takes a component position, so the compiler usually reports the graph as not valid
# before the plan's own message; either is an error naming the problem.)
PATTERN[second_root]='E0080.*Other is a root, but the pipeline.s root is a different type|E0277.*dataflow graph is not valid.*second root'
BODY[second_root]='
#[derive(Root)]
pub struct Other;
#[derive(ComponentSubscription)]
#[subscribe(s: Src, o: Other)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
fn main() { let _ = Pipeline::<Poll, (A,)>::build(Configs::new()); }'

# The graph's root is not the pipeline'"'"'s `R`.
CMD[wrong_root_type]=build
PATTERN[wrong_root_type]='E0080.*Poll is a root, but the pipeline.s root is a different type|E0277.*Is<'
BODY[wrong_root_type]='
#[derive(Root)]
pub struct Other;
fn main() { let _ = Pipeline::<Other, (Src,)>::build(Configs::new()); }'

CMD[keep_input_reference]=check
PATTERN[keep_input_reference]='lifetime may not live long enough|borrowed data escapes|E0521'
BODY[keep_input_reference]='
#[derive(ComponentSubscription)]
#[subscribe(value: Src)]
pub struct A { kept: Option<&'"'"'static u64> }
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A { kept: None }, 0) }
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
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, ()) { (Unused, ()) }
    fn update(&mut self, _: Self::Input<'"'"'_>) {}
}
fn main() {
    match Pipeline::<Poll, (Src,)>::build(Configs::new().with::<Unused>(())) {
        Ok(_) => println!("unexpectedly built"),
        Err(e) => println!("{e:?}"),
    }
}'

# More than MAX_COMPONENTS (512) components: 600 leaves under one root. Reported by the
# plan's constant evaluation, so only `cargo build` sees it.
CMD[too_many_components]=build
PATTERN[too_many_components]='E0080|too many components'
BODY[too_many_components]=$(python3 - <<'PY'
# 30 groups of 20 leaves: 600 leaves + 30 groups + A = 631 components.
G, K = 30, 20
out = []
def comp(name, subs):
    attr = "#[subscribe(%s)] " % ", ".join(f"s{i}: {d}" for i, d in enumerate(subs)) if subs else ""
    out.append(f"#[derive(ComponentSubscription)] {attr}pub struct {name};")
    out.append(f"impl Component for {name} {{ type Config = (); type Output = u64; fn new((): (), _: Self::Init<'_>) -> (Self, u64) {{ ({name}, 0) }} fn update(&mut self, _: Self::Input<'_>) -> u64 {{ 0 }} }}")
for g in range(G):
    for k in range(K):
        comp(f"L{g}_{k}", ["Poll"])
    comp(f"G{g}", [f"L{g}_{k}" for k in range(K)])
comp("A", [f"G{g}" for g in range(G)])
out.append("fn main() { let _ = Pipeline::<Poll, (A,)>::build(Configs::new()); }")
print("\n".join(out))
PY
)

ORDER=(field_typo not_a_component duplicate_name wrong_input_type cycle
       self_cycle
       too_many_components root_not_a_root sink_is_the_root
       generic_missing_sinks_bound generic_sinks_bound_ok
       unreachable_component second_root wrong_root_type
       keep_input_reference name_input_struct_outside unused_config)

run_case() {
    local name=$1 dir="$CASES_DIR/$1"
    mkdir -p "$dir/src"
    local feat=""
    if [ -n "${FEATURES[$name]:-}" ]; then feat=", features = [\"${FEATURES[$name]}\"]"; fi
    cat > "$dir/Cargo.toml" <<EOF
[package]
name = "case_$name"
version = "0.0.0"
edition = "2021"

[dependencies]
dataflow-flat = { path = "$ROOT/dataflow-flat"$feat }

[workspace]
EOF
    printf '#![allow(dead_code, unused_imports, unused_variables)]\n%s\n%s\n' "$PRELUDE" "${BODY[$name]}" > "$dir/src/main.rs"

    local out
    out=$(cd "$dir" && timeout "$TIMEOUT" cargo "${CMD[$name]}" -q 2>&1)
    local rc=$?
    if [ $rc -eq 124 ]; then
        echo "TIMEOUT  $name"; return 1
    fi
    if [ "${PATTERN[$name]}" = NOERROR ]; then
        if [ $rc -eq 0 ] && ! echo "$out" | grep -q '^error'; then echo "ok       $name"; return 0; fi
        echo "FAIL     $name  (cargo ${CMD[$name]} should succeed, exit $rc)"
        echo "$out" | grep -E "^error" -A5 | head -12 | sed 's/^/           /'
        return 1
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
