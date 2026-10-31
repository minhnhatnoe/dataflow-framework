//! The compile-time plan, computed by constant evaluation.
//!
//! The run needs one kind of fact: for every edge `parent -> dependency`,
//! whether this is where the depth-first walk from the root *first* reaches
//! the dependency ([`Edge::FIRST`]). Each component is stored (and later run)
//! at that edge only. The plan also answers "does the storage subtree of `top`
//! hold `id`?" ([`Plan::holds`]), which steers the input lookups.
//!
//! The plan is computed by running the depth-first walk itself inside constant
//! evaluation ([`visit`]): visiting a component records it, with the component
//! it was reached from, unless it was reached before; then visits its
//! dependencies in listing order. This needs `const_trait_impl`, because the
//! walk calls trait methods ([`Upstream::visit_all`]) on the types of the
//! graph from inside a `const fn`. The walk records components in first-reached
//! order, which is a pre-order of the first-parent tree, so the storage subtree
//! of a component is a contiguous range of the plan.
//!
//! A dependency cycle is found before the walk runs: the walk's trait bounds
//! (`C::Deps: const Upstream`, which mention the dependencies' bounds in turn)
//! are proved by the trait solver, and on a cyclic graph the proof needs itself.
//! That is reported as "overflow evaluating the requirement `(B,): Upstream`",
//! naming a component of the cycle. (The walk would also catch it, as a const
//! panic, but never gets to run.) The proof is as deep as the graph, so graphs
//! deeper than about 17 layers of 4 dependencies make the compiler's recursion
//! limit (128) a cost, and much deeper ones need `#![recursion_limit = "256"]`
//! in the user's crate. A component's own subscriptions add little depth: they
//! are a tree of 4-ary nodes, so 100 subscriptions are 4 levels.
//!
//! Every constant is a plain value with no references inside. (A graph of
//! nested `&'static` constants is hashed as a tree by incremental
//! compilation, which is exponential in graph depth.)

use std::any::type_name;
use std::marker::PhantomData;

use crate::component::Component;

/// Upper bound on components per pipeline. Compile-time memory grows with
/// MAX_COMPONENTS x (number of components), so raise it only as needed.
pub const MAX_COMPONENTS: usize = 512;
const SET_CAP: usize = MAX_COMPONENTS * 2; // power of two

/// Parent id of the root, and the id that means "no component". Component ids
/// are never 0 or 1.
pub(crate) const ROOT: u64 = 0;
const RESERVED: u64 = 1;

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
    if hash <= RESERVED {
        hash + 2 // keep ROOT and RESERVED free
    } else {
        hash
    }
}

/// Compile-time identity of a component.
#[doc(hidden)]
pub trait Described {
    const ID: u64;
}
impl<C: Component> Described for C {
    const ID: u64 = component_id(type_name::<C>());
}

// ---------------------------------------------------------------------------
// The walk
// ---------------------------------------------------------------------------

/// The depth-first walk over a component and everything upstream of it: record
/// `C` (reached from `parent`) and walk its dependencies, unless it was reached
/// before.
///
/// This is a function rather than a trait method on purpose: the only trait
/// bound it needs is `C::Deps: [const] Upstream`, so the trait solver's proof
/// that a graph can be walked has one level per layer of the graph, not two.
const fn visit<C: Component>(plan: &mut Plan, parent: u64)
where
    C::Deps: [const] Upstream,
{
    let id = <C as Described>::ID;
    let slot = plan.slot(id);
    if plan.keys[slot] == id {
        if plan.end[plan.index[slot] as usize - 1] == 0 {
            panic!("dependency cycle");
        }
        return;
    }
    let i = plan.len;
    if i == MAX_COMPONENTS {
        panic!("too many components: raise dataflow_flat::MAX_COMPONENTS");
    }
    plan.keys[slot] = id;
    plan.index[slot] = i as u32 + 1;
    plan.first_parent[i] = parent;
    plan.len = i + 1;
    <C::Deps as Upstream>::visit_all(plan, id);
    plan.end[i] = plan.len as u32;
}

