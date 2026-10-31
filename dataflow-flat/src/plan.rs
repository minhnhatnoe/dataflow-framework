//! The compile-time plan, computed by constant evaluation.
//!
//! The run needs one kind of fact: for every edge `parent -> dependency`,
//! whether this is where the depth-first walk from the root *first* reaches
//! the dependency ([`Edge::FIRST`]). Each component is stored (and later run)
//! at that edge only. The plan also gives the range of plan indices each
//! storage subtree covers ([`Plan::range_of`]), which steers the input lookups.
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

use std::any::TypeId;
use std::marker::PhantomData;

use crate::component::Component;
use crate::identity::{assert_distinct, At, Hashed, BLANK};
use crate::table::Table;

/// Upper bound on components per pipeline. Compile-time memory grows with
/// MAX_COMPONENTS x (number of components), so raise it only as needed.
pub const MAX_COMPONENTS: usize = 512;

/// The first parent of the root: it was not reached from any component.
const NO_PARENT: u32 = u32::MAX;

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
const fn visit<C: Component>(plan: &mut Plan, parent: u32)
where
    C::Deps: [const] Upstream,
{
    let id = TypeId::of::<C>();
    let slot = plan.table.slot(&plan.ids, <C as Hashed>::HASH, id);
    if let Some(at) = plan.table.at(slot) {
        if plan.end[at] == 0 {
            panic!("dependency cycle");
        }
        return;
    }
    let i = plan.len;
    if i == MAX_COMPONENTS {
        panic!("too many components: raise dataflow_flat::MAX_COMPONENTS");
    }
    plan.table.put(slot, i);
    plan.ids[i] = id;
    plan.first_parent[i] = parent;
    plan.len = i + 1;
    <C::Deps as Upstream>::visit_all(plan, i as u32);
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
    fn visit_all(plan: &mut Plan, parent: u32);
    /// Visit the entries of this node and then its children (no duplicate check).
    fn visit_rec(plan: &mut Plan, parent: u32);
    /// Append the `TypeId`s of this node's entries and then its children's to the
    /// plan's scratch list, in listing order.
    fn collect_ids(plan: &mut Plan);
}

macro_rules! upstream_leaf {
    ($($h:ident)*) => {
        #[allow(unused_variables)]
        const impl<$($h: Component),*> Upstream for ($($h,)*)
        where $($h::Deps: [const] Upstream,)*
        {
            fn visit_all(plan: &mut Plan, parent: u32) {
                assert_distinct(&[$(TypeId::of::<$h>()),*] as &[TypeId]);
                $(visit::<$h>(plan, parent);)*
            }
            fn visit_rec(plan: &mut Plan, parent: u32) {
                $(visit::<$h>(plan, parent);)*
            }
            fn collect_ids(plan: &mut Plan) {
                $(plan.push_id(TypeId::of::<$h>());)*
            }
        }
    };
}
upstream_leaf!();
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
    fn visit_all(plan: &mut Plan, parent: u32) {
        plan.scratch_len = 0;
        <Self as Upstream>::collect_ids(plan);
        assert_distinct(plan.scratch_ids());
        <Self as Upstream>::visit_rec(plan, parent);
    }
    fn visit_rec(plan: &mut Plan, parent: u32) {
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
        plan.push_id(TypeId::of::<A0>());
        plan.push_id(TypeId::of::<A1>());
        plan.push_id(TypeId::of::<A2>());
        plan.push_id(TypeId::of::<A3>());
        K0::collect_ids(plan);
        K1::collect_ids(plan);
        K2::collect_ids(plan);
        K3::collect_ids(plan);
    }
}

// ---------------------------------------------------------------------------
// The plan of a root
// ---------------------------------------------------------------------------

