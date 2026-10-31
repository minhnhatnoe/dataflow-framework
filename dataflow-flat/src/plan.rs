//! The compile-time plan, computed by constant evaluation.
//!
//! The run needs one kind of fact: for every edge `parent -> dependency`,
//! whether this is where the depth-first walk from the root *first* reaches
//! the dependency ([`Edge::FIRST`]). An edge is an entry of the parent's
//! subscription tree, `Sub<dependency, position>`, so two entries of one parent
//! that name the same type (through an alias) are two edges, and exactly one of
//! them is the first. Each component is stored (and later run)
//! at that edge only. The plan also gives the range of plan indices each
//! storage subtree covers ([`Plan::range_of`]), which steers the input lookups.
//!
//! The plan is computed by running the depth-first walk itself inside constant
//! evaluation ([`visit`]): visiting a component records it, with the component
//! it was reached from, unless it was reached before; then visits its
//! dependencies in listing order. This needs `const_trait_impl`, because the
//! walk calls trait methods ([`Upstream::visit_all`]) on the types of the
//! graph from inside a `const fn`. There is no check for a component listing a
//! dependency twice: the second entry is just a later visit of a component
//! already recorded. The walk records components in first-reached
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
//! are a tree of 8-ary nodes, so 100 subscriptions are 3 levels.
//!
//! Every constant is a plain value with no references inside. (A graph of
//! nested `&'static` constants is hashed as a tree by incremental
//! compilation, which is exponential in graph depth.)

use std::any::TypeId;
use std::marker::PhantomData;

use crate::component::{Component, Sub};
use crate::identity::{At, Hashed, BLANK};
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
/// `C` (reached from `parent`, at position `pos` of its list) and walk its
/// dependencies, unless it was reached before.
///
/// This is a function rather than a trait method on purpose: the only trait
/// bound it needs is `C::Deps: [const] Upstream`, so the trait solver's proof
/// that a graph can be walked has one level per layer of the graph, not two.
const fn visit<C: Component>(plan: &mut Plan, parent: u32, pos: u32)
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
    plan.first_pos[i] = pos;
    plan.len = i + 1;
    <C::Deps as Upstream>::visit_all(plan, i as u32);
    plan.end[i] = plan.len as u32;
}

/// The walk ([`visit`]) of every entry of a subscription tree (see
/// [`DepList`](crate::DepList)): one impl per node shape.
///
/// A leaf visits its (up to 7) entries; an inner node visits its 8 children.
/// The trait solver's proof for a component is as deep as its tree (about log8
/// of its number of subscriptions), and the proof for the root as deep as the
/// graph times that.
#[doc(hidden)]
pub const trait Upstream {
    /// Visit each entry, in listing order, as reached from `parent`; then the
    /// children. An entry `Sub<E, I>` visits `E` with position `I`.
    fn visit_all(plan: &mut Plan, parent: u32);
}

macro_rules! upstream_leaf {
    ([$($h:ident $i:ident)*]) => {
        #[allow(unused_variables)]
        const impl<$($h: Component, const $i: usize),*> Upstream for ($(Sub<$h, $i>,)*)
        where $($h::Deps: [const] Upstream,)*
        {
            fn visit_all(plan: &mut Plan, parent: u32) {
                $(visit::<$h>(plan, parent, $i as u32);)*
            }
        }
    };
}
dataflow_macros::leaf_shapes!(upstream_leaf);

macro_rules! upstream_inner {
    ([$($K:ident)*]) => {
        const impl<$($K: [const] Upstream,)*> Upstream for ($($K,)*) {
            fn visit_all(plan: &mut Plan, parent: u32) {
                $($K::visit_all(plan, parent);)*
            }
        }
    };
}
dataflow_macros::inner_shape!(upstream_inner);

// ---------------------------------------------------------------------------
// The plan of a root
// ---------------------------------------------------------------------------

/// The plan of one root: each component's `TypeId` in first-reached order
/// (the order is the component's index), its first-reached parent as an
/// index with the position of the entry that reached it, and its subtree
/// extent.
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
    /// Position, in that parent's list of subscriptions, of the entry that
    /// first reached each component (two entries of one parent may name the
    /// same component, through an alias).
    first_pos: [u32; MAX_COMPONENTS],
    /// Components are in first-reached (depth-first pre-)order, so the
    /// storage subtree of entry `i` is the range `i..end[i]`. 0 while the
    /// walk is still inside entry `i`.
    end: [u32; MAX_COMPONENTS],
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
            first_pos: [0; MAX_COMPONENTS],
            end: [0; MAX_COMPONENTS],
        };
        visit::<A>(&mut plan, NO_PARENT, 0);
        plan
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

    /// Position, in the first parent's list, of the entry that first reached
    /// entry `i`.
    pub(crate) const fn first_pos_of(&self, i: usize) -> u32 {
        self.first_pos[i]
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

/// Per-edge constant, looked up in the plan of root `R`: the entry at position
/// `I` of `P`'s list, naming `H`.
#[doc(hidden)]
pub struct Edge<R, P, H, const I: usize>(PhantomData<(R, P, H)>);
impl<R: Planned, P: Component, H: Component, const I: usize> Edge<R, P, H, I> {
    /// True if this entry is where the walk first reaches `H`: `P` is the
    /// first parent and this is the position that reached it. (Two entries of
    /// `P` naming `H` through an alias have the same `P` and `H`, and differ in
    /// `I`.)
    pub const FIRST: bool = {
        let h = At::<R, H>::INDEX;
        R::PLAN.first_parent_of(h) as usize == At::<R, P>::INDEX && R::PLAN.first_pos_of(h) as usize == I
    };
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
        macro_rules! edge { ($p:ty, $h:ty, $i:expr, $pn:expr, $hn:expr, $want:expr) => {
            let got = Edge::<App, $p, $h, $i>::FIRST;
            println!("  {:<10} -> {:<14} FIRST = {}", $pn, $hn, got);
            assert_eq!(got, $want, "{} -> {}", $pn, $hn);
        } }
        edge!(App, Report, 0, "App", "Report", true);
        edge!(App, Len<Tokenize>, 1, "App", "Len<Tokenize>", true);
        edge!(Report, TopWords, 0, "Report", "TopWords", true);
        edge!(Report, Longest, 1, "Report", "Longest", true);
        edge!(Report, Source, 2, "Report", "Source", false);
        edge!(TopWords, WordCount, 0, "TopWords", "WordCount", true);
        edge!(WordCount, Tokenize, 0, "WordCount", "Tokenize", true);
        edge!(Longest, Tokenize, 0, "Longest", "Tokenize", false);
        edge!(Tokenize, Source, 0, "Tokenize", "Source", true);
        edge!(Len<Tokenize>, Tokenize, 0, "Len<Tok>", "Tokenize", false);
        // The right parent at another position is not the first edge.
        edge!(App, Report, 1, "App", "Report@1", false);

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