/// The walk ([`visit`]) of every entry of a subscription tree (see
/// [`DepList`](crate::DepList)): one impl per node shape.
///
/// A node is handled in one impl: its (up to 4) entries, then its (up to 4)
/// children. The trait solver's proof for a component is as deep as its tree
/// (about log4 of its number of subscriptions), and the proof for the root as
/// deep as the graph times that.
#[doc(hidden)]
pub const trait Upstream {
    /// Visit each entry, in listing order, as reached from `parent`. Called on
    /// the top node of a component's tree: rejects a dependency listed twice,
    /// also when the two are in different nodes.
    fn visit_all(plan: &mut Plan, parent: u64);
    /// Visit the entries of this node and then its children (no duplicate check).
    fn visit_rec(plan: &mut Plan, parent: u64);
    /// Append the ids of this node's entries and then its children's to the
    /// plan's scratch list, in listing order.
    fn collect_ids(plan: &mut Plan);
}

const fn assert_distinct(ids: &[u64]) {
    let mut i = 0;
    while i < ids.len() {
        let mut j = i + 1;
        while j < ids.len() {
            if ids[i] == ids[j] {
                panic!("a component lists the same dependency twice");
            }
            j += 1;
        }
        i += 1;
    }
}

const impl Upstream for () {
    fn visit_all(_: &mut Plan, _: u64) {}
    fn visit_rec(_: &mut Plan, _: u64) {}
    fn collect_ids(_: &mut Plan) {}
}

macro_rules! upstream_leaf {
    ($($h:ident)+) => {
        const impl<$($h: Component),+> Upstream for ($($h,)+)
        where $($h::Deps: [const] Upstream,)+
        {
            fn visit_all(plan: &mut Plan, parent: u64) {
                assert_distinct(&[$(<$h as Described>::ID),+]);
                $(visit::<$h>(plan, parent);)+
            }
            fn visit_rec(plan: &mut Plan, parent: u64) {
                $(visit::<$h>(plan, parent);)+
            }
            fn collect_ids(plan: &mut Plan) {
                $(plan.push_id(<$h as Described>::ID);)+
            }
        }
    };
}
upstream_leaf!(A0);
upstream_leaf!(A0 A1);
upstream_leaf!(A0 A1 A2);
upstream_leaf!(A0 A1 A2 A3);

const impl<A0: Component, A1: Component, A2: Component, A3: Component,
           K0: [const] Upstream, K1: [const] Upstream, K2: [const] Upstream, K3: [const] Upstream>
    Upstream for (A0, A1, A2, A3, K0, K1, K2, K3)
where
    A0::Deps: [const] Upstream,
    A1::Deps: [const] Upstream,
    A2::Deps: [const] Upstream,
    A3::Deps: [const] Upstream,
{
    fn visit_all(plan: &mut Plan, parent: u64) {
        plan.scratch_len = 0;
        <Self as Upstream>::collect_ids(plan);
        assert_distinct(plan.scratch_ids());
        <Self as Upstream>::visit_rec(plan, parent);
    }
    fn visit_rec(plan: &mut Plan, parent: u64) {
        visit::<A0>(plan, parent);
        visit::<A1>(plan, parent);
        visit::<A2>(plan, parent);
        visit::<A3>(plan, parent);
        K0::visit_rec(plan, parent);
        K1::visit_rec(plan, parent);
        K2::visit_rec(plan, parent);
        K3::visit_rec(plan, parent);
    }
    fn collect_ids(plan: &mut Plan) {
        plan.push_id(<A0 as Described>::ID);
        plan.push_id(<A1 as Described>::ID);
        plan.push_id(<A2 as Described>::ID);
        plan.push_id(<A3 as Described>::ID);
        K0::collect_ids(plan);
        K1::collect_ids(plan);
        K2::collect_ids(plan);
        K3::collect_ids(plan);
    }
}

