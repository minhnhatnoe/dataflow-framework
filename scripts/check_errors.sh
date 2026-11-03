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

PRELUDE='use dataflow_flat::{Component, ComponentSubscription, Configs, Pipeline, Root, Sinks, UpdateHandler};

#[derive(Root)]
pub struct Poll;

#[derive(ComponentSubscription, UpdateHandler)]
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
#[derive(ComponentSubscription, UpdateHandler)]
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
#[derive(ComponentSubscription, UpdateHandler)]
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
#[derive(ComponentSubscription, UpdateHandler)]
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
#[derive(ComponentSubscription, UpdateHandler)]
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
# TODO(api-one-component): the overflow now names `Anchor<..>: Planned`, not a component.
PATTERN[cycle]='E0275.*overflow evaluating the requirement .*(Upstream|Node|Vertex|DepList|Planned)'
BODY[cycle]='
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(b: B)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
#[derive(ComponentSubscription, UpdateHandler)]
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
PATTERN[self_cycle]='E0275.*overflow evaluating the requirement .*(Upstream|Node|Vertex|DepList|Planned)'
BODY[self_cycle]='
#[derive(ComponentSubscription, UpdateHandler)]
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
#[derive(ComponentSubscription, UpdateHandler)]
pub struct Orphan;
impl Component for Orphan {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (Orphan, 0) }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
#[derive(ComponentSubscription, UpdateHandler)]
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
#[derive(ComponentSubscription, UpdateHandler)]
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
#[derive(ComponentSubscription, UpdateHandler)]
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
#[derive(ComponentSubscription, UpdateHandler)]
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

# More than MAX_COMPONENTS (512 with the `small-limits` feature) components: 600 leaves under one root. Reported by the
# plan's constant evaluation, so only `cargo build` sees it.
CMD[too_many_components]=build
FEATURES[too_many_components]=small-limits
PATTERN[too_many_components]='E0080|too many components'
BODY[too_many_components]=$(python3 - <<'PY'
# 30 groups of 20 leaves: 600 leaves + 30 groups + A = 631 components.
G, K = 30, 20
out = []
def comp(name, subs):
    attr = "#[subscribe(%s)] " % ", ".join(f"s{i}: {d}" for i, d in enumerate(subs)) if subs else ""
    out.append(f"#[derive(ComponentSubscription, UpdateHandler)] {attr}pub struct {name};")
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

# --- selectors (demuxing on the subscription edge) ---------------------------------------------------
# Selectors on `Poll`: two guards (never exclusive with each other), and two keyed alternatives (exclusive).
read -r -d '' SELECTORS <<'EOF'
use dataflow_flat::Select;
pub struct G1; pub struct G2; pub struct K0; pub struct K1;
impl Select<Poll> for G1 { type Out = Poll; const KEY: u32 = dataflow_flat::GUARD; fn select(x: &Poll) -> Option<&Poll> { Some(x) } fn key_of(_: &Poll) -> u32 { dataflow_flat::GUARD } }
impl Select<Poll> for G2 { type Out = Poll; const KEY: u32 = dataflow_flat::GUARD; fn select(x: &Poll) -> Option<&Poll> { Some(x) } fn key_of(_: &Poll) -> u32 { dataflow_flat::GUARD } }
impl Select<Poll> for K0 { type Out = Poll; const KEY: u32 = 0; fn select(x: &Poll) -> Option<&Poll> { Some(x) } fn key_of(_: &Poll) -> u32 { 0 } }
impl Select<Poll> for K1 { type Out = Poll; const KEY: u32 = 1; fn select(x: &Poll) -> Option<&Poll> { Some(x) } fn key_of(_: &Poll) -> u32 { 1 } }
macro_rules! comp {
    ($name:ident, [$($sub:tt)*]) => {
        #[derive(ComponentSubscription, UpdateHandler)]
        #[subscribe($($sub)*)]
        pub struct $name;
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): (), _: Self::Init<'_>) -> (Self, u64) { ($name, 0) }
            fn update(&mut self, _: Self::Input<'_>) -> u64 { 0 }
        }
    };
}
EOF

# Rule 4: two branches that can fire in the same step (two guards) joined in one component.
CMD[non_exclusive_join]=build
PATTERN[non_exclusive_join]='E0080.*J joins branches that can fire in the same step'
BODY[non_exclusive_join]="$SELECTORS
comp!(A, [G1(p): Poll]);
comp!(B, [G2(p): Poll]);
comp!(J, [a: A, b: B]);
fn main() { let _ = Pipeline::<Poll, (J,)>::build(Configs::new()); }"

# Different keys at the same place are exclusive: this compiles.
CMD[exclusive_join_ok]=build
PATTERN[exclusive_join_ok]='NOERROR'
BODY[exclusive_join_ok]="$SELECTORS
comp!(A, [K0(p): Poll]);
comp!(B, [K1(p): Poll]);
comp!(J, [a: A, b: B]);
fn main() { let _ = Pipeline::<Poll, (J,)>::build(Configs::new()); }"

