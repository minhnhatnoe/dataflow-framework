//! The compile-time plan, computed by constant evaluation.
//!
//! The plan is the depth-first walk from the pipeline's *anchor* (the node that
//! stands for the sinks), run inside constant evaluation ([`visit`]): visiting
//! a node records it, with the node it was reached from, unless it was reached
//! before; then visits its subscriptions in listing order. This needs
//! `const_trait_impl`, because the walk calls trait methods
//! ([`Upstream::visit_all`]) on the types of the graph from inside a `const fn`.
//!
//! What the plan records, all as integer arrays (no references, so that
//! incremental compilation stays linear):
//!
//! * for every node, its `TypeId` (identity), its **kind** (root, component,
//!   anchor; demuxers in phase 1b), the node it was first reached from and the
//!   position of the entry that reached it, the extent of its subtree, its depth
//!   in the first-parent tree;
//! * the **execution position** of every node: its place in the post-order of
//!   the walk. The root is always position 0 (it is the only node without
//!   subscriptions, and a component without subscriptions is an error), the
//!   anchor is last. The components are positions `1..len-1`, and *component
//!   position* `p` below is that minus one;
//! * the **upstream lists**: every subscription entry as an edge
//!   `(subscriber, node)`, in the order the walk visits the entries, which is
//!   listing order per subscriber ([`Plan::upstream`]). Phase 1b computes
//!   conditions from these.
//!
//! The checks that need only the walk are made in it and reported by
//! [`Plan::check`] (a const panic with the component's name): a graph with a second root or a root that is not the
//! pipeline's `R`, and a component that is not reachable from the root.
//!
//! A dependency cycle is found before the walk runs: the walk's trait bounds
//! (`N::Subs: const Upstream`, which mention the dependencies' bounds in turn)
//! are proved by the trait solver, and on a cyclic graph the proof needs itself.
//! That is reported as "overflow evaluating the requirement `(B,): Upstream`",
//! naming a component of the cycle. The proof is as deep as the graph, so
//! graphs deeper than about 40 layers need `#![recursion_limit = "256"]` in the
//! user's crate (README, "Graph depth").

use std::any::{type_name, TypeId};

use crate::component::Sub;
use crate::identity::{Hashed, BLANK};
use crate::node::{Node, KIND_ANCHOR, KIND_ROOT};
use crate::table::Table;

/// Upper bound on nodes (the root, the components and the anchor) per
/// pipeline. Compile-time memory grows with MAX_COMPONENTS x (number of
/// nodes), so raise it only as needed.
pub const MAX_COMPONENTS: usize = 512;

/// Upper bound on subscription entries per pipeline, summed over all
/// components.
pub const MAX_EDGES: usize = 4096;

/// The first parent of the anchor: it was not reached from any node.
const NO_PARENT: u32 = u32::MAX;

// ---------------------------------------------------------------------------
// Const panics with a name
// ---------------------------------------------------------------------------

/// Longest error message kept in the plan.
const MSG: usize = 256;

/// Assemble `name` followed by `tail` into a byte buffer (`panic!` can't format
/// in a `const fn`). Returns the buffer and the length used.
const fn message(name: &str, tail: &str) -> ([u8; MSG], usize) {
    let mut buf = [0u8; MSG];
    let mut n = 0;
    let parts = [name.as_bytes(), tail.as_bytes()];
    let mut p = 0;
    while p < 2 {
        let part = parts[p];
        let mut i = 0;
        while i < part.len() && n < buf.len() {
            buf[n] = part[i];
            n += 1;
            i += 1;
        }
        p += 1;
    }
    (buf, n)
}

// ---------------------------------------------------------------------------
// The walk
// ---------------------------------------------------------------------------