// ---------------------------------------------------------------------------
// The plan of a root
// ---------------------------------------------------------------------------

/// The plan of one root: each component's first-reached parent and subtree
/// extent, found by id through an open-addressing index.
#[doc(hidden)]
pub struct Plan {
    /// Number of components recorded so far.
    len: usize,
    /// id -> index + 1 (0 = empty), open addressing on `keys`.
    keys: [u64; SET_CAP],
    index: [u32; SET_CAP],
    first_parent: [u64; MAX_COMPONENTS],
    /// Components are in first-reached (depth-first pre-)order, so the
    /// storage subtree of entry `i` is the range `i..end[i]`. 0 while the
    /// walk is still inside entry `i`.
    end: [u32; MAX_COMPONENTS],
    /// The ids listed by the component whose duplicate check is running.
    scratch: [u64; MAX_COMPONENTS],
    scratch_len: usize,
}

impl Plan {
    /// The plan for root `A`: the depth-first walk from it.
    const fn of<A: Component>() -> Plan
    where
        A::Deps: [const] Upstream,
    {
        let mut plan = Plan {
            len: 0,
            keys: [0; SET_CAP],
            index: [0; SET_CAP],
            first_parent: [0; MAX_COMPONENTS],
            end: [0; MAX_COMPONENTS],
            scratch: [0; MAX_COMPONENTS],
            scratch_len: 0,
        };
        visit::<A>(&mut plan, ROOT);
        plan
    }

    const fn push_id(&mut self, id: u64) {
        if self.scratch_len == MAX_COMPONENTS {
            panic!("too many subscriptions in one component");
        }
        self.scratch[self.scratch_len] = id;
        self.scratch_len += 1;
    }

    const fn scratch_ids(&self) -> &[u64] {
        self.scratch.split_at(self.scratch_len).0
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

    /// The storage subtree of `id` as the range `lo..hi` of plan indices.
    pub(crate) const fn range_of(&self, id: u64) -> (usize, usize) {
        let i = self.index_of(id);
        (i, self.end[i] as usize)
    }

    /// Plan index of `id`.
    pub(crate) const fn position(&self, id: u64) -> usize {
        self.index_of(id)
    }

    /// Does the storage subtree of `top` hold `id`? Equivalently: is `top` on
    /// the first-parent chain of `id` (inclusive)?
    #[cfg(test)]
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
impl<A: Component> Planned for A
where
    A::Deps: const Upstream,
{
    const PLAN: &'static Plan = &Plan::of::<A>();
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
        macro_rules! n { ($($t:ty),*) => { $( m.insert(<$t as Described>::ID, type_name::<$t>().rsplit("::").next().unwrap()); )* } }
        n!(Source, Tokenize, WordCount, TopWords, Longest, Report, App);
        m.insert(<Len<Tokenize> as Described>::ID, "Len<Tokenize>");
        m
    }

    #[test]
    fn plan_trace() {
        let n = names();
        let plan = <App as Planned>::PLAN;
        // Entries in first-reached order, found by their index.
        let mut ids = vec![0u64; plan.len];
        for slot in 0..SET_CAP {
            if plan.keys[slot] != 0 {
                ids[plan.index[slot] as usize - 1] = plan.keys[slot];
            }
        }
        println!("plan (first-reached order, with the parent each was first reached from):");
        for (i, id) in ids.iter().enumerate() {
            println!(
                "  slot {i}  {:<14} reached from {:<10} subtree {}..{}",
                n[id],
                n[&plan.first_parent_of(*id)],
                i,
                plan.end[i]
            );
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
        assert_eq!(ids.len(), expected.len());
        for (i, (who, from)) in expected.iter().enumerate() {
            let id = ids[i];
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
