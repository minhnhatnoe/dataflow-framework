//! The compile-time plan, computed by constant evaluation.
//!
//! The run needs one kind of fact: for every edge `parent -> dependency`,
//! whether this is where the depth-first walk from the root *first* reaches
//! the dependency ([`Edge::FIRST`]). Each component is stored (and later run)
//! at that edge only. The plan also answers "does the storage subtree of `top`
//! hold `id`?" ([`Plan::holds`]), which steers the input lookups.
//!
//! How it is computed:
//!
//! * [`Described::SEQ`] of a component is the order in which a depth-first
//!   walk from it first reaches each component of its upstream graph, with the
//!   parent each was reached from. It is built in one pass ([`Seq::merge`])
//!   from the `SEQ`s of its dependencies: walking dependency B after A reaches
//!   exactly B's sequence minus what A already reached, in the same order,
//!   because what A reached is closed under "upstream of". The compiler caches
//!   each associated constant, so this is a bottom-up computation over the
//!   graph with one evaluation per concrete component type. (A `const fn`
//!   cannot call trait methods, so it cannot walk the types itself.)
//! * [`Planned::PLAN`] of the root is its `SEQ` indexed by id.
//!
//! Every constant is a plain value with no references inside, apart from the
//! one-level [`SeqList`] of already-computed constants. (A graph of nested
//! `&'static` constants is hashed as a tree by incremental compilation, which
//! is exponential in graph depth.) Sequences are structs of integer arrays
//! rather than arrays of structs: the compiler validates the result of a
//! constant field by field, which is slow for 512 entries.

use std::any::type_name;
use std::marker::PhantomData;

use crate::component::{Component, DepList};

/// Upper bound on components per pipeline. Compile-time memory grows with
/// MAX_COMPONENTS x (number of components), so raise it only as needed.
pub const MAX_COMPONENTS: usize = 512;
const SET_CAP: usize = MAX_COMPONENTS * 2; // power of two

/// Parent id meaning "whoever owns this dependency list"; replaced when a
/// component adopts the list.
const OWNER: u64 = 1;
/// Parent id of the root, and the id that means "no component".
pub(crate) const ROOT: u64 = 0;

/// FNV-1a of the type name: a compile-time identity for each component.
/// Collisions are caught when the pipeline is built (see `ConfigPass`).
const fn component_id(s: &str) -> u64 {
    let bytes = s.as_bytes();
    let mut hash: u64 = 0xcbf29ce484222325;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x100000001b3);
        i += 1;
    }
    if hash <= OWNER {
        hash + 2 // keep ROOT and OWNER reserved
    } else {
        hash
    }
}

/// Open-addressing set of ids (0 = empty slot; ids are never 0 or 1).
struct IdSet {
    slots: [u64; SET_CAP],
}

impl IdSet {
    const fn new() -> Self {
        IdSet { slots: [0; SET_CAP] }
    }
    const fn slot(&self, id: u64) -> usize {
        let mut i = (id as usize) & (SET_CAP - 1);
        while self.slots[i] != 0 && self.slots[i] != id {
            i = (i + 1) & (SET_CAP - 1);
        }
        i
    }
    const fn contains(&self, id: u64) -> bool {
        self.slots[self.slot(id)] == id
    }
    const fn insert(&mut self, id: u64) {
        let i = self.slot(id);
        self.slots[i] = id;
    }
}

// ---------------------------------------------------------------------------
// Walk orders
// ---------------------------------------------------------------------------

/// A walk order: component ids in first-reached order, each with the parent it
/// was reached from. Fixed capacity so it is a plain constant.
#[doc(hidden)]
pub struct Seq {
    len: usize,
    ids: [u64; MAX_COMPONENTS],
    parents: [u64; MAX_COMPONENTS],
}

/// The `SEQ`s of a dependency list's direct entries. One level of
/// references only: each `Seq` is a plain value.
#[doc(hidden)]
pub enum SeqList {
    Nil,
    Cons(&'static Seq, &'static SeqList),
}