/// Record `N` (reached from `parent`, at position `pos` of its list) unless it
/// was reached before; either way record the edge `parent -> N`. Returns the
/// plan index of `N` and whether it is new, i.e. whether its subscriptions
/// still have to be walked.
///
/// This is split from the walk of the subscriptions (and from [`leave`]) so
/// that the generated [`Upstream`] impls can call them inline around the
/// recursion into `N::Subs`, which keeps constant evaluation at one stack frame
/// per graph layer. The trait solver's proof is unaffected: the only bound is
/// `N::Subs: [const] Upstream`, a trait on the tree rather than on the node.
const fn enter<N: Node>(plan: &mut Plan, parent: u32, pos: u32) -> (u32, bool) {
    let id = TypeId::of::<N>();
    let slot = plan.table.slot(&plan.ids, <N as Hashed>::HASH, id);
    let (i, new) = if let Some(at) = plan.table.at(slot) {
        if plan.end[at] == 0 {
            panic!("dependency cycle");
        }
        (at, false)
    } else {
        if N::KIND == KIND_ROOT && !(id == plan.root_id) {
            plan.fail(
                type_name::<N>(),
                " is a root, but the pipeline's root is a different type: a pipeline has exactly one root",
            );
        }
        let i = plan.len;
        if i == MAX_COMPONENTS {
            // Not recorded, not walked: the plan is not meaningful from here on.
            plan.fail("too many components: ", "raise dataflow_flat::MAX_COMPONENTS");
            return (0, false);
        }
        plan.table.put(slot, i);
        plan.ids[i] = id;
        plan.kind[i] = N::KIND;
        plan.first_parent[i] = parent;
        plan.first_pos[i] = pos;
        plan.len = i + 1;
        plan.depth[i] = if parent == NO_PARENT { 0 } else { plan.depth[parent as usize] + 1 };
        (i, true)
    };
    if parent != NO_PARENT {
        let e = plan.n_edges;
        if e == MAX_EDGES {
            plan.fail("too many subscriptions: ", "raise dataflow_flat::MAX_EDGES");
        } else {
            plan.edge_from[e] = parent;
            plan.edge_to[e] = i as u32;
            plan.n_edges = e + 1;
        }
        // A node reached again is finished: if the root is upstream of it, it
        // is upstream of `parent` too. (A new node reports when it is left.)
        if !new && plan.reach[i] {
            plan.has_up[parent as usize] = true;
        }
    }
    (i as u32, new)
}

/// A node's walk is over: record its subtree end and execution position, and
/// check that it is reachable from the root (some node it subscribes to is).
const fn leave<N: Node>(plan: &mut Plan, i: u32) {
    let i = i as usize;
    plan.end[i] = plan.len as u32;
    plan.post_of[i] = plan.done as u32;
    plan.by_post[plan.done] = i as u32;
    plan.done += 1;
    let reach = N::KIND == KIND_ROOT || N::KIND == KIND_ANCHOR || plan.has_up[i];
    if !reach {
        plan.fail(type_name::<N>(), " is not reachable from the root");
    }
    plan.reach[i] = true;
    if N::KIND == KIND_ROOT {
        plan.root_idx = i as u32;
    }
    // The node that entered this one now has a reachable subscription.
    let fp = plan.first_parent[i];
    if fp != NO_PARENT {
        plan.has_up[fp as usize] = true;
    }
}

/// `enter`, the walk of the subscriptions, `leave`: used for the anchor only
/// (the `Upstream` impls spell this out inline for their entries).
const fn visit<N: Node>(plan: &mut Plan, parent: u32, pos: u32)
where
    N::Subs: [const] Upstream,
{
    let (i, new) = enter::<N>(plan, parent, pos);
    if new {
        <N::Subs as Upstream>::visit_all(plan, i);
        leave::<N>(plan, i);
    }
}