# Rule 4 at a deep place: six branches on different root keys plus two that agree on the first selector
# (K3) and differ only at the second place, where both are guards. Must be rejected, and the verdict must
# not depend on which place the pairwise check splits on first.
CMD[deep_non_exclusive_join]=build
PATTERN[deep_non_exclusive_join]='E0080.*J joins branches that can fire in the same step'
BODY[deep_non_exclusive_join]="$SELECTORS
pub struct K2; pub struct K3; pub struct K4; pub struct K5;
impl Select<Poll> for K2 { type Out = Poll; const KEY: u32 = 2; fn select(x: &Poll) -> Option<&Poll> { Some(x) } fn key_of(_: &Poll) -> u32 { 2 } }
impl Select<Poll> for K3 { type Out = Poll; const KEY: u32 = 3; fn select(x: &Poll) -> Option<&Poll> { Some(x) } fn key_of(_: &Poll) -> u32 { 3 } }
impl Select<Poll> for K4 { type Out = Poll; const KEY: u32 = 4; fn select(x: &Poll) -> Option<&Poll> { Some(x) } fn key_of(_: &Poll) -> u32 { 4 } }
impl Select<Poll> for K5 { type Out = Poll; const KEY: u32 = 5; fn select(x: &Poll) -> Option<&Poll> { Some(x) } fn key_of(_: &Poll) -> u32 { 5 } }
comp!(A0, [K0(p): Poll]);
comp!(A1, [K1(p): Poll]);
comp!(A2, [K2(p): Poll]);
comp!(A4, [K4(p): Poll]);
comp!(A5, [K5(p): Poll]);
comp!(D1, [K3(G1(p)): Poll]);
comp!(D2, [K3(G2(p)): Poll]);
comp!(J, [a0: A0, a1: A1, a2: A2, d1: D1, d2: D2, a4: A4, a5: A5]);
fn main() { let _ = Pipeline::<Poll, (J,)>::build(Configs::new()); }"

# The same shape with exclusive keys at the deep place: compiles.
CMD[deep_exclusive_join_ok]=build
PATTERN[deep_exclusive_join_ok]='NOERROR'
BODY[deep_exclusive_join_ok]="$SELECTORS
pub struct K2; pub struct K3; pub struct K4; pub struct K5;
impl Select<Poll> for K2 { type Out = Poll; const KEY: u32 = 2; fn select(x: &Poll) -> Option<&Poll> { Some(x) } fn key_of(_: &Poll) -> u32 { 2 } }
impl Select<Poll> for K3 { type Out = Poll; const KEY: u32 = 3; fn select(x: &Poll) -> Option<&Poll> { Some(x) } fn key_of(_: &Poll) -> u32 { 3 } }
impl Select<Poll> for K4 { type Out = Poll; const KEY: u32 = 4; fn select(x: &Poll) -> Option<&Poll> { Some(x) } fn key_of(_: &Poll) -> u32 { 4 } }
impl Select<Poll> for K5 { type Out = Poll; const KEY: u32 = 5; fn select(x: &Poll) -> Option<&Poll> { Some(x) } fn key_of(_: &Poll) -> u32 { 5 } }
comp!(A0, [K0(p): Poll]);
comp!(A1, [K1(p): Poll]);
comp!(A2, [K2(p): Poll]);
comp!(A4, [K4(p): Poll]);
comp!(A5, [K5(p): Poll]);
comp!(D1, [K3(K0(p)): Poll]);
comp!(D2, [K3(K1(p)): Poll]);
comp!(J, [a0: A0, a1: A1, a2: A2, d1: D1, d2: D2, a4: A4, a5: A5]);
fn main() { let _ = Pipeline::<Poll, (J,)>::build(Configs::new()); }"

# Rule 5 is gone: a selected edge read in steps where its selectors do not match is stored (J also runs when K1
# matches, and reads the last K0 value). It needs an initial value from the selector: these hand-written selectors
# have no `Initial` impl.
CMD[demux_reader_not_implied]=build
PATTERN[demux_reader_not_implied]='E0277.*the selector .K1. has no initial value'
BODY[demux_reader_not_implied]="$SELECTORS
comp!(A, [K0(p): Poll]);
comp!(J, [a: A, K1(p): Poll]);
fn main() { let _ = Pipeline::<Poll, (J,)>::build(Configs::new()); }"

# The same graph with `Initial` impls compiles. A covered edge (A below) is not stored and needs none.
CMD[stored_selector_ok]=build
PATTERN[stored_selector_ok]='NOERROR'
BODY[stored_selector_ok]="$SELECTORS
impl Clone for Poll { fn clone(&self) -> Poll { Poll } }
impl dataflow_flat::Initial<Poll> for K0 { fn initial() -> Poll { Poll } }
impl dataflow_flat::Initial<Poll> for K1 { fn initial() -> Poll { Poll } }
comp!(A, [K0(p): Poll]);
comp!(J, [a: A, K1(p): Poll]);
fn main() { let _ = Pipeline::<Poll, (J,)>::build(Configs::new()); }"

# A stored payload without `Default` (the selector's initial value is `Default::default()`).
CMD[stored_payload_no_default]=build
PATTERN[stored_payload_no_default]='E0277.*(has no initial value|NoDef: (std::default::)?Default|NoDef. doesn.t implement)'
read -r -d '' BODY[stored_payload_no_default] <<'EOT'
#[derive(Clone)]
pub struct NoDef(u8);
#[derive(Root, dataflow_flat::Demux)]
pub enum E { X(NoDef), Y(u64) }
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(E::X(x): E, E::Y(y): E)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u64 { *i.y + i.x.0 as u64 }
}
fn main() { let _ = Pipeline::<E, (A,)>::build(Configs::new()); }
EOT

