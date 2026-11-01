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
use crate::node::{Node, KIND_ANCHOR, KIND_COMPONENT, KIND_ROOT};
use crate::select::ChainWalk;
use crate::table::Table;

/// Upper bound on nodes (the root, the components and the anchor) per
/// pipeline. Compile-time memory grows with MAX_COMPONENTS x (number of
/// nodes), so raise it only as needed.
pub const MAX_COMPONENTS: usize = 512;

/// Upper bound on subscription entries per pipeline, summed over all
/// components.
pub const MAX_EDGES: usize = 4096;

/// Upper bound on distinct selector decisions per pipeline (see
/// [`Plan::decision`]).
pub const MAX_DECISIONS: usize = 512;

/// Upper bound on the terms of one node's condition (after absorption).
pub const MAX_TERMS: usize = 64;

/// Sizes of the pools that hold terms (lists of decisions), the lists of
/// terms of each node, and the names kept for messages.
const TPOOL: usize = 16384;
const OPOOL: usize = 8192;
const NPOOL: usize = 8192;
/// Longest name kept for a message.
const NAME: usize = 96;

/// The first parent of the anchor: it was not reached from any node. Also
/// "none" in the plan's index arrays.
const NO_PARENT: u32 = u32::MAX;
const NONE: u32 = u32::MAX;
/// `flag` of a component whose firing nobody reads.
pub(crate) const NO_FLAG: usize = usize::MAX;

// ---------------------------------------------------------------------------
// Const panics with a name
// ---------------------------------------------------------------------------

/// Longest error message kept in the plan.
const MSG: usize = 512;

/// Assemble the parts into a byte buffer (`panic!` can't format in a `const
/// fn`). Returns the buffer and the length used.
const fn message(parts: &[&[u8]]) -> ([u8; MSG], usize) {
    let mut buf = [0u8; MSG];
    let mut n = 0;
    let mut p = 0;
    while p < parts.len() {
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

/// Record `N` (reached from `parent`, at position `pos` of its list, through
/// the selector chain `C`) unless it was reached before; either way record the
/// edge `parent -> N` and the decisions of its chain. Returns the plan index of
/// `N` and whether it is new, i.e. whether its subscriptions still have to be
/// walked.
///
/// This is split from the walk of the subscriptions (and from [`leave`]) so
/// that the generated [`Upstream`] impls can call them inline around the
/// recursion into `N::Subs`, which keeps constant evaluation at one stack frame
/// per graph layer. The trait solver's proof is unaffected: the only bound is
/// `N::Subs: [const] Upstream`, a trait on the tree rather than on the node.
const fn enter<N: Node, C: [const] ChainWalk<N::Output>>(plan: &mut Plan, parent: u32, pos: u32) -> (u32, bool) {
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
                "",
            );
        }
        let i = plan.len;
        if i == MAX_COMPONENTS {
            // Not recorded, not walked: the plan is not meaningful from here on.
            plan.fail("too many components: ", "raise dataflow_flat::MAX_COMPONENTS", "");
            return (0, false);
        }
        plan.table.put(slot, i);
        plan.ids[i] = id;
        plan.kind[i] = N::KIND;
        plan.listener[i] = N::LISTENER;
        plan.first_parent[i] = parent;
        plan.first_pos[i] = pos;
        plan.len = i + 1;
        plan.depth[i] = if parent == NO_PARENT { 0 } else { plan.depth[parent as usize] + 1 };
        (i, true)
    };
    if parent != NO_PARENT {
        let e = plan.n_edges;
        if e == MAX_EDGES {
            plan.fail("too many subscriptions: ", "raise dataflow_flat::MAX_EDGES", "");
        } else {
            let dn = C::walk(plan, i as u32, NONE);
            plan.edge_to[e] = i as u32;
            plan.edge_dn[e] = dn;
            plan.edge_next[e] = NONE;
            if plan.up_head[parent as usize] == NONE {
                plan.up_head[parent as usize] = e as u32;
            } else {
                plan.edge_next[plan.up_tail[parent as usize] as usize] = e as u32;
            }
            plan.up_tail[parent as usize] = e as u32;
            plan.n_edges = e + 1;
            if dn != NONE {
                plan.set_edge_name(e, type_name::<N>());
            }
        }
        // A node reached again is finished: if the root is upstream of it, it
        // is upstream of `parent` too. (A new node reports when it is left.)
        if !new && plan.reach[i] {
            plan.has_up[parent as usize] = true;
        }
    }
    (i as u32, new)
}