/// The walk ([`visit`]) of every entry of a subscription tree (see
/// [`DepList`](crate::DepList)): one impl per node shape.
///
/// A leaf visits its (up to 7) entries; an inner node visits its 8 children.
/// The trait solver's proof for a node is as deep as its tree (about log8 of
/// its number of subscriptions), and the proof for the anchor as deep as the
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
        const impl<$($h: Node, const $i: usize),*> Upstream for ($(Sub<$h, $i>,)*)
        where $($h::Subs: [const] Upstream,)*
        {
            fn visit_all(plan: &mut Plan, parent: u32) {
                $({
                    let (i, new) = enter::<$h>(plan, parent, $i as u32);
                    if new {
                        <$h::Subs as Upstream>::visit_all(plan, i);
                        leave::<$h>(plan, i);
                    }
                })*
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
// The plan of a pipeline
// ---------------------------------------------------------------------------

/// The plan of one pipeline: each node's `TypeId` in first-reached order (the
/// order is the node's *index*), its kind, first-reached parent, subtree
/// extent, execution position, and the upstream edges.
#[doc(hidden)]
pub struct Plan {
    /// Number of nodes recorded so far.
    len: usize,
    /// The nodes, by index.
    ids: [TypeId; MAX_COMPONENTS],
    /// Position by node: hash of the type name to start, `TypeId` to decide
    /// (see [`table`](crate::table)).
    table: Table,
    /// `KIND_*` of each node.
    kind: [u8; MAX_COMPONENTS],
    /// Index of the node each was first reached from (`NO_PARENT` for the
    /// anchor).
    first_parent: [u32; MAX_COMPONENTS],
    /// Position, in that parent's list of subscriptions, of the entry that
    /// first reached each node (two entries of one parent may name the same
    /// node, through an alias).
    first_pos: [u32; MAX_COMPONENTS],
    /// Nodes are in first-reached (depth-first pre-)order, so the first-parent
    /// subtree of node `i` is the range `i..end[i]`. 0 while the walk is still
    /// inside node `i`.
    end: [u32; MAX_COMPONENTS],
    /// Depth in the first-parent tree (anchor 0).
    depth: [u32; MAX_COMPONENTS],
    /// Execution position (post-order) of each node.
    post_of: [u32; MAX_COMPONENTS],
    /// Index of the node at each execution position.
    by_post: [u32; MAX_COMPONENTS],
    /// Number of nodes finished so far.
    done: usize,
    /// Is the root upstream of this node (or the node itself)? Set when the
    /// node is left.
    reach: [bool; MAX_COMPONENTS],
    /// Has a subscription of this node been found to be reachable so far?
    has_up: [bool; MAX_COMPONENTS],
    /// The subscription entries: `edge_from[e]` subscribes to `edge_to[e]`
    /// (both node indices), in the order the walk meets them.
    edge_from: [u32; MAX_EDGES],
    edge_to: [u32; MAX_EDGES],
    n_edges: usize,
    /// The pipeline's root type, and its index once the walk has met it.
    root_id: TypeId,
    root_idx: u32,
    /// The first error the walk met (0 = none) and its message. The walk
    /// carries on with a plan that is merely not meaningful: failing in the
    /// plan itself, inside the types computed from it, makes the compiler
    /// search for a very long time. [`Plan::check`] reports it, once.
    failed: bool,
    msg: [u8; MSG],
    msg_len: usize,
}

impl Plan {
    /// The plan for anchor `A` of a pipeline whose root type has `root_id`:
    /// the depth-first walk from it.
    pub(crate) const fn of<A: Node>(root_id: TypeId) -> Plan
    where
        A::Subs: [const] Upstream,
    {
        let mut plan = Plan {
            len: 0,
            ids: [BLANK; MAX_COMPONENTS],
            table: Table::new(),
            kind: [0; MAX_COMPONENTS],
            first_parent: [NO_PARENT; MAX_COMPONENTS],
            first_pos: [0; MAX_COMPONENTS],
            end: [0; MAX_COMPONENTS],
            depth: [0; MAX_COMPONENTS],
            post_of: [0; MAX_COMPONENTS],
            by_post: [0; MAX_COMPONENTS],
            done: 0,
            reach: [false; MAX_COMPONENTS],
            has_up: [false; MAX_COMPONENTS],
            edge_from: [0; MAX_EDGES],
            edge_to: [0; MAX_EDGES],
            n_edges: 0,
            root_id,
            root_idx: NO_PARENT,
            failed: false,
            msg: [0; MSG],
            msg_len: 0,
        };
        visit::<A>(&mut plan, NO_PARENT, 0);
        if !plan.failed && (plan.root_idx == NO_PARENT || plan.post_of[plan.root_idx as usize] != 0) {
            plan.fail("the sinks", " do not depend on the root");
        }
        plan
    }

    /// Record the first error.
    const fn fail(&mut self, name: &str, tail: &str) {
        if !self.failed {
            let (msg, len) = message(name, tail);
            self.failed = true;
            self.msg = msg;
            self.msg_len = len;
        }
    }

    /// Panic (in constant evaluation) with the first error the walk met, if any.
    pub(crate) const fn check(&self) {
        if self.failed {
            let (msg, _) = self.msg.split_at(self.msg_len);
            match core::str::from_utf8(msg) {
                Ok(s) => core::panicking::panic_display(&s),
                Err(_) => panic!("invalid message"),
            }
        }
    }

    /// Is the node with this `TypeId` in the plan? (Run time, at build.)
    pub(crate) fn contains(&self, id: TypeId) -> bool {
        self.ids[..self.len].contains(&id)
    }

    /// Plan index of the node with this name hash and `TypeId`.
    pub(crate) const fn index_of(&self, hash: u64, id: TypeId) -> usize {
        match self.table.at(self.table.slot(&self.ids, hash, id)) {
            Some(i) => i,
            None => panic!("node not in plan"),
        }
    }

    /// Number of nodes: the root, the components and the anchor.
    pub(crate) const fn len(&self) -> usize {
        self.len
    }

    /// Kind (`KIND_*`) of the node at plan index `i`.
    #[allow(dead_code)]
    pub(crate) const fn kind_of(&self, i: usize) -> u8 {
        self.kind[i]
    }

    /// Execution position of the node at plan index `i`.
    pub(crate) const fn post_of_idx(&self, i: usize) -> usize {
        self.post_of[i] as usize
    }

    /// Plan index of the node at execution position `p`.
    pub(crate) const fn idx_at_post(&self, p: usize) -> usize {
        self.by_post[p] as usize
    }

    /// Number of subscription entries of the node at execution position `p`.
    #[allow(dead_code)]
    pub(crate) const fn upstream_count(&self, p: usize) -> usize {
        let i = self.by_post[p] as usize;
        let mut n = 0;
        let mut e = 0;
        while e < self.n_edges {
            if self.edge_from[e] as usize == i {
                n += 1;
            }
            e += 1;
        }
        n
    }

    /// The execution position of the `k`-th subscription entry (listing order,
    /// aliases counted separately) of the node at execution position `p`.
    #[allow(dead_code)]
    pub(crate) const fn upstream(&self, p: usize, k: usize) -> usize {
        let i = self.by_post[p] as usize;
        let mut seen = 0;
        let mut e = 0;
        while e < self.n_edges {
            if self.edge_from[e] as usize == i {
                if seen == k {
                    return self.post_of[self.edge_to[e] as usize] as usize;
                }
                seen += 1;
            }
            e += 1;
        }
        panic!("no such subscription")
    }

    /// How to get from the node at plan index `a` to its descendant `j` in the
    /// first-parent tree: 0 same, 1 child, 2 further down.
    pub(crate) const fn kind(&self, a: usize, j: usize) -> u8 {
        if a == j {
            0
        } else if self.first_parent[j] as usize == a {
            1
        } else {
            if !(a < j && j < self.end[a] as usize) {
                panic!("not a descendant");
            }
            2
        }
    }

    /// The ancestor of `j` (a descendant of `a`, at least 2 levels below)
    /// halfway between `a` and `j` in the first-parent tree.
    pub(crate) const fn mid(&self, a: usize, j: usize) -> usize {
        let target = self.depth[a] + (self.depth[j] - self.depth[a]) / 2;
        let mut x = j;
        while self.depth[x] > target {
            x = self.first_parent[x] as usize;
        }
        x
    }

    /// Position, in the first parent's list, of the entry that first reached
    /// node `i`.
    pub(crate) const fn first_pos_of(&self, i: usize) -> u32 {
        self.first_pos[i]
    }
}

/// An anchor whose plan has been computed, evaluated once per pipeline type.
#[doc(hidden)]
pub trait Planned: Node {
    const PLAN: &'static Plan;
    /// Evaluating this reports the plan's error, if any (a const panic naming
    /// the component). Evaluated once, by `Pipeline::build`.
    const CHECK: () = Self::PLAN.check();
}

#[cfg(test)]
mod trace {
    use super::*;
    use crate::node::{KIND_COMPONENT, KIND_ROOT};
    use crate::sinks::Anchor;
    use crate::{Component, ComponentSubscription, Root};

    #[derive(Root)]
    pub struct Poll;

    macro_rules! comp {
        ($name:ident, [$($n:ident: $d:ty),*]) => {
            #[derive(ComponentSubscription)]
            #[subscribe($($n: $d),*)]
            pub struct $name;
            impl Component for $name {
                type Config = ();
                type Output = u8;
                fn new((): (), _: Self::Init<'_>) -> (Self, u8) { ($name, 0) }
                fn update(&mut self, _: Self::Input<'_>) -> u8 { 0 }
            }
        };
    }
    comp!(A, [p: Poll]);
    comp!(B, [a: A, p: Poll]);
    comp!(C, [a: A]);

    #[test]
    fn plan_positions_and_upstreams() {
        let plan = <Anchor<Poll, (B, C)> as Planned>::PLAN;
        plan.check();
        // Post-order: Poll, A, B, C, anchor.
        assert_eq!(plan.len(), 5);
        assert_eq!(plan.kind_of(plan.idx_at_post(0)), KIND_ROOT);
        assert_eq!(plan.kind_of(plan.idx_at_post(1)), KIND_COMPONENT);
        assert_eq!(plan.kind_of(plan.idx_at_post(4)), KIND_ANCHOR);
        // B (position 2) lists A then Poll; C (3) lists A; the anchor lists B, C.
        assert_eq!((plan.upstream_count(2), plan.upstream(2, 0), plan.upstream(2, 1)), (2, 1, 0));
        assert_eq!((plan.upstream_count(3), plan.upstream(3, 0)), (1, 1));
        assert_eq!((plan.upstream_count(4), plan.upstream(4, 0), plan.upstream(4, 1)), (2, 2, 3));
        assert_eq!(plan.upstream_count(0), 0);
        assert!(plan.contains(TypeId::of::<A>()) && !plan.contains(TypeId::of::<u8>()));
    }
}
