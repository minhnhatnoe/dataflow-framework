# usage: python3 bench/gen_bench.py full|flat|hand [layers=20]  -> writes bench/src/main.rs
import random, sys
mode = sys.argv[1]
random.seed(1)
L, W, D = int(sys.argv[2]) if len(sys.argv) > 2 else 20, 20, 4
deps = {}
for l in range(L):
    for w in range(W):
        deps[(l,w)] = [] if l == 0 else [(l-1, d) for d in random.sample(range(W), D)]
roots = [(L-1, w) for w in range(W)]
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
    out=["use dataflow_flat::{Component, ComponentSubscription, Deps};"]
    for n, ds in deps.items():
        nm = name(n)
        subs = ", ".join(f"s{i}: {name(d)}" for i, d in enumerate(ds))
        attr = f"#[subscribe({subs})]\n" if ds else ""
        out.append(f"""#[derive(ComponentSubscription)]
{attr}pub struct {nm}(pub u64);
impl Component for {nm} {{
    type Config = ();
    type Output = u64;
    fn new((): ()) -> Self {{ {nm}({n[1]}) }}
    #[inline]
    fn update(&mut self, input: Self::Input<'_>) -> u64 {{ {body_of(n, [f"input.s{i}" for i in range(len(ds))])} }}
}}""")
    out.append(f"pub type Roots = Deps![{', '.join(name(r) for r in roots)}];")
    return "\n".join(out)

def nest(xs):
    s = "()"
    for x in reversed(xs): s = f"({x}, {s})"
    return s
pat = ", ".join(f"r{i}" for i in range(len(roots)))
xor = " ^ ".join(f"*r{i}" for i in range(len(roots)))
out = ["#![allow(non_snake_case, unused_imports, dead_code)]", "use std::hint::black_box;", "use std::time::Instant;", ""]
use_flat = mode in ("full", "flat")
use_hand = mode in ("full", "hand")
if use_flat: out.append("mod flat {\n" + components_flat() + "\n}")
if use_hand:
    out.append("mod hand_components {\n" + components_hand() + "\n}\nuse hand_components::*;\npub struct Hand {")
    for n in order: out.append(f"    c_{name(n)}: {name(n)}, o_{name(n)}: u64,")
    out.append("}\nimpl Hand {\n    fn new() -> Self { Hand {")
    for n in order: out.append(f"        c_{name(n)}: {name(n)}::new(), o_{name(n)}: 0,")
    out.append("    } }\n    #[inline(never)]\n    fn run(&mut self) {")
    for n in order:
        out.append(f"        self.o_{name(n)} = self.c_{name(n)}.update({', '.join(f'&self.o_{name(d)}' for d in deps[n])});")
    out.append("    }\n    fn roots(&self) -> u64 { 0" + "".join(f" ^ self.o_{name(r)}" for r in roots) + " }\n}")
out.append("const RUNS: u32 = 200_000;")
out.append("fn ns(d: std::time::Duration) -> f64 { d.as_nanos() as f64 / RUNS as f64 }")
body = ["fn main() {"]
if use_flat: body.append(f"""    let mut b = dataflow_flat::Pipeline::<flat::Roots>::build(dataflow_flat::Configs::new()).unwrap();
    let t = Instant::now(); for _ in 0..RUNS {{ run_flat(&mut b); }}
    let dataflow_flat::deps!({pat}) = b.outputs().unwrap(); let xb = {xor}; black_box(xb);
    println!("flat:         {{:>6.0}} ns/run  ({{}} components)", ns(t.elapsed()), b.len());""")
if use_hand: body.append(f"""    let mut h = Hand::new();
    let t = Instant::now(); for _ in 0..RUNS {{ h.run(); }}
    let xh = h.roots(); black_box(xh);
    println!("hand-written: {{:>6.0}} ns/run", ns(t.elapsed()));""")
if use_flat and use_hand: body.append('    assert_eq!(xb, xh); println!("results agree");')
body.append("}")
if use_flat: out.append("#[inline(never)]\nfn run_flat(p: &mut dataflow_flat::Pipeline<flat::Roots>) { p.run() }")
out += body
open(__import__("os").path.join(__import__("os").path.dirname(__file__), "src", "main.rs"),"w").write("\n".join(out))
print(f"mode={mode}: {len(order)} components, {sum(len(deps[n]) for n in order)} edges")