/// A node's walk is over: record its subtree end and execution position, check
/// that it is reachable from the root (some node it subscribes to is), and
/// compute its condition.
const fn leave<N: Node>(plan: &mut Plan, i: u32) {
    let i = i as usize;
    plan.end[i] = plan.len as u32;
    plan.post_of[i] = plan.done as u32;
    plan.by_post[plan.done] = i as u32;
    plan.done += 1;
    let reach = N::KIND == KIND_ROOT || N::KIND == KIND_ANCHOR || plan.has_up[i];
    if !reach {
        plan.fail(type_name::<N>(), " is not reachable from the root", "");
    }
    plan.reach[i] = true;
    if N::KIND == KIND_ROOT {
        plan.root_idx = i as u32;
        plan.cond_root(i);
    } else if N::KIND == KIND_COMPONENT {
        plan.cond_comp(i, type_name::<N>());
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
    let (i, new) = enter::<N, ()>(plan, parent, pos);
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
    /// children. An entry `Sub<E, C, I>` visits `E` with position `I`.
    fn visit_all(plan: &mut Plan, parent: u32);
}

macro_rules! upstream_leaf {
    ([$($h:ident $c:ident $i:ident)*]) => {
        #[allow(unused_variables)]
        const impl<$($h: Node, $c: [const] ChainWalk<<$h as Node>::Output>, const $i: usize),*> Upstream for ($(Sub<$h, $c, $i>,)*)
        where $($h::Subs: [const] Upstream,)*
        {
            fn visit_all(plan: &mut Plan, parent: u32) {
                $({
                    let (i, new) = enter::<$h, $c>(plan, parent, $i as u32);
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
/// extent, execution position, the upstream edges with their selector
/// decisions, and the conditions.
///
/// # Conditions
///
/// A *decision* is one selector applied at one place: the node it reads, the
/// decision before it in its chain, the selector's `TypeId` and `KEY`. Edges
/// that share a prefix of a chain share its decisions (the per-parent trie).
/// A *term* is a set of decisions; a node's *condition* is a set of terms, the
/// ways it can fire in a step (a term holds when all its decisions match):
///
/// * the root's condition is the empty term, "always";
/// * a component's condition is the union over its edges of the edge's
///   condition: the parent's terms, each extended by the decisions of the
///   edge's chain.
///
/// After every insertion a term that another term implies (is a superset of)
/// is dropped (*absorption*), so a condition with the empty term is just
/// "always". The remaining terms must be pairwise *exclusive* (rule 4): two
/// terms are, when they hold decisions with the same parent and the same
/// previous decision but different keys (neither a guard).
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
    /// The subscription entries, in the order the walk meets them: the node
    /// each reads, the last decision of its selector chain (`NONE` if plain),
    /// and the next entry of the same subscriber (`NONE` at the end).
    edge_to: [u32; MAX_EDGES],
    edge_dn: [u32; MAX_EDGES],
    edge_next: [u32; MAX_EDGES],
    /// Where the name of the node a selected edge reads is kept (message pool).
    edge_nm: [u32; MAX_EDGES],
    n_edges: usize,
    /// First and last entry of each subscriber (`NONE` if it has none).
    up_head: [u32; MAX_COMPONENTS],
    up_tail: [u32; MAX_COMPONENTS],
    /// The decisions: the node whose output is selected, the previous decision
    /// of the chain, the selector's `TypeId` and its `KEY`.
    dn_len: usize,
    dn_node: [u32; MAX_DECISIONS],
    dn_prev: [u32; MAX_DECISIONS],
    dn_id: [TypeId; MAX_DECISIONS],
    dn_key: [u32; MAX_DECISIONS],
    /// Is the decision a keyed selector (`On`), and where its name is kept.
    dn_keyed: [bool; MAX_DECISIONS],
    dn_nm: [u32; MAX_DECISIONS],
    /// Is the node a `Listener`? Its requests: `lk_n[i]` sorted, distinct keys
    /// at `lk_pool[lk_off[i]..]`.
    listener: [bool; MAX_COMPONENTS],
    lk_off: [u32; MAX_COMPONENTS],
    lk_n: [u32; MAX_COMPONENTS],
    lk_pool: [u32; MAX_DECISIONS],
    lk_len: usize,
    /// Terms: at `tpool[o]` the number of decisions, then the decisions.
    /// Offset 0 is the empty term ("always").
    tpool: [u32; TPOOL],
    tp_len: usize,
    /// The terms of each node's condition: `cn[i]` of them, as offsets into
    /// `tpool`, in `olist[co[i]..]`.
    olist: [u32; OPOOL],
    ol_len: usize,
    co: [u32; MAX_COMPONENTS],
    cn: [u32; MAX_COMPONENTS],
    /// Is the node's condition "always"?
    always: [bool; MAX_COMPONENTS],
    /// Index into the push's `fired` flags of a component whose firing is
    /// read by a conditional component (`NONE` if nobody reads it).
    flag: [u32; MAX_COMPONENTS],
    n_flags: usize,
    /// Component position of each component (`NONE` for the root and the
    /// anchor) and the node index of each component position.
    cpos: [u32; MAX_COMPONENTS],
    by_cpos: [u32; MAX_COMPONENTS],
    n_comp: usize,
    /// Names for messages.
    nm_pool: [u8; NPOOL],
    nm_len: usize,
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
            edge_to: [0; MAX_EDGES],
            edge_dn: [NONE; MAX_EDGES],
            edge_next: [NONE; MAX_EDGES],
            edge_nm: [NONE; MAX_EDGES],
            n_edges: 0,
            up_head: [NONE; MAX_COMPONENTS],
            up_tail: [NONE; MAX_COMPONENTS],
            dn_len: 0,
            dn_node: [0; MAX_DECISIONS],
            dn_prev: [NONE; MAX_DECISIONS],
            dn_id: [BLANK; MAX_DECISIONS],
            dn_key: [0; MAX_DECISIONS],
            dn_keyed: [false; MAX_DECISIONS],
            dn_nm: [NONE; MAX_DECISIONS],
            listener: [false; MAX_COMPONENTS],
            lk_off: [0; MAX_COMPONENTS],
            lk_n: [0; MAX_COMPONENTS],
            lk_pool: [0; MAX_DECISIONS],
            lk_len: 0,
            tpool: [0; TPOOL],
            tp_len: 1,
            olist: [0; OPOOL],
            ol_len: 0,
            co: [0; MAX_COMPONENTS],
            cn: [0; MAX_COMPONENTS],
            always: [false; MAX_COMPONENTS],
            flag: [NONE; MAX_COMPONENTS],
            n_flags: 0,
            cpos: [NONE; MAX_COMPONENTS],
            by_cpos: [NONE; MAX_COMPONENTS],
            n_comp: 0,
            nm_pool: [0; NPOOL],
            nm_len: 0,
            root_id,
            root_idx: NO_PARENT,
            failed: false,
            msg: [0; MSG],
            msg_len: 0,
        };
        visit::<A>(&mut plan, NO_PARENT, 0);
        if !plan.failed && (plan.root_idx == NO_PARENT || plan.post_of[plan.root_idx as usize] != 0) {
            plan.fail("the sinks", " do not depend on the root", "");
        }
        if !plan.failed {
            plan.number_components();
            plan.collect_requests();
        }
        plan
    }

    /// Record the first error.
    const fn fail(&mut self, a: &str, b: &str, c: &str) {
        self.fail4(a, b, c, "");
    }

    const fn fail4(&mut self, a: &str, b: &str, c: &str, d: &str) {
        self.fail_msg(message(&[a.as_bytes(), b.as_bytes(), c.as_bytes(), d.as_bytes()]));
    }

    const fn fail_msg(&mut self, m: ([u8; MSG], usize)) {
        if !self.failed {
            self.failed = true;
            self.msg = m.0;
            self.msg_len = m.1;
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

    // -- names ------------------------------------------------------------

    /// Keep (the start of) `name` for the messages about edge `e`.
    const fn set_edge_name(&mut self, e: usize, name: &str) {
        self.edge_nm[e] = self.keep_name(name);
    }

    /// Keep (the start of) `name`; returns where, or `NONE` if the pool is full.
    const fn keep_name(&mut self, name: &str) -> u32 {
        let bytes = name.as_bytes();
        let n = if bytes.len() < NAME { bytes.len() } else { NAME };
        if self.nm_len + 1 + n > NPOOL {
            return NONE;
        }
        let at = self.nm_len;
        self.nm_pool[at] = n as u8;
        let mut k = 0;
        while k < n {
            self.nm_pool[at + 1 + k] = bytes[k];
            k += 1;
        }
        self.nm_len = at + 1 + n;
        at as u32
    }

    const fn edge_name(&self, e: usize) -> &str {
        self.name_at(self.edge_nm[e], "a selected node")
    }

    const fn name_at(&self, at: u32, default: &'static str) -> &str {
        if at == NONE {
            return default;
        }
        let at = at as usize;
        let n = self.nm_pool[at] as usize;
        let (_, rest) = self.nm_pool.split_at(at + 1);
        let (name, _) = rest.split_at(n);
        match core::str::from_utf8(name) {
            Ok(s) => s,
            Err(_) => default,
        }
    }

    // -- decisions and terms ----------------------------------------------

    /// The decision for selector `id` (with `key`) applied to the output of
    /// node `node` after decision `prev` of its chain (`NONE` for the first):
    /// found, or recorded. Edges that share a chain prefix share its decisions.
    pub(crate) const fn decision(&mut self, node: u32, prev: u32, id: TypeId, key: u32, keyed: bool, label: &str) -> u32 {
        let mut d = 0;
        while d < self.dn_len {
            if self.dn_node[d] == node && self.dn_prev[d] == prev && self.dn_id[d] == id {
                return d as u32;
            }
            d += 1;
        }
        if keyed {
            // Two symbols with one key at one place would be one alternative
            // with two names: reject.
            let mut x = 0;
            while x < self.dn_len {
                if self.dn_keyed[x] && self.dn_node[x] == node && self.dn_prev[x] == prev && self.dn_key[x] == key {
                    let mut digits = [0u8; 10];
                    let mut nd = 0;
                    let mut v = key;
                    loop {
                        digits[9 - nd] = b'0' + (v % 10) as u8;
                        nd += 1;
                        v /= 10;
                        if v == 0 {
                            break;
                        }
                    }
                    let (_, num) = digits.split_at(10 - nd);
                    let other = self.name_at(self.dn_nm[x], "a symbol");
                    let m = message(&[b"symbols ", other.as_bytes(), b" and ", label.as_bytes(), b" share key ", num]);
                    self.fail_msg(m);
                    return x as u32;
                }
                x += 1;
            }
        }
        if self.dn_len == MAX_DECISIONS {
            self.fail("too many selectors: ", "raise dataflow_flat::MAX_DECISIONS", "");
            return prev;
        }
        let d = self.dn_len;
        self.dn_node[d] = node;
        self.dn_prev[d] = prev;
        self.dn_id[d] = id;
        self.dn_key[d] = key;
        self.dn_keyed[d] = keyed;
        if keyed {
            self.dn_nm[d] = self.keep_name(label);
        }
        self.dn_len = d + 1;
        d as u32
    }

    const fn term_len(&self, t: u32) -> usize {
        self.tpool[t as usize] as usize
    }
    const fn term_item(&self, t: u32, k: usize) -> u32 {
        self.tpool[t as usize + 1 + k]
    }
    const fn term_has(&self, t: u32, x: u32) -> bool {
        let n = self.term_len(t);
        let mut k = 0;
        while k < n {
            if self.term_item(t, k) == x {
                return true;
            }
            k += 1;
        }
        false
    }
    /// Does term `b` hold every decision of term `a` (a implied by b)?
    const fn subset(&self, a: u32, b: u32) -> bool {
        let n = self.term_len(a);
        let mut k = 0;
        while k < n {
            if !self.term_has(b, self.term_item(a, k)) {
                return false;
            }
            k += 1;
        }
        true
    }
    /// Does term `t` hold decision `d` and every decision before it in its chain?
    const fn holds_path(&self, t: u32, d: u32) -> bool {
        let mut x = d;
        while x != NONE {
            if !self.term_has(t, x) {
                return false;
            }
            x = self.dn_prev[x as usize];
        }
        true
    }
    /// The term `t` extended by the decisions of the chain ending at `d`
    /// (`t` itself if `d` is `NONE`). Returns `NONE` if the pool is full.
    const fn extend(&mut self, t: u32, d: u32) -> u32 {
        if d == NONE {
            return t;
        }
        let base = self.tp_len;
        let mut room = self.term_len(t) + 1;
        let mut x = d;
        while x != NONE {
            room += 1;
            x = self.dn_prev[x as usize];
        }
        if base + room > TPOOL {
            self.fail("too many conditions: ", "raise dataflow_flat's term pool", "");
            return NONE;
        }
        let n = self.term_len(t);
        let mut k = 0;
        while k < n {
            self.tpool[base + 1 + k] = self.term_item(t, k);
            k += 1;
        }
        let mut count = n;
        let mut x = d;
        while x != NONE {
            if !self.term_has(t, x) {
                self.tpool[base + 1 + count] = x;
                count += 1;
            }
            x = self.dn_prev[x as usize];
        }
        self.tpool[base] = count as u32;
        self.tp_len = base + 1 + count;
        base as u32
    }
    /// Can terms `a` and `b` never hold in the same step? They can if they hold
    /// decisions at the same place (same node, same previous decision) with
    /// different keys, neither a guard.
    const fn exclusive(&self, a: u32, b: u32) -> bool {
        let na = self.term_len(a);
        let nb = self.term_len(b);
        let mut i = 0;
        while i < na {
            let x = self.term_item(a, i) as usize;
            let mut j = 0;
            while j < nb {
                let y = self.term_item(b, j) as usize;
                if x != y
                    && self.dn_node[x] == self.dn_node[y]
                    && self.dn_prev[x] == self.dn_prev[y]
                    && self.dn_key[x] != self.dn_key[y]
                    && self.dn_key[x] != crate::select::GUARD
                    && self.dn_key[y] != crate::select::GUARD
                {
                    return true;
                }
                j += 1;
            }
            i += 1;
        }
        false
    }

    // -- conditions -------------------------------------------------------

    /// The root fires on every push.
    const fn cond_root(&mut self, i: usize) {
        self.co[i] = self.ol_len as u32;
        self.cn[i] = 1;
        self.always[i] = true;
        self.olist[self.ol_len] = 0;
        self.ol_len += 1;
    }

    /// Compute the condition of component `i` from its edges, and check rules
    /// 4 (exclusive joins) and 5 (selected edges implied by the condition).
    const fn cond_comp(&mut self, i: usize, name: &str) {
        if self.failed {
            return;
        }
        let mut offs = [0u32; MAX_TERMS];
        let mut n = 0usize;
        let mut e = self.up_head[i];
        while e != NONE {
            let u = self.edge_to[e as usize] as usize;
            let d = self.edge_dn[e as usize];
            let cn = self.cn[u] as usize;
            let mut k = 0;
            while k < cn {
                let tp = self.olist[self.co[u] as usize + k];
                let t = self.extend(tp, d);
                if t == NONE {
                    return;
                }
                if !insert_term(self, &mut offs, &mut n, t) {
                    self.fail(name, " has more than ", "64 distinct conditions (MAX_TERMS)");
                    return;
                }
                k += 1;
            }
            e = self.edge_next[e as usize];
        }
        // Rule 4: after absorption, the terms are pairwise exclusive.
        let mut a = 0;
        while a < n {
            let mut b = a + 1;
            while b < n {
                if !self.exclusive(offs[a], offs[b]) {
                    self.fail(name, " joins branches that can fire in the same step", "");
                    return;
                }
                b += 1;
            }
            a += 1;
        }
        // Rule 5: every term of the condition implies each selected edge.
        let mut e = self.up_head[i];
        while e != NONE {
            let u = self.edge_to[e as usize] as usize;
            let d = self.edge_dn[e as usize];
            if d != NONE {
                let mut a = 0;
                while a < n {
                    let t = offs[a];
                    let mut ok = false;
                    let cn = self.cn[u] as usize;
                    let mut k = 0;
                    while k < cn {
                        let tp = self.olist[self.co[u] as usize + k];
                        if self.subset(tp, t) && self.holds_path(t, d) {
                            ok = true;
                        }
                        k += 1;
                    }
                    if !ok {
                        let m = message(&[
                            name.as_bytes(),
                            b" reads ",
                            self.edge_name(e as usize).as_bytes(),
                            b" through selectors in steps where they do not match; subscribe to a component that stores it",
                        ]);
                        self.fail_msg(m);
                        return;
                    }
                    a += 1;
                }
            }
            e = self.edge_next[e as usize];
        }
        // Store.
        if self.ol_len + n > OPOOL {
            self.fail("too many conditions: ", "raise dataflow_flat's term list pool", "");
            return;
        }
        self.co[i] = self.ol_len as u32;
        self.cn[i] = n as u32;
        let mut k = 0;
        while k < n {
            self.olist[self.ol_len + k] = offs[k];
            k += 1;
        }
        self.ol_len += n;
        let always = n == 1 && self.term_len(offs[0]) == 0;
        self.always[i] = always;
        // The firing of the components this one reads is read by its check.
        if !always {
            let mut e = self.up_head[i];
            while e != NONE {
                let u = self.edge_to[e as usize] as usize;
                if self.kind[u] == KIND_COMPONENT && !self.always[u] && self.flag[u] == NONE {
                    self.flag[u] = self.n_flags as u32;
                    self.n_flags += 1;
                }
                e = self.edge_next[e as usize];
            }
        }
    }

    /// The requests of the listeners: for each listener node, the sorted,
    /// distinct keys of the keyed decisions that read its output, directly or
    /// after other selectors.
    const fn collect_requests(&mut self) {
        let mut i = 0;
        while i < self.len {
            if self.listener[i] {
                self.lk_off[i] = self.lk_len as u32;
                let mut n = 0usize;
                let mut d = 0;
                while d < self.dn_len {
                    if self.dn_keyed[d] && self.dn_node[d] as usize == i {
                        // Insert the key into the sorted run at lk_pool[off..off+n].
                        let key = self.dn_key[d];
                        let off = self.lk_len;
                        let mut at = 0;
                        while at < n && self.lk_pool[off + at] < key {
                            at += 1;
                        }
                        if !(at < n && self.lk_pool[off + at] == key) {
                            let mut m = n;
                            while m > at {
                                self.lk_pool[off + m] = self.lk_pool[off + m - 1];
                                m -= 1;
                            }
                            self.lk_pool[off + at] = key;
                            n += 1;
                        }
                    }
                    d += 1;
                }
                self.lk_n[i] = n as u32;
                self.lk_len += n;
            }
            i += 1;
        }
    }

    /// The keys requested of the listener at plan index `i` (empty for other nodes).
    pub(crate) fn listener_keys(&self, i: usize) -> &[u32] {
        let off = self.lk_off[i] as usize;
        &self.lk_pool[off..off + self.lk_n[i] as usize]
    }

    /// Component positions: the components in execution order.
    const fn number_components(&mut self) {
        let mut p = 0;
        while p < self.done {
            let i = self.by_post[p] as usize;
            if self.kind[i] == KIND_COMPONENT {
                self.cpos[i] = self.n_comp as u32;
                self.by_cpos[self.n_comp] = i as u32;
                self.n_comp += 1;
            }
            p += 1;
        }
    }

    // -- queries ------------------------------------------------------------

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

    /// Number of components.
    pub(crate) const fn n_components(&self) -> usize {
        self.n_comp
    }

    /// Plan index of the component at component position `q`.
    pub(crate) const fn idx_of_comp(&self, q: usize) -> usize {
        self.by_cpos[q] as usize
    }

    /// Component position of the component at plan index `i`.
    pub(crate) const fn comp_pos_of_idx(&self, i: usize) -> usize {
        self.cpos[i] as usize
    }

    /// Kind (`KIND_*`) of the node at plan index `i`.
    #[allow(dead_code)]
    pub(crate) const fn kind_of(&self, i: usize) -> u8 {
        self.kind[i]
    }

    /// Execution position of the node at plan index `i`.
    #[allow(dead_code)]
    pub(crate) const fn post_of_idx(&self, i: usize) -> usize {
        self.post_of[i] as usize
    }

    /// Plan index of the node at execution position `p`.
    #[allow(dead_code)]
    pub(crate) const fn idx_at_post(&self, p: usize) -> usize {
        self.by_post[p] as usize
    }

    /// Does the component at plan index `i` run in every step?
    pub(crate) const fn always_idx(&self, i: usize) -> bool {
        self.always[i]
    }

    /// The `fired` flag of the component at plan index `i` (`NO_FLAG` if no
    /// condition reads whether it fired).
    pub(crate) const fn flag_idx(&self, i: usize) -> usize {
        if self.flag[i] == NONE {
            NO_FLAG
        } else {
            self.flag[i] as usize
        }
    }

    /// Number of `fired` flags a push needs.
    pub(crate) const fn n_flags(&self) -> usize {
        self.n_flags
    }

    /// Number of terms of the condition of the node at plan index `i`.
    #[allow(dead_code)]
    pub(crate) const fn n_terms(&self, i: usize) -> usize {
        self.cn[i] as usize
    }

    /// Number of subscription entries of the node at execution position `p`.
    #[allow(dead_code)]
    pub(crate) const fn upstream_count(&self, p: usize) -> usize {
        let mut e = self.up_head[self.by_post[p] as usize];
        let mut n = 0;
        while e != NONE {
            n += 1;
            e = self.edge_next[e as usize];
        }
        n
    }

    /// The execution position of the `k`-th subscription entry (listing order,
    /// aliases counted separately) of the node at execution position `p`.
    #[allow(dead_code)]
    pub(crate) const fn upstream(&self, p: usize, k: usize) -> usize {
        let mut e = self.up_head[self.by_post[p] as usize];
        let mut seen = 0;
        while e != NONE {
            if seen == k {
                return self.post_of[self.edge_to[e as usize] as usize] as usize;
            }
            seen += 1;
            e = self.edge_next[e as usize];
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

/// Add term `t` to the set `offs[..*n]` unless a term already there implies it
/// is redundant (is a subset of `t`); drop the terms `t` makes redundant.
/// Returns false if the set is full.
const fn insert_term(plan: &Plan, offs: &mut [u32; MAX_TERMS], n: &mut usize, t: u32) -> bool {
    let mut k = 0;
    while k < *n {
        if plan.subset(offs[k], t) {
            return true;
        }
        k += 1;
    }
    let mut w = 0;
    k = 0;
    while k < *n {
        if !plan.subset(t, offs[k]) {
            offs[w] = offs[k];
            w += 1;
        }
        k += 1;
    }
    *n = w;
    if *n == MAX_TERMS {
        return false;
    }
    offs[*n] = t;
    *n += 1;
    true
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

    // Conditions and flags.
    pub struct K0;
    pub struct K1;
    impl crate::Select<Poll> for K0 {
        type Out = Poll;
        const KEY: u32 = 0;
        fn select(x: &Poll) -> Option<&Poll> { Some(x) }
    }
    impl crate::Select<Poll> for K1 {
        type Out = Poll;
        const KEY: u32 = 1;
        fn select(x: &Poll) -> Option<&Poll> { Some(x) }
    }
    macro_rules! scomp {
        ($name:ident, [$($sub:tt)*]) => {
            #[derive(ComponentSubscription)]
            #[subscribe($($sub)*)]
            pub struct $name;
            impl Component for $name {
                type Config = ();
                type Output = u8;
                fn new((): (), _: Self::Init<'_>) -> (Self, u8) { ($name, 0) }
                fn update(&mut self, _: Self::Input<'_>) -> u8 { 0 }
            }
        };
    }
    scomp!(X, [p: Poll => K0]);
    scomp!(Y, [p: Poll => K1]);
    scomp!(Z, [x: X, y: Y]);
    scomp!(W, [z: Z, p: Poll => K0]);

    #[test]
    fn conditions_terms_and_flags() {
        let plan = <Anchor<Poll, (Z,)> as Planned>::PLAN;
        plan.check();
        // Poll, X, Y, Z, anchor.
        let at = |p: usize| plan.idx_at_post(p);
        assert!(plan.always_idx(at(0)));
        assert!(!plan.always_idx(at(1)) && !plan.always_idx(at(2)) && !plan.always_idx(at(3)));
        assert_eq!((plan.n_terms(at(1)), plan.n_terms(at(2)), plan.n_terms(at(3))), (1, 1, 2));
        // X and Y are read by the conditional Z; Z is read by nobody.
        assert_eq!(plan.n_flags(), 2);
        assert_ne!(plan.flag_idx(at(1)), plan.flag_idx(at(2)));
        assert_eq!(plan.flag_idx(at(3)), NO_FLAG);
        assert_eq!(plan.n_components(), 3);
    }

    #[test]
    fn rule_5_is_reported_by_check() {
        // W = [z: Z (terms K0, K1), p: Poll => K0]: its condition has the K1 term, which does not imply K0.
        let plan = <Anchor<Poll, (W,)> as Planned>::PLAN;
        assert!(plan.failed);
        let msg = core::str::from_utf8(&plan.msg[..plan.msg_len]).unwrap();
        assert!(msg.contains("through selectors in steps where they do not match"), "{msg}");
    }
}