# A stored payload without `Clone`.
CMD[stored_payload_no_clone]=build
PATTERN[stored_payload_no_clone]='E0277.*NoClone. is read through selectors in steps where they do not match, so it is stored, and must implement .Clone.'
read -r -d '' BODY[stored_payload_no_clone] <<'EOT'
#[derive(Default)]
pub struct NoClone(u8);
#[derive(Root, dataflow_flat::Demux)]
pub enum E { X(NoClone), Y(u64) }
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(E::X(x): E, E::Y(y): E)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u64 { *i.y + i.x.0 as u64 }
}
fn main() { let _ = Pipeline::<E, (A,)>::build(Configs::new()); }
EOT

# A payload with neither `Default` nor `Clone` compiles when its variant is only read covered (`X` here: the
# subscriber runs only when it matches) or as a handled entry.
CMD[unstored_payload_ok]=build
PATTERN[unstored_payload_ok]='NOERROR'
read -r -d '' BODY[unstored_payload_ok] <<'EOT'
pub struct Opaque(u8);
#[derive(Root, dataflow_flat::Demux)]
pub enum E { X(Opaque), Y(u64) }
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(E::X(x): E)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u64 { i.x.0 as u64 }
}
fn main() { let _ = Pipeline::<E, (A,)>::build(Configs::new()); }
EOT

# Rule 6 through selectors: B is reached from the root only via a selector on a component nothing connects.
CMD[selector_unreachable_component]=build
PATTERN[selector_unreachable_component]='E0080.*Orphan is not reachable from the root'
read -r -d '' BODY[selector_unreachable_component] <<'EOF'
#[derive(ComponentSubscription, UpdateHandler)]
pub struct Orphan;
impl Component for Orphan {
    type Config = ();
    type Output = Option<u64>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Option<u64>) { (Orphan, None) }
    fn update(&mut self, _: Self::Input<'_>) -> Option<u64> { None }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(s: Src, Some(o): Orphan)]
pub struct B;
impl Component for B {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (B, 0) }
    fn update(&mut self, _: Self::Input<'_>) -> u64 { 0 }
}
fn main() { let _ = Pipeline::<Poll, (B,)>::build(Configs::new()); }
EOF

# A selector applied to a value it does not select from.
CMD[selector_wrong_input]=check
PATTERN[selector_wrong_input]='E0277.*(selector chain.*cannot be applied|SelectChain<u64>)'
read -r -d '' BODY[selector_wrong_input] <<'EOF'
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(x): Src)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, _: Self::Input<'_>) -> u64 { 0 }
}
fn main() {}
EOF

# --- plan rules ---------------------------------------------------------------------------

