# usage: python3 bench/gen_bench.py full|flat|hand [layers=20] [wide]  -> writes bench/src/main.rs
# `wide`: the root subscribes to the last 5 layers (100 components at 20 wide) instead of the last one.
# `flat` generates the root `Poll`, the layer-0 components subscribe to it, and one sink `App`
# (subscribes to the last layer, stores the XOR checksum in a static); every component runs on every
# push (event-model phase 1a, no demuxers); `full` also generates the hand-written baseline and asserts the results agree.
import random, sys
mode = sys.argv[1]
random.seed(1)
# Further arguments (any order after `layers`): `wide`, `d=N` (deps per component, default 4,
# sampled from the previous layer), `w=N` (components per layer, default 20), `root=N` (the root
# subscribes to the last N components, layer-major; default 20, or 100 with `wide`), `rl=N`
# (adds #![recursion_limit = "N"] to the generated crate).
extra = {a.split("=")[0]: int(a.split("=")[1]) for a in sys.argv[3:] if "=" in a}
L, W, D = int(sys.argv[2]) if len(sys.argv) > 2 else 20, extra.get("w", 20), extra.get("d", 4)
deps = {}
for l in range(L):
    for w in range(W):
        deps[(l,w)] = [] if l == 0 else [(l-1, d) for d in random.sample(range(W), D)]
WIDE = "wide" in sys.argv[3:]
roots = [(l, w) for l in range(L - (5 if WIDE else 1), L) for w in range(W)]
if "root" in extra: roots = [(l, w) for l in range(L) for w in range(W)][-extra["root"]:]
order, seen = [], set()
def visit(n):
    if n in seen: return
    for d in deps[n]: visit(d)
    seen.add(n); order.append(n)
for r in roots: visit(r)
name = lambda n: f"C{n[0]}_{n[1]}"
def body_of(n, args):
    if not args:
        return f"self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add({n[1]+1}); self.0"
    return "(" + " ^ ".join(f"{a}.rotate_left({i+1})" for i, a in enumerate(args)) + f").wrapping_add({n[0]*31+n[1]})"

def components_hand():
    # Plain structs with inherent methods: the hand-written baseline.
    out=[]
    for n, ds in deps.items():
        nm = name(n); params = "".join(f", d{i}: &u64" for i in range(len(ds)))
        out.append(f"""pub struct {nm}(pub u64);
impl {nm} {{
    pub fn new() -> Self {{ {nm}({n[1]}) }}
    #[inline]
    pub fn update(&mut self{params}) -> u64 {{ {body_of(n, [f"d{i}" for i in range(len(ds))])} }}
}}""")
    return "\n".join(out)

def components_flat():
    # Nightly `dataflow-flat` crate: derive syntax.
    out=["use dataflow_flat::{Component, ComponentSubscription, Root};",
         "use std::sync::atomic::{AtomicU64, Ordering};",
         "#[derive(Root)]\npub struct Poll;",
         "pub static OUT: AtomicU64 = AtomicU64::new(0);"]
    for n, ds in deps.items():
        nm = name(n)
        subs = ", ".join(f"s{i}: {name(d)}" for i, d in enumerate(ds)) if ds else "p: Poll"
        attr = f"#[subscribe({subs})]\n"
        out.append(f"""#[derive(ComponentSubscription)]
{attr}pub struct {nm}(pub u64);
impl Component for {nm} {{
    type Config = ();
    type Output = u64;
    fn new((): (), _: Self::Init<'_>) -> (Self, u64) {{ ({nm}({n[1]}), 0) }}
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 {{ {body_of(n, [f"input.s{i}" for i in range(len(ds))])} }}
}}""")
    # The single root: subscribes to the last layer; its output is the XOR checksum.
    subs = ", ".join(f"r{i}: {name(r)}" for i, r in enumerate(roots))
    xor_in = " ^ ".join(f"*input.r{i}" for i in range(len(roots)))
    out.append(f"""#[derive(ComponentSubscription)]
#[subscribe({subs})]
pub struct App;
impl Component for App {{
    type Config = ();
    type Output = ();
    fn new((): (), _: Self::Init<'_>) -> (Self, ()) {{ (App, ()) }}
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) {{ OUT.store({xor_in}, Ordering::Relaxed) }}
}}""")
    return "\n".join(out)

