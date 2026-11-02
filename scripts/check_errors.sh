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
        #[derive(ComponentSubscription)]
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

# Rule 5: a selected edge read in steps where its selectors do not match (J also runs when K1 matches).
CMD[demux_reader_not_implied]=build
PATTERN[demux_reader_not_implied]='E0080.*J reads .*Poll through selectors in steps where they do not match; subscribe to a component that stores it'
BODY[demux_reader_not_implied]="$SELECTORS
comp!(A, [K0(p): Poll]);
comp!(J, [a: A, K1(p): Poll]);
fn main() { let _ = Pipeline::<Poll, (J,)>::build(Configs::new()); }"

# Rule 6 through selectors: B is reached from the root only via a selector on a component nothing connects.
CMD[selector_unreachable_component]=build
PATTERN[selector_unreachable_component]='E0080.*Orphan is not reachable from the root'
read -r -d '' BODY[selector_unreachable_component] <<'EOF'
#[derive(ComponentSubscription)]
pub struct Orphan;
impl Component for Orphan {
    type Config = ();
    type Output = Option<u64>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Option<u64>) { (Orphan, None) }
    fn update(&mut self, _: Self::Input<'_>) -> Option<u64> { None }
}
#[derive(ComponentSubscription)]
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
#[derive(ComponentSubscription)]
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

# --- keyed selectors (On<S>) ---------------------------------------------------------------------------

# Rule 6 when the orphan is the first leaf (it takes execution position 0, not the root): reported by the
# plan, not as an arithmetic underflow in the component position.
CMD[orphan_first]=build
PATTERN[orphan_first]='E0080.*Orphan is not reachable from the root'
read -r -d '' BODY[orphan_first] <<'EOF2'
#[derive(ComponentSubscription)]
pub struct Orphan;
impl Component for Orphan {
    type Config = ();
    type Output = Option<u64>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Option<u64>) { (Orphan, None) }
    fn update(&mut self, _: Self::Input<'_>) -> Option<u64> { None }
}
#[derive(ComponentSubscription)]
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

# On<S> on a parent whose output is not Keyed.
CMD[on_not_keyed]=check
PATTERN[on_not_keyed]='E0277.*On<_>. needs an output that implements .Keyed.'
read -r -d '' BODY[on_not_keyed] <<'EOF2'
use dataflow_flat::{On, Symbol};
pub struct Btc;
impl Symbol for Btc { const KEY: u32 = 0; const NAME: &'static str = "Btc"; }
#[derive(ComponentSubscription)]
#[subscribe(On::<Btc>(x): Src)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, _: Self::Input<'_>) -> u64 { 0 }
}
fn main() {}
EOF2

