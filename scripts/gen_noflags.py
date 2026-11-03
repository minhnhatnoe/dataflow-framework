#!/usr/bin/env python3
"""Random graphs for the differential check of the push (no stored "fired" bits).

Writes a scratch crate (Cargo.toml + src/main.rs) that builds `--graphs` random pipelines,
pushes `--packets` random packets into each and writes every construct / update / on_update
event, in order with its values, to the file given as argv[1]. Build it against two checkouts
of the framework (`--lib PATH`) and compare the two logs.

Component kinds (all outputs are u32):
  A: one edge: a keyed variant of Dm's enum output, a guard on the root, a plain edge to an
     earlier component, or a guard on an earlier component's output (a stale parent: its
     stored output keeps matching in pushes where it did not run).
  B: always runs (plain root edge) plus 1..3 plain edges to earlier components, each with an
     `on_update` (random), so a reader of independent guards / of an order-split parent.
  W: (graphs with i % 4 == 3) a wide B with 25 handled edges.
"""
import argparse
import os
import random

p = argparse.ArgumentParser()
p.add_argument("--out", required=True)
p.add_argument("--lib", required=True, help="path of the dataflow-flat crate")
p.add_argument("--graphs", type=int, default=12)
p.add_argument("--packets", type=int, default=3000)
p.add_argument("--seed", type=int, default=1)
a = p.parse_args()

head = """#![allow(dead_code)]
use dataflow_flat::{Component, ComponentSubscription, Configs, Demux, Pipeline, Root, Select, UpdateHandler, GUARD};
use std::fmt::Write as _;
use std::sync::Mutex;

static LOG: Mutex<String> = Mutex::new(String::new());
fn log(s: String) {
    let mut l = LOG.lock().unwrap();
    l.push_str(&s);
    l.push('\\n');
}

#[derive(Root)]
pub struct Pk {
    pub t: u8,
    pub a: u32,
    pub b: u32,
}
#[derive(Demux)]
pub enum Ev {
    V0(u32),
    V1(u32),
    V2(u32),
    V3(u32),
}
pub struct G1;
impl Select<Pk> for G1 {
    type Out = u32;
    const KEY: u32 = GUARD;
    fn select(x: &Pk) -> Option<&u32> { (x.a % 3 == 0).then_some(&x.a) }
    fn key_of(_: &Pk) -> u32 { GUARD }
}
pub struct G2;
impl Select<Pk> for G2 {
    type Out = u32;
    const KEY: u32 = GUARD;
    fn select(x: &Pk) -> Option<&u32> { (x.b > 50).then_some(&x.b) }
    fn key_of(_: &Pk) -> u32 { GUARD }
}
pub struct Big;
impl Select<u32> for Big {
    type Out = u32;
    const KEY: u32 = GUARD;
    fn select(x: &u32) -> Option<&u32> { (*x >= 50).then_some(x) }
    fn key_of(_: &u32) -> u32 { GUARD }
}
pub struct Odd;
impl Select<u32> for Odd {
    type Out = u32;
    const KEY: u32 = GUARD;
    fn select(x: &u32) -> Option<&u32> { (*x % 2 == 1).then_some(x) }
    fn key_of(_: &u32) -> u32 { GUARD }
}
/// The view of a handle component: a borrowed variant and an owned one.
#[derive(Demux)]
pub enum HV<'a> {
    Big(&'a u64),
    Small(u64),
}
#[derive(ComponentSubscription, UpdateHandler)]
#[subscribe(p: Pk)]
pub struct Dm;
impl Component for Dm {
    type Config = ();
    type Output = Ev;
    fn new((): (), _: Self::Init<'_>) -> (Self, Ev) { log("new Dm".into()); (Dm, Ev::V3(0)) }
    fn update(&mut self, i: Self::Input<'_>) -> Ev {
        log(format!("Dm upd {} {}", i.p.t, i.p.a));
        match i.p.t % 4 { 0 => Ev::V0(i.p.a), 1 => Ev::V1(i.p.b), 2 => Ev::V2(i.p.a + i.p.b), _ => Ev::V3(i.p.a) }
    }
}
"""