def nest(xs):
    s = "()"
    for x in reversed(xs): s = f"({x}, {s})"
    return s
out = ["#![allow(non_snake_case, unused_imports, dead_code)]"] + ([f'#![recursion_limit = "{extra["rl"]}"]'] if "rl" in extra else []) + ["use std::hint::black_box;", "use std::time::Instant;", ""]
use_flat = mode in ("full", "flat")
use_hand = mode in ("full", "hand")
if use_flat: out.append("mod flat {\n" + components_flat() + "\n}")
if use_hand:
    out.append("mod hand_components {\n" + components_hand() + "\n}\nuse hand_components::*;\npub struct Hand {")
    # Fair baseline: like the framework, every component's latest output is stored in the struct
    # (persisting across runs), not kept in a local.
    for n in order: out.append(f"    c_{name(n)}: {name(n)},\n    o_{name(n)}: u64,")
    out.append("}\nimpl Hand {\n    fn new() -> Self { Hand {")
    for n in order: out.append(f"        c_{name(n)}: {name(n)}::new(),\n        o_{name(n)}: 0,")
    out.append("    } }\n    #[inline(never)]\n    fn run(&mut self) -> u64 {")
    for n in order:
        out.append(f"        self.o_{name(n)} = self.c_{name(n)}.update({', '.join(f'&self.o_{name(d)}' for d in deps[n])});")
    out.append("        0" + "".join(f" ^ self.o_{name(r)}" for r in roots) + "\n    }\n}")
out.append("""const WARMUP: u32 = 5_000; // untimed runs per side
const RUNS: u32 = 50_000; // timed runs per side per round
const ROUNDS: usize = 11; // timed rounds; the side that runs first alternates
fn stats(v: &mut Vec<f64>) -> (f64, f64) {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    (v[0], v[v.len() / 2])
}""")
body = ["fn main() {"]
if use_flat: body.append("""    let mut b = dataflow_flat::Pipeline::<flat::Poll, (flat::App,)>::build(dataflow_flat::Configs::new()).unwrap();
    let mut xb = 0;
    let mut tf: Vec<f64> = Vec::new();
    for _ in 0..WARMUP { xb = run_flat(&mut b); }""")
if use_hand: body.append("""    let mut h = Hand::new();
    let mut xh = 0;
    let mut th: Vec<f64> = Vec::new();
    for _ in 0..WARMUP { xh = h.run(); }""")
blk_f = """{ let t = Instant::now(); for _ in 0..RUNS { xb = run_flat(&mut b); } let el = t.elapsed(); black_box(xb); tf.push(el.as_nanos() as f64 / RUNS as f64); }"""
blk_h = """{ let t = Instant::now(); for _ in 0..RUNS { xh = h.run(); } let el = t.elapsed(); black_box(xh); th.push(el.as_nanos() as f64 / RUNS as f64); }"""
if use_flat and use_hand:
    body.append(f"    for r in 0..ROUNDS {{ if r % 2 == 0 {{ {blk_f} {blk_h} }} else {{ {blk_h} {blk_f} }} }}")
elif use_flat: body.append(f"    for _ in 0..ROUNDS {{ {blk_f} }}")
else: body.append(f"    for _ in 0..ROUNDS {{ {blk_h} }}")
if use_flat: body.append("""    let (mn, md) = stats(&mut tf);
    println!("flat:         min {:>6.1}  median {:>6.1} ns/run  ({} components, {} rounds x {} runs)", mn, md, b.len(), ROUNDS, RUNS);""")
if use_hand: body.append("""    let (mn, md) = stats(&mut th);
    println!("hand-written: min {:>6.1}  median {:>6.1} ns/run", mn, md);""")
if use_flat and use_hand: body.append('    assert_eq!(xb, xh); println!("results agree");')
body.append("}")
if use_flat: out.append("#[inline(never)]\nfn run_flat(p: &mut dataflow_flat::Pipeline<flat::Poll, (flat::App,)>) -> u64 { p.push(flat::Poll); flat::OUT.load(std::sync::atomic::Ordering::Relaxed) }")
out += body
open(__import__("os").path.join(__import__("os").path.dirname(__file__), "src", "main.rs"),"w").write("\n".join(out))
print(f"mode={mode}: {len(order)} components, {sum(len(deps[n]) for n in order)} edges")