# Two different symbol types with one KEY on one parent.
CMD[symbols_share_key]=build
PATTERN[symbols_share_key]='E0080.*symbols Btc and Xbt share key 0'
read -r -d '' BODY[symbols_share_key] <<'EOF2'
use dataflow_flat::{Keyed, On, Symbol};
pub struct Btc;
impl Symbol for Btc { const KEY: u32 = 0; const NAME: &'static str = "Btc"; }
pub struct Xbt;
impl Symbol for Xbt { const KEY: u32 = 0; const NAME: &'static str = "Xbt"; }
pub struct Frame(u32, u64);
impl Keyed for Frame {
    type Item = u64;
    fn key(&self) -> Option<u32> { Some(self.0) }
    fn item(&self) -> &u64 { &self.1 }
}
#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
pub struct Ws;
impl Component for Ws {
    type Config = ();
    type Output = Frame;
    fn new((): (), _: Self::Init<'_>) -> (Self, Frame) { (Ws, Frame(0, 0)) }
    fn update(&mut self, _: Self::Input<'_>) -> Frame { Frame(0, 0) }
}
#[derive(ComponentSubscription)]
#[subscribe(On::<Btc>(a): Ws, On::<Xbt>(b): Ws)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) { (A, 0) }
    fn update(&mut self, _: Self::Input<'_>) -> u64 { 0 }
}
fn main() { let _ = Pipeline::<Poll, (A,)>::build(Configs::new()); }
EOF2

# --- handler components ------------------------------------------------------------------------------
read -r -d '' HANDLERS <<'EOF'
use dataflow_flat::{Collect, Handler, UpdateHandler};
macro_rules! handler {
    ($name:ident, [$($attr:tt)*]) => {
        #[derive(Handler)]
        $($attr)*
        pub struct $name;
        impl Collect for $name {
            type Config = ();
            type Output = u64;
            type Buf = u64;
            fn new((): ()) -> Self { $name }
            fn begin(&mut self) -> u64 { 0 }
            fn finish(&mut self, buf: u64, _: Self::Input<'_>) -> u64 { buf }
        }
    };
}
EOF

# A handled edge without an `UpdateHandler` impl: the on_unimplemented message.
CMD[handler_missing_update]=check
PATTERN[handler_missing_update]='E0277.*does not handle the edge|does not handle the edge.*E0277'
BODY[handler_missing_update]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
comp!(B, [K1(p): Poll]);
handler!(H, [#[handles(A, B)]]);
impl UpdateHandler<A> for H { fn on_update(&mut self, buf: &mut u64, v: &u64) { *buf += *v; } }
fn main() { let _ = Pipeline::<Poll, (H,)>::build(Configs::new()); }"

# Rule 4 still applies to handled edges: two guards can fire in the same step.
CMD[handler_non_exclusive_join]=build
PATTERN[handler_non_exclusive_join]='E0080.*H joins branches that can fire in the same step'
BODY[handler_non_exclusive_join]="$SELECTORS
$HANDLERS
comp!(A, [G1(p): Poll]);
comp!(B, [G2(p): Poll]);
handler!(H, [#[handles(A, B)]]);
impl UpdateHandler<A> for H { fn on_update(&mut self, buf: &mut u64, v: &u64) { *buf += *v; } }
impl UpdateHandler<B> for H { fn on_update(&mut self, buf: &mut u64, v: &u64) { *buf += *v; } }
fn main() { let _ = Pipeline::<Poll, (H,)>::build(Configs::new()); }"

# Exclusive handled edges compile (and a handled edge is not subject to rule 5).
CMD[handler_exclusive_ok]=build
PATTERN[handler_exclusive_ok]='NOERROR'
BODY[handler_exclusive_ok]="$SELECTORS
$HANDLERS
comp!(A, [K0(p): Poll]);
comp!(B, [K1(p): Poll]);
handler!(H, [#[handles(A, B)] #[handles(K0(sel): Poll)]]);
impl UpdateHandler<A> for H { fn on_update(&mut self, buf: &mut u64, v: &u64) { *buf += *v; } }
impl UpdateHandler<B> for H { fn on_update(&mut self, buf: &mut u64, v: &u64) { *buf += *v; } }
impl UpdateHandler<h_edges::Sel> for H { fn on_update(&mut self, buf: &mut u64, v: &Poll) { *buf += 1; } }
fn main() { let _ = Pipeline::<Poll, (H,)>::build(Configs::new()); }"

# A plain `#[subscribe]` input of a handler can't carry selectors.
CMD[handler_selected_input]=check
PATTERN[handler_selected_input]="can't use selectors"
BODY[handler_selected_input]="$SELECTORS
$HANDLERS
handler!(H, [#[subscribe(K0(p): Poll)]]);
fn main() {}"

# A plain input that is the root has no value at build, when `finish` runs for the initial output.
CMD[handler_root_input]=check
PATTERN[handler_root_input]='plain `#\[subscribe\]` input of a handler must be a component'
BODY[handler_root_input]="$HANDLERS
handler!(H, [#[handles(Src)] #[subscribe(p: Poll)]]);
impl UpdateHandler<Src> for H { fn on_update(&mut self, buf: &mut u64, v: &u64) { *buf += *v; } }
fn main() {}"


# --- subscription patterns (`Some(Btc(t)): Src`): sugar over selector chains ------------------------------
# pat_case <name> <pattern> <regex of our message>; the pattern is a subscription of component `A`.
pat_case() {
    CMD[$1]=check
    PATTERN[$1]="subscription pattern: $3"
    BODY[$1]="
#[derive(ComponentSubscription)]
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
pat_case pattern_wildcard 'Some(_)' 'a wildcard `_` is not a selector'
pat_case pattern_wildcard_selector '_(x)' 'a wildcard `_` is not a selector'
pat_case pattern_reference 'Some(&x)' 'references are not supported'

# `=>` is gone: the message says how to write the pattern.
CMD[pattern_arrow]=check
PATTERN[pattern_arrow]='`=>` is gone: write the selectors as a pattern'
BODY[pattern_arrow]='
#[derive(ComponentSubscription)]
#[subscribe(x: Src => Some)]
pub struct A;
fn main() {}'

# A bare uppercase name is a unit selector; one that names no type is rustc's E0425 "cannot find type" (at the struct).
CMD[unit_selector_unknown]=check
PATTERN[unit_selector_unknown]='E0425.*cannot find type `Disconnected` in this scope'
BODY[unit_selector_unknown]='
#[derive(ComponentSubscription)]
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
#[derive(ComponentSubscription)]
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
#[derive(ComponentSubscription)]
#[subscribe(E::X(x): E)]
pub struct A;
impl Component for A {
    type Config = ();
    type Output = Option<Result<u64, u64>>;
    fn new((): (), _: Self::Init<'_>) -> (Self, Self::Output) { (A, None) }
    fn update(&mut self, i: Self::Input<'_>) -> Self::Output { Some(Ok(*i.x)) }
}
#[derive(ComponentSubscription)]
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
use dataflow_flat::{Demux, Listener, Request};
pub mod m { pub struct Btc; pub struct Eth; }
#[derive(Clone, Copy, Demux)]
#[demux(markers = m)]
pub enum F { Btc(u64), Eth(u64) }
#[derive(Default)]
pub struct Reqs(Vec<u32>);
impl Request<m::Btc> for Reqs { fn add(&mut self) { self.0.push(0); } }

#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
#[listener]
pub struct L;
impl Listener for L { type Requests = Reqs; }
impl Component for L {
    type Config = ();
    type Output = Option<F>;
    fn new((): (), i: Self::Init<'_>) -> (Self, Option<F>) { let _ = i.requests.0; (L, None) }
    fn update(&mut self, _: Self::Input<'_>) -> Option<F> { None }
}
#[derive(ComponentSubscription)]
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
use dataflow_flat::{Demux, Listener, Request};
pub mod m { pub struct Btc; pub struct Eth; }
#[derive(Clone, Copy, Demux)]
#[demux(markers = m)]
pub enum F { Btc(u64), Eth(u64) }
#[derive(Default)]
pub struct Reqs(Vec<u32>);
impl Request<m::Btc> for Reqs { fn add(&mut self) { self.0.push(0); } }

#[derive(ComponentSubscription)]
#[subscribe(p: Poll)]
#[listener]
pub struct L;
impl Listener for L { type Requests = Reqs; }
impl Component for L {
    type Config = ();
    type Output = Option<F>;
    fn new((): (), i: Self::Init<'_>) -> (Self, Option<F>) { let _ = i.requests.0; (L, None) }
    fn update(&mut self, _: Self::Input<'_>) -> Option<F> { None }
}
#[derive(ComponentSubscription)]
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

ORDER=(field_typo not_a_component duplicate_name wrong_input_type cycle
       self_cycle
       too_many_components root_not_a_root sink_is_the_root
       generic_missing_sinks_bound generic_sinks_bound_ok
       unreachable_component second_root wrong_root_type
       keep_input_reference name_input_struct_outside unused_config
       non_exclusive_join exclusive_join_ok deep_non_exclusive_join deep_exclusive_join_ok demux_reader_not_implied selector_unreachable_component
       selector_wrong_input orphan_first on_not_keyed symbols_share_key
       handler_missing_update handler_non_exclusive_join handler_exclusive_ok handler_selected_input handler_root_input
       pattern_struct pattern_multi_field pattern_range pattern_range_literal pattern_guard pattern_or pattern_literal
       pattern_wildcard pattern_wildcard_selector pattern_reference pattern_arrow unit_selector_unknown unit_selector_data_variant pattern_ok
       listener_missing_request listener_request_ok)

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