impl Seq {
    /// The walk order of the component `owner` whose dependency list has the
    /// given `SEQ`s: the owner (parent `OWNER`, to be replaced by whoever
    /// adopts it), then each dependency's sequence minus what is already
    /// reached. A dependency's own entry is the one whose parent is `OWNER`;
    /// it is attached to `owner`.
    ///
    /// This is the hot loop of compile-time evaluation (one visit per entry
    /// of every dependency's sequence), so it is written to be few interpreter
    /// steps: one probe read per entry, no calls.
    const fn merge(owner: u64, mut deps: &'static SeqList) -> Seq {
        let mut out = Seq { len: 1, ids: [0; MAX_COMPONENTS], parents: [0; MAX_COMPONENTS] };
        out.ids[0] = owner;
        out.parents[0] = OWNER;
        let mut n = 1;
        let mut seen = [0u64; SET_CAP];
        let mut s = (owner as usize) & (SET_CAP - 1);
        seen[s] = owner;
        let mut direct = IdSet::new();
        while let SeqList::Cons(dep, rest) = deps {
            deps = rest;
            let head = dep.ids[0];
            if direct.contains(head) {
                panic!("a component lists the same dependency twice");
            }
            direct.insert(head);
            // What is reached so far is closed under "upstream of", so a
            // dependency that is already reached contributes nothing.
            s = (head as usize) & (SET_CAP - 1);
            while seen[s] != 0 && seen[s] != head {
                s = (s + 1) & (SET_CAP - 1);
            }
            if seen[s] != 0 {
                continue;
            }
            seen[s] = head;
            if n == MAX_COMPONENTS {
                panic!("too many components: raise dataflow_flat::MAX_COMPONENTS");
            }
            out.ids[n] = head;
            out.parents[n] = owner;
            n += 1;
            let (ids, parents, len) = (&dep.ids, &dep.parents, dep.len);
            let mut i = 1;
            while i < len {
                let id = ids[i];
                s = (id as usize) & (SET_CAP - 1);
                let mut v = seen[s];
                while v != 0 && v != id {
                    s = (s + 1) & (SET_CAP - 1);
                    v = seen[s];
                }
                if v == 0 {
                    seen[s] = id;
                    if n == MAX_COMPONENTS {
                        panic!("too many components: raise dataflow_flat::MAX_COMPONENTS");
                    }
                    out.ids[n] = id;
                    out.parents[n] = parents[i];
                    n += 1;
                }
                i += 1;
            }
        }
        out.len = n;
        out
    }
}

/// Compile-time identity and upstream walk of a component.
#[doc(hidden)]
pub trait Described {
    const ID: u64;
    /// The walk order of everything upstream of this component, itself first.
    const SEQ: Seq;
}

impl<C: Component> Described for C {
    const ID: u64 = component_id(type_name::<C>());
    const SEQ: Seq = Seq::merge(<C as Described>::ID, <C::Deps as DepList>::HEADS);
}

// ---------------------------------------------------------------------------
// The plan of a root
// ---------------------------------------------------------------------------

/// The plan of one root: each component's first-reached parent and subtree
/// extent, found by id through an open-addressing index.
#[doc(hidden)]
pub struct Plan {
    /// id -> index + 1 (0 = empty), open addressing on `keys`.
    keys: [u64; SET_CAP],
    index: [u32; SET_CAP],
    first_parent: [u64; MAX_COMPONENTS],
    /// Components are in first-reached (depth-first pre-)order, so the
    /// storage subtree of entry `i` is the range `i..end[i]`.
    end: [u32; MAX_COMPONENTS],
}