HANDLE = '''
    /// A handle component behind a guard: its view is stale in pushes where it did not run.
    #[derive(ComponentSubscription, UpdateHandler)]
    #[subscribe(G1(v): Pk)]
    pub struct HF { ring: [u64; 2], n: usize }
    impl Component for HF {
        type Config = ();
        type Handle = usize;
        type View<'a> = HV<'a>;
        fn new((): (), _: Self::Init<'_>) -> (Self, usize) { log("gGG new HF".into()); (HF { ring: [0; 2], n: 0 }, 0) }
        fn update(&mut self, i: Self::Input<'_>) -> usize {
            self.n += 1;
            let s = self.n % 2;
            self.ring[s] = *i.v as u64;
            log(format!("gGG HF upd {}", i.v));
            s
        }
        fn view<'a>(&'a self, h: &'a usize) -> HV<'a> {
            if self.ring[*h] >= 15 { HV::Big(&self.ring[*h]) } else { HV::Small(self.ring[*h]) }
        }
    }
    #[derive(ComponentSubscription, UpdateHandler)]
    #[subscribe(HV::Big(x): HF)]
    pub struct HB;
    impl Component for HB {
        type Config = ();
        type Output = u32;
        fn new((): (), _: Self::Init<'_>) -> (Self, u32) { log("gGG new HB".into()); (HB, 0) }
        fn update(&mut self, i: Self::Input<'_>) -> u32 { log(format!("gGG HB upd {}", i.x)); *i.x as u32 }
    }
    #[derive(ComponentSubscription, UpdateHandler)]
    #[subscribe(HV::Small(n): HF)]
    pub struct HS;
    impl Component for HS {
        type Config = ();
        type Output = u32;
        fn new((): (), _: Self::Init<'_>) -> (Self, u32) { log("gGG new HS".into()); (HS, 0) }
        fn update(&mut self, i: Self::Input<'_>) -> u32 { log(format!("gGG HS upd {}", i.n)); i.n as u32 }
    }
    /// Always runs; handles two conditional by-value parents whose condition is on a handle source.
    #[derive(ComponentSubscription)]
    #[subscribe(p: Pk, b: HB, s: HS)]
    pub struct HR;
    impl Component for HR {
        type Config = ();
        type Output = u32;
        fn new((): (), _: Self::Init<'_>) -> (Self, u32) { (HR, 0) }
        fn update(&mut self, i: Self::Input<'_>) -> u32 { log(format!("gGG HR upd {} {}", i.b, i.s)); i.p.a }
    }
    impl UpdateHandler<Pk> for HR { const ACTIVE: bool = false; }
    impl UpdateHandler<HB> for HR { fn on_update(&mut self, _: Self::Input<'_>, v: &u32) { log(format!("gGG HR on HB {}", v)); } }
    impl UpdateHandler<HS> for HR { fn on_update(&mut self, _: Self::Input<'_>, v: &u32) { log(format!("gGG HR on HS {}", v)); } }
    /// Always runs; handles the handle parent itself (its view) and a conditional by-value child of it.
    #[derive(ComponentSubscription)]
    #[subscribe(p: Pk, h: HF, b: HB)]
    pub struct HH;
    impl Component for HH {
        type Config = ();
        type Output = u32;
        fn new((): (), _: Self::Init<'_>) -> (Self, u32) { (HH, 0) }
        fn update(&mut self, i: Self::Input<'_>) -> u32 {
            let h = match i.h { HV::Big(x) => *x as u32, HV::Small(n) => n as u32 };
            log(format!("gGG HH upd {} {}", h, i.b));
            h
        }
    }
    impl UpdateHandler<Pk> for HH { const ACTIVE: bool = false; }
    impl UpdateHandler<HF> for HH { fn on_update(&mut self, _: Self::Input<'_>, v: HV<'_>) { log(format!("gGG HH on HF {}", match v { HV::Big(x) => *x, HV::Small(n) => n })); } }
    impl UpdateHandler<HB> for HH { fn on_update(&mut self, _: Self::Input<'_>, v: &u32) { log(format!("gGG HH on HB {}", v)); } }
'''