# Rule 6 when the orphan is the first leaf (it takes execution position 0, not the root): reported by the
# plan, not as an arithmetic underflow in the component position.
CMD[orphan_first]=build
PATTERN[orphan_first]='E0080.*Orphan is not reachable from the root'
read -r -d '' BODY[orphan_first] <<'EOF2'
#[derive(ComponentSubscription, UpdateHandler)]
pub struct Orphan;
impl Component for Orphan {
    type Config = ();
    type Output = Option<u64>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Option<u64>) { (Orphan, None) }
    fn update(&mut self, _: Self::Input<'_>) -> Option<u64> { None }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(o): Orphan, s: Src)]
pub struct B;
impl Component for B {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (B, 0) }
    fn update(&mut self, _: Self::Input<'_>) -> u64 { 0 }
}
fn main() { let _ = Pipeline::<Poll, (B,)>::build(Configs::new()); }
EOF2

# --- handler components (handled entries in `#[subscribe]`) ----------------------------------------------
read -r -d '' HANDLERS <<'EOF'
macro_rules! handler {
    ($name:ident, [$($attr:tt)*]) => {
        #[derive(ComponentSubscription)]
        $($attr)*
        pub struct $name { acc: u64 }
        impl Component for $name {
            type Config = ();
            type Output = u64;
            fn new((): (), _: Self::Init<'_>) -> (Self, u64) { ($name { acc: 0 }, 0) }
            fn update(&mut self, _: Self::Input<'_>) -> u64 { self.acc }
        }
    };
}
EOF

# A handled entry without an `UpdateHandler` impl: the on_unimplemented message, naming the entry and the impl to write.
CMD[handler_missing_update]=check
PATTERN[handler_missing_update]='E0277.*`H` subscribes to `B` without an `UpdateHandler<B>` impl'
BODY[handler_missing_update]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
comp!(B, [K1(p): Poll]);
handler!(H, [#[subscribe(A, B)]]);
impl UpdateHandler<A> for H { fn on_update(&mut self, _input: Self::Input<'_>, v: &u64) { self.acc += *v; } }
fn main() { let _ = Pipeline::<Poll, (H,)>::build(Configs::new()); }"

# The note shows the impl to write.
CMD[handler_missing_update_note]=check
PATTERN[handler_missing_update_note]='write `impl UpdateHandler<B> for H'
BODY[handler_missing_update_note]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
comp!(B, [K1(p): Poll]);
handler!(H, [#[subscribe(A, B)]]);
impl UpdateHandler<A> for H { fn on_update(&mut self, _input: Self::Input<'_>, v: &u64) { self.acc += *v; } }
fn main() {}"

# Rule 4 still applies to handled edges: two guards can fire in the same step.
CMD[handler_non_exclusive_join]=build
PATTERN[handler_non_exclusive_join]='E0080.*H joins branches that can fire in the same step'
BODY[handler_non_exclusive_join]="$SELECTORS
$HANDLERS
comp!(A, [G1(p): Poll]);
comp!(B, [G2(p): Poll]);
handler!(H, [#[subscribe(A, B)]]);
impl UpdateHandler<A> for H { fn on_update(&mut self, _input: Self::Input<'_>, v: &u64) { self.acc += *v; } }
impl UpdateHandler<B> for H { fn on_update(&mut self, _input: Self::Input<'_>, v: &u64) { self.acc += *v; } }
fn main() { let _ = Pipeline::<Poll, (H,)>::build(Configs::new()); }"

# Exclusive handled edges compile (and a handled edge is not subject to rule 5); a selected handled edge `K0(_): Poll`
# is keyed by the marker `h_edges::K0`.
CMD[handler_exclusive_ok]=build
PATTERN[handler_exclusive_ok]='NOERROR'
BODY[handler_exclusive_ok]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
comp!(B, [K1(p): Poll]);
handler!(H, [#[subscribe(A, B)] #[subscribe(K0(_): Poll)]]);
impl UpdateHandler<A> for H { fn on_update(&mut self, _input: Self::Input<'_>, v: &u64) { self.acc += *v; } }
impl UpdateHandler<B> for H { fn on_update(&mut self, _input: Self::Input<'_>, v: &u64) { self.acc += *v; } }
impl UpdateHandler<h_edges::K0> for H { fn on_update(&mut self, _input: Self::Input<'_>, _: &Poll) { self.acc += 1; } }
fn main() { let _ = Pipeline::<Poll, (H,)>::build(Configs::new()); }"

# Two selected handled entries ending in the same selector would share one `UpdateHandler` key.
CMD[handler_duplicate_marker]=check
PATTERN[handler_duplicate_marker]='two handled entries end in the selector `K0`'
BODY[handler_duplicate_marker]="$SELECTORS
$HANDLERS
handler!(H, [#[subscribe(K0(_): Poll, K0(_): Poll)]]);
fn main() {}"

# A selected handled entry without its impl names the marker.
CMD[handler_selected_missing_update]=check
PATTERN[handler_selected_missing_update]='E0277.*`H` subscribes to `h_edges::K0` without an `UpdateHandler<h_edges::K0>` impl'
BODY[handler_selected_missing_update]="$SELECTORS
$HANDLERS
handler!(H, [#[subscribe(K0(_): Poll)]]);
fn main() {}"

# A bare `_` is not a pattern.
CMD[handler_bare_wildcard]=check
PATTERN[handler_bare_wildcard]='a bare `_` is not a pattern'
BODY[handler_bare_wildcard]="$SELECTORS
$HANDLERS
handler!(H, [#[subscribe(_: Poll)]]);
fn main() {}"


# A named entry needs an impl too, and the error names the edge.
CMD[handler_named_missing_update]=check
PATTERN[handler_named_missing_update]='E0277.*`H` subscribes to `A` without an `UpdateHandler<A>` impl'
BODY[handler_named_missing_update]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
handler!(H, [#[subscribe(a: A)]]);
fn main() {}"

# `#[derive(UpdateHandler)]` is a blanket impl, so a hand-written impl for the same component overlaps it.
CMD[handler_derive_and_impl]=check
PATTERN[handler_derive_and_impl]='E0119.*conflicting implementations of trait `UpdateHandler<A>` for type `H`'
BODY[handler_derive_and_impl]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(A)]
pub struct H { acc: u64 }
impl Component for H {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (H { acc: 0 }, 0) }
    fn update(&mut self, _: Self::Input<'_>) -> u64 { self.acc }
}
impl UpdateHandler<A> for H { fn on_update(&mut self, _: Self::Input<'_>, v: &u64) { self.acc += *v; } }
fn main() {}"

# `on_update` gets the edge's value: `&u64` here, not `&u32`.
CMD[handler_wrong_value_type]=check
PATTERN[handler_wrong_value_type]='E0053'
BODY[handler_wrong_value_type]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
handler!(H, [#[subscribe(A)]]);
impl UpdateHandler<A> for H { fn on_update(&mut self, _: Self::Input<'_>, v: &u32) { self.acc += *v as u64; } }
fn main() {}"

# The key of an impl is a node or a generated marker: `u32` is neither.
CMD[handler_wrong_edge_type]=check
PATTERN[handler_wrong_edge_type]='E0277.*the trait bound `u32: HandledEdge` is not satisfied'
BODY[handler_wrong_edge_type]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
handler!(H, [#[subscribe(A)]]);
impl UpdateHandler<A> for H {}
impl UpdateHandler<u32> for H {}
fn main() {}"

# --- `for` entries (catalogs) --------------------------------------------------------------------------------

# A `for` over a catalog compiles when every element has its `UpdateHandler` impl (here one generic impl).
CMD[for_ok]=build
PATTERN[for_ok]='NOERROR'
BODY[for_ok]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
comp!(B, [K1(p): Poll]);
handler!(H, [#[subscribe(for T in dataflow_flat::catalog![A, B] { T })]]);
impl<T: dataflow_flat::Node<Output = u64>> UpdateHandler<T> for H { fn on_update(&mut self, _input: Self::Input<'_>, v: &u64) { self.acc += *v; } }
fn main() { let _ = Pipeline::<Poll, (H,)>::build(Configs::new()); }"

# An element of the catalog without its `UpdateHandler` impl: the same message as for a written-out entry, naming the element.
CMD[for_missing_update]=check
PATTERN[for_missing_update]='E0277.*`H` subscribes to `B` without an `UpdateHandler<B>` impl'
BODY[for_missing_update]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
comp!(B, [K1(p): Poll]);
handler!(H, [#[subscribe(for T in dataflow_flat::catalog![A, B] { T })]]);
impl UpdateHandler<A> for H { fn on_update(&mut self, _input: Self::Input<'_>, v: &u64) { self.acc += *v; } }
fn main() { let _ = Pipeline::<Poll, (H,)>::build(Configs::new()); }"

# Something that is not a catalog.
CMD[for_not_a_catalog]=check
PATTERN[for_not_a_catalog]='E0277.*`u32` is not a catalog.*for T in u32'
BODY[for_not_a_catalog]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
handler!(H, [#[subscribe(for T in u32 { T })]]);
impl UpdateHandler<A> for H { fn on_update(&mut self, _input: Self::Input<'_>, v: &u64) { self.acc += *v; } }
fn main() { let _ = Pipeline::<Poll, (H,)>::build(Configs::new()); }"

# `Zip` of catalogs of different shapes.
CMD[for_zip_shapes]=check
PATTERN[for_zip_shapes]='E0277.*cannot be mapped by this `for` entry|is not a catalog'
BODY[for_zip_shapes]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
comp!(B, [K1(p): Poll]);
handler!(H, [#[subscribe(for (X, Y) in dataflow_flat::Zip<dataflow_flat::catalog![A, B], dataflow_flat::catalog![A]> { X })]]);
fn main() {}"

# A `for` body is one node type: a named entry in it is rejected.
CMD[for_body_named]=check
PATTERN[for_body_named]='a `for` body is one node type'
BODY[for_body_named]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
handler!(H, [#[subscribe(for T in (A,) { x: T })]]);
fn main() {}"

# A selected entry in a `for` body is rejected with the reason.
CMD[for_body_selected]=check
PATTERN[for_body_selected]='a `for` body is one node type.*selected one \(`Some\(S\(_\)\): Ws`\) is not supported'
BODY[for_body_selected]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
handler!(H, [#[subscribe(for S in (A,) { Some(S(_)): Ws })]]);
fn main() {}"

# A pair pattern needs exactly two parts.
CMD[for_pattern_triple]=check
PATTERN[for_pattern_triple]='a `for` pattern destructures a pair'
BODY[for_pattern_triple]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
handler!(H, [#[subscribe(for (X, Y, Z) in (A,) { X })]]);
fn main() {}"

# The element variable of a `for` needs the bounds of the node (`Q<S: Sym>` needs `S: Sym`): leaving them off is rustc's `S: Sym` error, located at the `for` entry (the `for S + Sym` help is rustc's own).
CMD[for_missing_bound]=check
PATTERN[for_missing_bound]='E0277.*the trait bound `S: Sym` is not satisfied.*subscribe\(for S in dataflow_flat::catalog!\[A\] \{ Q<S> \}\).*restricting type parameter `S`.*for S \+ Sym in'
BODY[for_missing_bound]="$SELECTORS
$HANDLERS
pub trait Sym: 'static {}
impl Sym for A {}
comp!(A, [K0(p): Poll]);
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(K0(p): Poll)]
pub struct Q<S: Sym>(std::marker::PhantomData<S>);
impl<S: Sym> Component for Q<S> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (Q(std::marker::PhantomData), 0) }
    fn update(&mut self, _: Self::Input<'_>) -> u64 { 0 }
}
handler!(H, [#[subscribe(for S in dataflow_flat::catalog![A] { Q<S> })]]);
impl<S: Sym> UpdateHandler<Q<S>> for H { fn on_update(&mut self, _input: Self::Input<'_>, v: &u64) { self.acc += *v; } }
fn main() {}"

# --- subscription patterns (`Some(Btc(t)): Src`): sugar over selector chains ------------------------------
# pat_case <name> <pattern> <regex of our message>; the pattern is a subscription of component `A`.
pat_case() {
    CMD[$1]=check
    PATTERN[$1]="subscription pattern: $3"
    BODY[$1]="
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe($2: Src)]
pub struct A;
fn main() {}"
}
pat_case pattern_struct 'Foo { x }' 'struct patterns are not supported'
pat_case pattern_multi_field 'Some(Foo(a, b))' 'multi-field variants are not supported'
pat_case pattern_range 'Foo(a..=b)' 'range patterns are not supported'
pat_case pattern_range_literal 'Foo(1..=3)' 'literal patterns are not supported'
pat_case pattern_guard 'Foo(x) if x > 1' 'guards are not supported'
pat_case pattern_or 'Foo(x) | Bar(x)' 'or-patterns are not supported'
pat_case pattern_literal 'Some(3)' 'literal patterns are not supported'
pat_case pattern_reference 'Some(&x)' 'references are not supported'

# `=>` is gone: the message says how to write the pattern.
CMD[pattern_arrow]=check
PATTERN[pattern_arrow]='`=>` is gone: write the selectors as a pattern'
BODY[pattern_arrow]='
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(x: Src => Some)]
pub struct A;
fn main() {}'

# A bare uppercase name is a unit selector; one that names no type is rustc's E0425 "cannot find type", reported at the attribute.
CMD[unit_selector_unknown]=check
PATTERN[unit_selector_unknown]='E0425.*cannot find type `Disconnected` in this scope.*subscribe\(Disconnected: Src\)'
BODY[unit_selector_unknown]='
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Disconnected: Src)]
pub struct A;
fn main() {}'

# A unit selector on a variant that carries data compiles: the edge fires on that variant, the payload sits in a hidden input field.
CMD[unit_selector_data_variant]=check
PATTERN[unit_selector_data_variant]=NOERROR
read -r -d '' BODY[unit_selector_data_variant] <<'EOT'
#[derive(Root, dataflow_flat::Demux)]
pub enum E { Gone, Data(u64) }
use e::Data;
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Data: E, E::Gone: E)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, _: Self::Input<'_>) -> u64 { 0 }
}
fn main() {}
EOT

# Accepted: `Enum::Variant`, Option and Result patterns, nested.
CMD[pattern_ok]=check
PATTERN[pattern_ok]=NOERROR
read -r -d '' BODY[pattern_ok] <<'EOT'
#[derive(Root, dataflow_flat::Demux)]
pub enum E { X(u64), Y(u64) }
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(E::X(x): E)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = Option<Result<u64, u64>>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Self::Output) { (A, None) }
    fn update(&mut self, i: Self::Input<'_>) -> Self::Output { Some(Ok(*i.x)) }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(Ok(v)): A)]
pub struct B;
impl Component for B {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (B, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u64 { *i.v }
}
fn main() { let _ = Pipeline::<E, (B,)>::build(Configs::new()); }
EOT

# --- listener requests (Request<S>) --------------------------------------------------------------------

# A selector read downstream of a listener whose `Requests` has no `Request<S>` impl for it.
CMD[listener_missing_request]=check
PATTERN[listener_missing_request]='E0277.*cannot take the request of the selector'
read -r -d '' BODY[listener_missing_request] <<'EOF2'
use dataflow_flat::{Demux, Request};
pub mod m { pub struct Btc; pub struct Eth; }
#[derive(Clone, Copy, Demux)]
#[demux(markers = m)]
pub enum F { Btc(u64), Eth(u64) }
#[derive(Default)]
pub struct Reqs(Vec<u32>);
impl Request<m::Btc> for Reqs { fn add(&mut self) { self.0.push(0); } }

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Poll)]
pub struct L;
impl Component for L {
    type Requests = Reqs;
    type Config = ();
    type Output = Option<F>;
    fn new((): (), i: Self::Init<'_>) -> (Self, Option<F>) { let _ = i.requests.0; (L, None) }
    fn update(&mut self, _: Self::Input<'_>) -> Option<F> { None }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(m::Eth(t)): L)]
pub struct R;
impl Component for R {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (R, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u64 { *i.t }
}
fn main() { let _ = Pipeline::<Poll, (R,)>::build(Configs::new()); }
EOF2

# The same with the impl present for the selector that is read (and none for the unread one).
CMD[listener_request_ok]=check
PATTERN[listener_request_ok]=NOERROR
read -r -d '' BODY[listener_request_ok] <<'EOF2'
use dataflow_flat::{Demux, Request};
pub mod m { pub struct Btc; pub struct Eth; }
#[derive(Clone, Copy, Demux)]
#[demux(markers = m)]
pub enum F { Btc(u64), Eth(u64) }
#[derive(Default)]
pub struct Reqs(Vec<u32>);
impl Request<m::Btc> for Reqs { fn add(&mut self) { self.0.push(0); } }

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Poll)]
pub struct L;
impl Component for L {
    type Config = ();
    type Requests = Reqs;
    type Output = Option<F>;
    fn new((): (), i: Self::Init<'_>) -> (Self, Option<F>) { let _ = i.requests.0; (L, None) }
    fn update(&mut self, _: Self::Input<'_>) -> Option<F> { None }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Some(m::Btc(t)): L)]
pub struct R;
impl Component for R {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (R, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u64 { *i.t }
}
fn main() { let _ = Pipeline::<Poll, (R,)>::build(Configs::new()); }
EOF2


# --- handle components -----------------------------------------------------------------------------------------

HANDLE_H='
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Poll)]
pub struct H(u64);
impl Component for H {
    type Config = ();
    type Handle = ();
    type View<'"'"'a> = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, ()) { (H(0), ()) }
    fn update(&mut self, _: Self::Input<'"'"'_>) {}
    fn view<'"'"'a>(&'"'"'a self, (): &'"'"'a ()) -> u64 { self.0 }
}
'

# A handle component read by value and by a handle parent builds and runs.
CMD[handle_ok]=build
PATTERN[handle_ok]='NOERROR'
BODY[handle_ok]="$HANDLE_H
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(h: H, s: Src)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) { let h: u64 = i.h; (A, h) }
    fn update(&mut self, input: Self::Input<'_>) -> u64 { let h: u64 = input.h; h + *input.s }
}
fn main() { let _ = Pipeline::<Poll, (A,)>::build(Configs::new()); }"

# A handle component (no `type Output`) that did not write `fn view`: found when the code that reads
# it is built (not by `cargo check`), and the message names the component and says what to write.
CMD[handle_missing_view]=build
PATTERN[handle_missing_view]='H as .*Component>::VIEW_DEFINED.*needs .fn view.'
BODY[handle_missing_view]='
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Poll)]
pub struct H;
impl Component for H {
    type Config = ();
    type Handle = ();
    type View<'"'"'a> = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, ()) { (H, ()) }
    fn update(&mut self, _: Self::Input<'"'"'_>) {}
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(h: H)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, i: Self::Input<'"'"'_>) -> u64 { i.h }
}
fn main() { let _ = Pipeline::<Poll, (A,)>::build(Configs::new()); }'

# A component that sets neither `Output` nor `Handle`.
CMD[component_no_output_no_handle]=check
PATTERN[component_no_output_no_handle]='NeitherOutputNorHandleIsSet'
BODY[component_no_output_no_handle]='
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Poll)]
pub struct H;
impl Component for H {
    type Config = ();
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, ()) { (H, ()) }
    fn update(&mut self, _: Self::Input<'"'"'_>) {}
}
fn main() { let _ = Pipeline::<Poll, (H,)>::build(Configs::new()); }'

# `Output` set together with a `View` that is not `&Output`.
CMD[output_with_conflicting_view]=check
PATTERN[output_with_conflicting_view]='cannot be read as .&Output. by a subscriber.*of another type'
BODY[output_with_conflicting_view]='
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Poll)]
pub struct H;
impl Component for H {
    type Config = ();
    type Output = u64;
    type View<'"'"'a> = u32;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (H, 0) }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(h: H)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, i: Self::Input<'"'"'_>) -> u64 { 0 }
}
fn main() { let _ = Pipeline::<Poll, (A,)>::build(Configs::new()); }'

# `Output` set together with a `Handle` of another type.
CMD[output_with_conflicting_handle]=check
PATTERN[output_with_conflicting_handle]='cannot be read as .&Output. by a subscriber.*of another type'
BODY[output_with_conflicting_handle]='
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Poll)]
pub struct H;
impl Component for H {
    type Config = ();
    type Output = u64;
    type Handle = u32;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u32) { (H, 0) }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u32 { 0 }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(h: H)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, _: Self::Input<'"'"'_>) -> u64 { 0 }
}
fn main() { let _ = Pipeline::<Poll, (A,)>::build(Configs::new()); }'

# A handle component that is also a listener builds and runs (`Requests` is independent of `Handle`).
CMD[handle_with_listener]=build
PATTERN[handle_with_listener]='NOERROR'
BODY[handle_with_listener]='
use dataflow_flat::Request;
#[derive(Default)]
pub struct Reqs(u32);
impl Request<u8> for Reqs { fn add(&mut self) { self.0 += 1; } }
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Poll)]
pub struct HL(u64);
impl Component for HL {
    type Config = ();
    type Handle = ();
    type View<'"'"'a> = u64;
    type Requests = Reqs;
    fn new((): (), i: Self::Init<'"'"'_>) -> (Self, ()) { (HL(i.requests.0 as u64), ()) }
    fn update(&mut self, _: Self::Input<'"'"'_>) {}
    fn view<'"'"'a>(&'"'"'a self, (): &'"'"'a ()) -> u64 { self.0 }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(h: HL)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, i: Self::Input<'"'"'_>) -> u64 { i.h }
}
fn main() { let _ = Pipeline::<Poll, (A,)>::build(Configs::new()); }'

# A handle component that is a listener: a selector read through its view whose `Requests` has no
# `Request<S>` impl for it is the same error as for a by-value listener.
CMD[handle_listener_missing_request]=check
PATTERN[handle_listener_missing_request]='E0277.*cannot take the request of the selector'
read -r -d '' BODY[handle_listener_missing_request] <<'EOF2'
use dataflow_flat::{Demux, Request};
pub mod m { pub struct Btc; pub struct Eth; }
#[derive(Demux)]
#[demux(markers = m)]
pub enum V<'a> { Btc(&'a u64), Eth(&'a u64) }
#[derive(Default)]
pub struct Reqs(Vec<u32>);
impl Request<m::Btc> for Reqs { fn add(&mut self) { self.0.push(0); } }

#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Poll)]
pub struct L(u64);
impl Component for L {
    type Config = ();
    type Handle = ();
    type View<'a> = V<'a>;
    type Requests = Reqs;
    fn new((): (), i: Self::Init<'_>) -> (Self, ()) { let _ = i.requests.0; (L(0), ()) }
    fn update(&mut self, _: Self::Input<'_>) {}
    fn view<'a>(&'a self, (): &'a ()) -> V<'a> { V::Btc(&self.0) }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(m::Eth(t): L)]
pub struct R;
impl Component for R {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (R, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u64 { *i.t }
}
fn main() { let _ = Pipeline::<Poll, (R,)>::build(Configs::new()); }
EOF2

# A handler (unnamed entry, `on_update` gets the handle parent's view exactly) over a handle component builds.
CMD[handle_handler_ok]=build
PATTERN[handle_handler_ok]='NOERROR'
BODY[handle_handler_ok]="$HANDLE_H
#[derive(ComponentSubscription)]
#[subscribe(H, s: Src)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, input: Self::Input<'_>) -> u64 { *input.s }
}
impl UpdateHandler<H> for A { fn on_update(&mut self, _: Self::Input<'_>, v: u64) { let _: u64 = v; } }
impl UpdateHandler<Src> for A { const ACTIVE: bool = false; }
fn main() { let _ = Pipeline::<Poll, (A,)>::build(Configs::new()); }"

# A handle component whose view is an enum of borrowed and owned variants (selectors on it work).
HANDLE_V='
pub struct Big(u64);
#[derive(dataflow_flat::Demux)]
pub enum Vw<'"'"'a> { Big(&'"'"'a Big), Small(u64) }
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Poll)]
pub struct HV(Big);
impl Component for HV {
    type Config = ();
    type Handle = ();
    type View<'"'"'a> = Vw<'"'"'a>;
    fn new((): (), _: Self::Init<'"'"'_>) -> (Self, ()) { (HV(Big(0)), ()) }
    fn update(&mut self, _: Self::Input<'"'"'_>) {}
    fn view<'"'"'a>(&'"'"'a self, (): &'"'"'a ()) -> Vw<'"'"'a> { Vw::Big(&self.0) }
}
'

# A selector on a handle component: the field is exactly the variant's field type (`&Big`, `u64`),
# and `new` gets an `Option` of it.
CMD[handle_selector_ok]=build
PATTERN[handle_selector_ok]='NOERROR'
BODY[handle_selector_ok]="$HANDLE_V
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Vw::Big(b): HV)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) { let b: Option<&Big> = i.b; (A, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u64 { let b: &Big = i.b; b.0 }
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Vw::Small(n): HV)]
pub struct B;
impl Component for B {
    type Config = ();
    type Output = u64;
    fn new((): (), i: Self::Init<'_>) -> (Self, u64) { let n: Option<u64> = i.n; (B, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u64 { let n: u64 = i.n; n }
}
fn main() { let _ = Pipeline::<Poll, (A, B)>::build(Configs::new()); }"

# A by-value selector (a `Demux` enum without a lifetime, `Select<E>`) on a handle source: the message
# says that a view needs view selectors.
CMD[handle_selector_by_value]=check
PATTERN[handle_selector_by_value]='cannot be applied to the view'
BODY[handle_selector_by_value]="$HANDLE_H
#[derive(Clone, Copy, dataflow_flat::Demux)]
pub enum E { X(u8), Y }
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(E::X(v): H)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, _: Self::Input<'_>) -> u64 { 0 }
}
fn main() {}"

# Selectors of another view enum on a handle source: a type mismatch with a clear message.
CMD[handle_selector_wrong_view]=check
PATTERN[handle_selector_wrong_view]='cannot be applied to the view'
BODY[handle_selector_wrong_view]="$HANDLE_V
#[derive(dataflow_flat::Demux)]
pub enum Other<'a> { Big(&'a Big), Tiny(u8) }
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Other::Tiny(t): HV)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, _: Self::Input<'_>) -> u64 { 0 }
}
fn main() {}"

# View selectors on a by-value source: the message says a stored output needs `Select`.
CMD[handle_selector_view_on_by_value]=check
PATTERN[handle_selector_view_on_by_value]='the selector chain .* cannot be applied to a value of type'
BODY[handle_selector_view_on_by_value]="$HANDLE_V
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Vw::Small(n): Src)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, _: Self::Input<'_>) -> u64 { 0 }
}
fn main() {}"

# A selected input of a handle source that would have to be stored (the subscriber also runs when
# the selectors do not match): a plan error naming the input and what to do instead.
CMD[handle_selector_stored]=build
PATTERN[handle_selector_stored]='the input .n. selects from a handle component.*a view is not stored.*by-value component'
BODY[handle_selector_stored]="$HANDLE_V
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(Vw::Small(n): HV, s: Src)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u64 { i.n + *i.s }
}
fn main() { let _ = Pipeline::<Poll, (A,)>::build(Configs::new()); }"

# Generic code written for `Node<Output = u64>` does not accept a handle component.
CMD[handle_generic_node_bound]=check
# rustc reports the projection mismatch of the `Node` impl before any marker trait, so the message is
# `type mismatch resolving <H as Component>::Handle == HandleComponentHasNoOutputSoItIsNotAByValueNode`
# (the sentinel's name says what is wrong) followed by `required for H to implement Node`.
PATTERN[handle_generic_node_bound]='E0271.*Handle == .*HandleComponentHasNoOutputSoItIsNotAByValueNode.*required for .H. to implement .Node.'
BODY[handle_generic_node_bound]="$HANDLE_H
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(x: X)]
pub struct G<X: dataflow_flat::Node<Output = u64>>(std::marker::PhantomData<X>);
impl<X: dataflow_flat::Node<Output = u64>> Component for G<X> {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (G(std::marker::PhantomData), 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u64 { *i.x }
}
fn main() { let _: Option<G<H>> = None; }"

# A handle component has no `Output` stored: asking a subscriber for `&Output` is a type error.
CMD[handle_field_is_the_view]=check
PATTERN[handle_field_is_the_view]='E0614|E0308|mismatched types'
BODY[handle_field_is_the_view]="$HANDLE_H
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(h: H)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, i: Self::Input<'_>) -> u64 { *i.h }
}
fn main() {}"

ORDER=(field_typo not_a_component duplicate_name wrong_input_type cycle
       self_cycle
       too_many_components root_not_a_root sink_is_the_root
       generic_missing_sinks_bound generic_sinks_bound_ok
       unreachable_component second_root wrong_root_type
       keep_input_reference name_input_struct_outside unused_config
       non_exclusive_join exclusive_join_ok deep_non_exclusive_join deep_exclusive_join_ok demux_reader_not_implied stored_selector_ok
       stored_payload_no_default stored_payload_no_clone unstored_payload_ok selector_unreachable_component
       selector_wrong_input orphan_first
       handler_missing_update handler_missing_update_note handler_non_exclusive_join handler_exclusive_ok handler_duplicate_marker handler_selected_missing_update handler_bare_wildcard handler_named_missing_update handler_derive_and_impl handler_wrong_value_type handler_wrong_edge_type
       for_ok for_missing_update for_not_a_catalog for_zip_shapes for_body_named for_body_selected for_pattern_triple for_missing_bound
       pattern_struct pattern_multi_field pattern_range pattern_range_literal pattern_guard pattern_or pattern_literal
       pattern_reference pattern_arrow unit_selector_unknown unit_selector_data_variant pattern_ok
       listener_missing_request listener_request_ok
       handle_ok handle_handler_ok handle_missing_view component_no_output_no_handle output_with_conflicting_view output_with_conflicting_handle handle_with_listener handle_listener_missing_request handle_selector_ok handle_selector_by_value handle_selector_wrong_view handle_selector_view_on_by_value handle_selector_stored handle_generic_node_bound handle_field_is_the_view)

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