impl Plan {
    /// The plan for the root whose walk order is `seq`.
    const fn from_seq(seq: &Seq) -> Plan {
        let mut plan = Plan {
            keys: [0; SET_CAP],
            index: [0; SET_CAP],
            first_parent: [0; MAX_COMPONENTS],
            end: [0; MAX_COMPONENTS],
        };
        let mut i = 0;
        while i < seq.len {
            let slot = plan.slot(seq.ids[i]);
            plan.keys[slot] = seq.ids[i];
            plan.index[slot] = i as u32 + 1;
            plan.first_parent[i] = if i == 0 { ROOT } else { seq.parents[i] };
            plan.end[i] = i as u32 + 1;
            i += 1;
        }
        // Children come after their parent: grow each parent's range, last first.
        let mut i = seq.len;
        while i > 1 {
            i -= 1;
            let p = plan.index_of(plan.first_parent[i]);
            if plan.end[i] > plan.end[p] {
                plan.end[p] = plan.end[i];
            }
        }
        plan
    }

    const fn slot(&self, id: u64) -> usize {
        let mut i = (id as usize) & (SET_CAP - 1);
        while self.keys[i] != 0 && self.keys[i] != id {
            i = (i + 1) & (SET_CAP - 1);
        }
        i
    }

    const fn index_of(&self, id: u64) -> usize {
        match self.index[self.slot(id)] {
            0 => panic!("component not in plan"),
            n => n as usize - 1,
        }
    }

    const fn first_parent_of(&self, id: u64) -> u64 {
        self.first_parent[self.index_of(id)]
    }

    /// Does the storage subtree of `top` hold `id`? Equivalently: is `top` on
    /// the first-parent chain of `id` (inclusive)?
    pub(crate) const fn holds(&self, top: u64, id: u64) -> bool {
        let t = self.index_of(top);
        let i = self.index_of(id);
        t <= i && i < self.end[t] as usize
    }
}

/// A component that can be the root of a plan, evaluated once per root type.
#[doc(hidden)]
pub trait Planned: Component {
    const PLAN: &'static Plan;
}
impl<A: Component> Planned for A {
    const PLAN: &'static Plan = &Plan::from_seq(&<A as Described>::SEQ);
}

/// Per-edge constant, looked up in the plan of root `R`.
#[doc(hidden)]
pub struct Edge<R, P, H>(PhantomData<(R, P, H)>);
impl<R: Planned, P: Component, H: Component> Edge<R, P, H> {
    /// True if the edge `P -> H` is where the walk first reaches `H`.
    pub const FIRST: bool = R::PLAN.first_parent_of(<H as Described>::ID) == <P as Described>::ID;
}

#[cfg(test)]
mod trace {
    use super::*;
    use crate::ComponentSubscription;
    use std::collections::HashMap;

    macro_rules! comp {
        ($name:ident, [$($n:ident: $d:ty),*], $out:ty) => {
            #[derive(ComponentSubscription)]
            #[subscribe($($n: $d),*)]
            pub struct $name;
            impl Component for $name {
                type Config = ();
                type Output = $out;
                fn new((): ()) -> Self { $name }
                fn update(&mut self, _: Self::Input<'_>) -> $out { Default::default() }
            }
        };
    }
    comp!(Source, [], String);
    comp!(Tokenize, [line: Source], Vec<String>);
    comp!(WordCount, [tokens: Tokenize], u64);
    comp!(TopWords, [counts: WordCount], Vec<u64>);
    comp!(Longest, [tokens: Tokenize], Option<String>);
    comp!(Report, [top: TopWords, longest: Longest, line: Source], String);

    #[derive(ComponentSubscription)]
    #[subscribe(items: U)]
    pub struct Len<U: Component<Output = Vec<String>>>(PhantomData<U>);
    impl<U: Component<Output = Vec<String>>> Component for Len<U> {
        type Config = ();
        type Output = usize;
        fn new((): ()) -> Self {
            Len(PhantomData)
        }
        fn update(&mut self, input: Self::Input<'_>) -> usize {
            input.items.len()
        }
    }

    comp!(App, [report: Report, count: Len<Tokenize>], ());

    fn names() -> HashMap<u64, &'static str> {
        let mut m = HashMap::new();
        m.insert(ROOT, "ROOT");
        m.insert(OWNER, "OWNER");
        macro_rules! n { ($($t:ty),*) => { $( m.insert(<$t as Described>::ID, type_name::<$t>().rsplit("::").next().unwrap()); )* } }
        n!(Source, Tokenize, WordCount, TopWords, Longest, Report, App);
        m.insert(<Len<Tokenize> as Described>::ID, "Len<Tokenize>");
        m
    }