out = [head]
mains = []
for g in range(a.graphs):
    rnd = random.Random(a.seed * 1000 + g)
    wide = g % 4 == 3
    n = 30 if wide else rnd.randint(7, 12)
    comps = []  # (name, subscribe-string, fields[(expr)], handled[(target, field)], is_b)
    for i in range(n):
        name = f"C{i}"
        last_wide = wide and i == n - 1
        if last_wide:
            tgt = list(range(i))[:25]
            fields = ["(i.p.a + i.p.b) as u32"] + [f"*i.e{k}" for k in range(len(tgt))]
            subs = ["p: Pk"] + [f"e{k}: C{t}" for k, t in enumerate(tgt)]
            handled = [(f"C{t}", f"e{k}") for k, t in enumerate(tgt)]
            comps.append((name, subs, fields, handled, True))
            continue
        kind = rnd.choice(["key", "key", "rg", "plain", "og", "og", "b"] if i > 1 else ["key", "rg", "key"])
        if kind == "b" and i < 2:
            kind = "key"
        if kind == "key":
            k = rnd.randint(0, 2)
            comps.append((name, [f"Ev::V{k}(x): Dm"], ["*i.x"], [], False))
        elif kind == "rg":
            gg = rnd.choice(["G1", "G2"])
            comps.append((name, [f"{gg}(x): Pk"], ["*i.x"], [], False))
        elif kind == "plain":
            t = rnd.randrange(i) if i else None
            if t is None:
                comps.append((name, ["Ev::V0(x): Dm"], ["*i.x"], [], False))
            else:
                comps.append((name, [f"e: C{t}"], ["*i.e"], [], False))
        elif kind == "og":
            t = rnd.randrange(i)
            s = rnd.choice(["Big", "Odd"])
            comps.append((name, [f"{s}(x): C{t}"], ["*i.x"], [], False))
        else:
            m = rnd.randint(1, min(3, i))
            tgt = rnd.sample(range(i), m)
            fields = ["(i.p.a + i.p.b) as u32"] + [f"*i.e{k}" for k in range(m)]
            subs = ["p: Pk"] + [f"e{k}: C{t}" for k, t in enumerate(tgt)]
            handled = [(f"C{t}", f"e{k}") for k, t in enumerate(tgt) if rnd.random() < 0.7]
            comps.append((name, subs, fields, handled, True))
    used = set()
    for (_, subs, _, _, _) in comps:
        for s in subs:
            if ": C" in s:
                used.add(int(s.split(": C")[1]))
    sinks = [f"C{i}" for i in range(n) if i not in used]
    # sinks must be at most 15: pick the last 15 and chain the rest into a final B
    extra = ""
    if len(sinks) > 13:
        keep = sinks[:12]
        rest = sinks[12:]
        rest = rest[:25]
        fields = ["(i.p.a + i.p.b) as u32"] + [f"*i.e{k}" for k in range(len(rest))]
        subs = ["p: Pk"] + [f"e{k}: {t}" for k, t in enumerate(rest)]
        comps.append((f"C{n}", subs, fields, [], True))
        sinks = keep + [f"C{n}"]
    mod = [f"pub mod g{g} {{\n    use super::*;"]
    if g % 4 == 2:
        mod.append(HANDLE.replace("GG", str(g)))
        sinks += ["HR", "HH"]
    for (name, subs, fields, handled, is_b) in comps:
        derive = "ComponentSubscription" if handled else "ComponentSubscription, UpdateHandler"
        mod.append(f"    #[derive({derive})]\n    #[subscribe({', '.join(subs)})]\n    pub struct {name};")
        mod.append(f"""    impl Component for {name} {{
        type Config = ();
        type Output = u32;
        fn new((): (), _: Self::Init<'_>) -> (Self, u32) {{ log("g{g} new {name}".into()); ({name}, 0) }}
        fn update(&mut self, i: Self::Input<'_>) -> u32 {{
            let s: u32 = 0u32{''.join(' + (' + f + ')' for f in fields)};
            let o = s.wrapping_mul(7).wrapping_add({name[1:]}) % 100;
            log(format!("g{g} {name} upd {{}} -> {{}}", s, o));
            o
        }}
    }}""")
        for (tgt, fld) in handled:
            mod.append(f"""    impl UpdateHandler<{tgt}> for {name} {{
        fn on_update(&mut self, i: Self::Input<'_>, v: &u32) {{
            log(format!("g{{}} {name} on {tgt} {{}} p={{}}", {g}, v, i.p.a));
        }}
    }}""")
        if handled:
            # every other entry needs an (inactive) impl
            done = {t for (t, _) in handled}
            for s in subs:
                if s.startswith("p: Pk"):
                    mod.append(f"    impl UpdateHandler<Pk> for {name} {{ const ACTIVE: bool = false; }}")
                elif ": C" in s and s.split(": ")[1] not in done:
                    mod.append(f"    impl UpdateHandler<{s.split(': ')[1]}> for {name} {{ const ACTIVE: bool = false; }}")
    mod.append("}")
    out.append("\n".join(mod))
    mains.append((g, sinks))

body = ["fn main() {", "    let path = std::env::args().nth(1).unwrap();"]
for g, sinks in mains:
    body.append(f"    {{\n        let mut p = Pipeline::<Pk, ({''.join(f'g{g}::' + s + ',' for s in sinks)})>::build(Configs::new()).unwrap();")
    body.append(f"        let mut r: u64 = {a.seed * 7919 + g};")
    body.append("        let mut nx = || { r = r.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407); (r >> 33) as u32 };")
    body.append(f"        for _ in 0..{a.packets} {{")
    body.append("            let (t, a, b) = ((nx() % 5) as u8, nx() % 30, nx() % 100);")
    body.append(f'            log(format!("g{g} push {{}} {{}} {{}}", t, a, b));')
    body.append("            p.push(Pk { t, a, b });")
    body.append("        }\n    }")
body.append("    std::fs::write(path, LOG.lock().unwrap().as_bytes()).unwrap();")
body.append("}")
out.append("\n".join(body))

os.makedirs(os.path.join(a.out, "src"), exist_ok=True)
with open(os.path.join(a.out, "Cargo.toml"), "w") as f:
    f.write(f"""[package]
name = "noflags"
version = "0.1.0"
edition = "2021"

[dependencies]
dataflow-flat = {{ path = "{a.lib}" }}

[workspace]
""")
with open(os.path.join(a.out, "src", "main.rs"), "w") as f:
    f.write("\n".join(out) + "\n")