/// The plan of one root: each component's `TypeId` in first-reached order
/// (the order is the component's index), its first-reached parent as an
/// index, and its subtree extent.
#[doc(hidden)]
pub struct Plan {
    /// Number of components recorded so far.
    len: usize,
    /// The components, by index.
    ids: [TypeId; MAX_COMPONENTS],
    /// Position by component: hash of the type name to start, `TypeId` to
    /// decide (see [`table`](crate::table)).
    table: Table,
    /// Index of the component each was first reached from (`NO_PARENT` for
    /// the root).
    first_parent: [u32; MAX_COMPONENTS],
    /// Components are in first-reached (depth-first pre-)order, so the
    /// storage subtree of entry `i` is the range `i..end[i]`. 0 while the
    /// walk is still inside entry `i`.
    end: [u32; MAX_COMPONENTS],
    /// The `TypeId`s listed by the component whose duplicate check is running.
    scratch: [TypeId; MAX_COMPONENTS],
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
            ids: [BLANK; MAX_COMPONENTS],
            table: Table::new(),
            first_parent: [NO_PARENT; MAX_COMPONENTS],
            end: [0; MAX_COMPONENTS],
            scratch: [BLANK; MAX_COMPONENTS],
            scratch_len: 0,
        };
        visit::<A>(&mut plan, NO_PARENT);
        plan
    }

    const fn push_id(&mut self, id: TypeId) {
        if self.scratch_len == MAX_COMPONENTS {
            panic!("too many subscriptions in one component");
        }
        self.scratch[self.scratch_len] = id;
        self.scratch_len += 1;
    }

    const fn scratch_ids(&self) -> &[TypeId] {
        self.scratch.split_at(self.scratch_len).0
    }

    /// Is the component with this `TypeId` in the plan? (Run time, at build.)
    pub(crate) fn contains(&self, id: TypeId) -> bool {
        self.ids[..self.len].contains(&id)
    }

    /// Plan index of the component with this name hash and `TypeId`.
    pub(crate) const fn index_of(&self, hash: u64, id: TypeId) -> usize {
        match self.table.at(self.table.slot(&self.ids, hash, id)) {
            Some(i) => i,
            None => panic!("component not in plan"),
        }
    }

    /// Index of the component that entry `i` was first reached from.
    pub(crate) const fn first_parent_of(&self, i: usize) -> u32 {
        self.first_parent[i]
    }

    /// The storage subtree of entry `i` as the range `lo..hi` of plan indices.
    pub(crate) const fn range_of(&self, i: usize) -> (usize, usize) {
        (i, self.end[i] as usize)
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
    pub const FIRST: bool = R::PLAN.first_parent_of(At::<R, H>::INDEX) as usize == At::<R, P>::INDEX;
}

#[cfg(test)]
mod trace {
    use super::*;
    use std::any::type_name;
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

    fn names() -> HashMap<TypeId, &'static str> {
        let mut m = HashMap::new();
        macro_rules! n { ($($t:ty),*) => { $( m.insert(TypeId::of::<$t>(), type_name::<$t>().rsplit("::").next().unwrap()); )* } }
        n!(Source, Tokenize, WordCount, TopWords, Longest, Report, App);
        m.insert(TypeId::of::<Len<Tokenize>>(), "Len<Tokenize>");
        m
    }

    fn parent_name(plan: &Plan, n: &HashMap<TypeId, &'static str>, i: usize) -> &'static str {
        match plan.first_parent_of(i) {
            NO_PARENT => "ROOT",
            p => n[&plan.ids[p as usize]],
        }
    }

    #[test]
    fn plan_trace() {
        let n = names();
        let plan = <App as Planned>::PLAN;
        // Entries in first-reached order.
        println!("plan (first-reached order, with the parent each was first reached from):");
        for i in 0..plan.len {
            println!(
                "  slot {i}  {:<14} reached from {:<10} subtree {}..{}",
                n[&plan.ids[i]],
                parent_name(plan, &n, i),
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
        assert_eq!(plan.len, expected.len());
        for (i, (who, from)) in expected.iter().enumerate() {
            assert_eq!((n[&plan.ids[i]], parent_name(plan, &n, i)), (*who, *from), "slot {i}");
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

        // The storage subtree of Report holds TopWords..Source and Longest, but
        // not Len<Tokenize> or App.
        let at = |id: TypeId| plan.ids[..plan.len].iter().position(|x| *x == id).unwrap();
        let (report, source) = (at(TypeId::of::<Report>()), at(TypeId::of::<Source>()));
        let holds = |top: usize, i: usize| {
            let (lo, hi) = plan.range_of(top);
            lo <= i && i < hi
        };
        assert!(holds(report, source));
        assert!(holds(report, at(TypeId::of::<Longest>())));
        assert!(!holds(report, at(TypeId::of::<Len<Tokenize>>())));
        assert!(!holds(report, at(TypeId::of::<App>())));
        assert!(holds(at(TypeId::of::<Tokenize>()), source));
        // A type that is not in the plan is not found.
        assert!(!plan.contains(TypeId::of::<u8>()));
        assert!(plan.contains(TypeId::of::<Longest>()));
    }
}

const _: () = assert!(
    crate::ARITY == 4,
    "the node-shape impls in this file are written for ARITY = 4; update them"
);