    fn show(label: &str, s: &Seq, n: &HashMap<u64, &str>) {
        let items: Vec<String> = (0..s.len).map(|i| format!("{}←{}", n[&s.ids[i]], n[&s.parents[i]])).collect();
        println!("{label:<16} [{}]", items.join(", "));
    }

    #[test]
    fn plan_trace() {
        let n = names();
        println!("component ids: Source = {:#018x}", <Source as Described>::ID);
        show("SEQ(Source)", &<Source as Described>::SEQ, &n);
        show("SEQ(Tokenize)", &<Tokenize as Described>::SEQ, &n);
        show("SEQ(WordCount)", &<WordCount as Described>::SEQ, &n);
        show("SEQ(TopWords)", &<TopWords as Described>::SEQ, &n);
        show("SEQ(Longest)", &<Longest as Described>::SEQ, &n);
        show("SEQ(Report)", &<Report as Described>::SEQ, &n);
        show("SEQ(Len<Tok>)", &<Len<Tokenize> as Described>::SEQ, &n);
        show("SEQ(App)", &<App as Described>::SEQ, &n);

        let plan = <App as Planned>::PLAN;
        let roots = &<App as Described>::SEQ;
        println!("\nplan (first-reached order, with the parent each was first reached from):");
        for i in 0..roots.len {
            let id = roots.ids[i];
            println!("  slot {i}  {:<14} reached from {}", n[&id], n[&plan.first_parent_of(id)]);
        }

        let expected = [
            ("App", "ROOT"),
            ("Report", "App"),
            ("TopWords", "Report"),
            ("WordCount", "TopWords"),
            ("Tokenize", "WordCount"),
            ("Source", "Tokenize"),
            ("Longest", "Report"),
            ("Len<Tokenize>", "App"),
        ];
        assert_eq!(roots.len, expected.len());
        for (i, (who, from)) in expected.iter().enumerate() {
            let id = roots.ids[i];
            assert_eq!((n[&id], n[&plan.first_parent_of(id)]), (*who, *from), "slot {i}");
        }

        println!("\nedges:");
        macro_rules! edge { ($p:ty, $h:ty, $pn:expr, $hn:expr, $want:expr) => {
            let got = Edge::<App, $p, $h>::FIRST;
            println!("  {:<10} -> {:<14} FIRST = {}", $pn, $hn, got);
            assert_eq!(got, $want, "{} -> {}", $pn, $hn);
        } }
        edge!(App, Report, "App", "Report", true);
        edge!(App, Len<Tokenize>, "App", "Len<Tokenize>", true);
        edge!(Report, TopWords, "Report", "TopWords", true);
        edge!(Report, Longest, "Report", "Longest", true);
        edge!(Report, Source, "Report", "Source", false);
        edge!(TopWords, WordCount, "TopWords", "WordCount", true);
        edge!(WordCount, Tokenize, "WordCount", "Tokenize", true);
        edge!(Longest, Tokenize, "Longest", "Tokenize", false);
        edge!(Tokenize, Source, "Tokenize", "Source", true);
        edge!(Len<Tokenize>, Tokenize, "Len<Tok>", "Tokenize", false);

        // `holds`: the storage subtree of Report holds TopWords..Source and
        // Longest, but not Len<Tokenize> or App.
        let id = |i: u64| i;
        assert!(plan.holds(id(<Report as Described>::ID), id(<Source as Described>::ID)));
        assert!(plan.holds(id(<Report as Described>::ID), id(<Longest as Described>::ID)));
        assert!(!plan.holds(id(<Report as Described>::ID), id(<Len<Tokenize> as Described>::ID)));
        assert!(!plan.holds(id(<Report as Described>::ID), id(<App as Described>::ID)));
        assert!(plan.holds(id(<Tokenize as Described>::ID), id(<Source as Described>::ID)));
    }
}
