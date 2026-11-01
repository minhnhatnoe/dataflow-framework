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
pub const MAX_TERMS: usize = 256;

/// Sizes of the push program (see [`Plan::build_program`]): nodes, child
/// entries, the stack of component lists of the construction, the stack of
/// small per-switch arrays, and the nesting depth of switches.
const MAX_PROG: usize = 32768;
const MAX_PCH: usize = 32768;
const MAX_LST: usize = 32768;
const KPOOL: usize = 32768;
const MAX_CTX: usize = 32;

/// Program node kinds.
pub(crate) const PG_SEQ: u8 = 0;
pub(crate) const PG_RUN: u8 = 1;
pub(crate) const PG_RUN_IF: u8 = 2;
pub(crate) const PG_SWITCH: u8 = 3;
/// A handler component's run: `begin`, the `on_update` of the handled edges listed for
/// the node, `finish` (see `Plan::mk_hrun`).
pub(crate) const PG_HRUN: u8 = 4;
/// Items (handled edges that may fire) of all handler runs in the program.
const HPOOL: usize = 32768;

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
        plan.handler[i] = N::HANDLER;
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
            plan.cur_sub = parent;
            plan.cur_pos = pos;
            plan.cur_h = false;
            let dn = C::walk(plan, i as u32, NONE);
            plan.edge_h[e] = plan.cur_h;
            plan.edge_pos[e] = pos;
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
    /// Is the node a handler component? Is edge `e` a handled edge, and the entry
    /// position that named it?
    handler: [bool; MAX_COMPONENTS],
    edge_h: [bool; MAX_EDGES],
    edge_pos: [u32; MAX_EDGES],
    cur_h: bool,
    /// Items of handler runs: entry position and mode (bit 0: the parent's flag
    /// must be read, bit 1: the selectors must be matched).
    hi_len: usize,
    hi_pos: [u32; HPOOL],
    hi_mode: [u8; HPOOL],
    hi_key: [u32; HPOOL],
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
    /// The subscriber and entry position the walk is at (owner of new decisions).
    cur_sub: u32,
    cur_pos: u32,
    /// Per decision: the subscriber (plan index) and entry position of the edge
    /// that first walked it.
    dn_owner: [u32; MAX_DECISIONS],
    dn_opos: [u32; MAX_DECISIONS],
    /// Does a guarded run read whether this component fired?
    flag_need: [bool; MAX_COMPONENTS],
    /// The push program: nodes (`pg_*`) and their children (`ch_*`).
    pg_len: usize,
    pg_kind: [u8; MAX_PROG],
    pg_a: [u32; MAX_PROG],
    pg_b: [u32; MAX_PROG],
    pg_c: [u32; MAX_PROG],
    pg_d: [u32; MAX_PROG],
    pg_e: [u32; MAX_PROG],
    ch_len: usize,
    ch_node: [u32; MAX_PCH],
    ch_key: [u32; MAX_PCH],
    root_prog: u32,
    /// Work stacks of the construction of the program.
    lst: [u32; MAX_LST],
    lst_len: usize,
    ib: [u32; MAX_LST],
    ib_len: usize,
    kpool: [u32; KPOOL],
    kp_len: usize,
    ctx: [u32; MAX_CTX],
    ctx_len: usize,
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
            handler: [false; MAX_COMPONENTS],
            edge_h: [false; MAX_EDGES],
            edge_pos: [0; MAX_EDGES],
            cur_h: false,
            hi_len: 0,
            hi_pos: [0; HPOOL],
            hi_mode: [0; HPOOL],
            hi_key: [0; HPOOL],
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
            cur_sub: NONE,
            cur_pos: 0,
            dn_owner: [NONE; MAX_DECISIONS],
            dn_opos: [0; MAX_DECISIONS],
            flag_need: [false; MAX_COMPONENTS],
            pg_len: 0,
            pg_kind: [0; MAX_PROG],
            pg_a: [0; MAX_PROG],
            pg_b: [0; MAX_PROG],
            pg_c: [0; MAX_PROG],
            pg_d: [0; MAX_PROG],
            pg_e: [0; MAX_PROG],
            ch_len: 0,
            ch_node: [0; MAX_PCH],
            ch_key: [0; MAX_PCH],
            root_prog: NONE,
            lst: [0; MAX_LST],
            lst_len: 0,
            ib: [0; MAX_LST],
            ib_len: 0,
            kpool: [0; KPOOL],
            kp_len: 0,
            ctx: [0; MAX_CTX],
            ctx_len: 0,
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
        if !plan.failed {
            plan.build_program();
        }
        if plan.failed {
            // The program of a failed plan is empty (the types computed from it
            // must still be well-formed; `check` reports the error).
            plan.pg_len = 0;
            plan.ch_len = 0;
            plan.root_prog = plan.new_node(PG_SEQ, 0, 0);
        }
        plan
    }

    /// The edge being walked is a handled edge (called by the `Hd` chain wrapper).
    pub(crate) const fn mark_handled(&mut self) {
        self.cur_h = true;
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
        self.dn_owner[d] = self.cur_sub;
        self.dn_opos[d] = self.cur_pos;
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
                    self.fail(name, " has more than ", "256 distinct conditions (MAX_TERMS)");
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
            if d != NONE && !self.edge_h[e as usize] {
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
        if !always || self.handler[i] {
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


    // -- the push program: construction ---------------------------------------
    //
    // The program is a tree of nodes: `Seq` (children in order), `Run` (a
    // component, unconditionally), `RunIf` (a component guarded by its edges,
    // as the flag-based push), `Switch` (branch on the key of the alternative
    // a value is, one child per key). It is built by `emit` over a *context*
    // of decisions known to hold: components whose condition has a term inside
    // the context run straight-line; consecutive components that all wait for
    // a decision at the same place of a node's trie form a segment, which
    // becomes one switch whose branches repeat the process with the decision
    // added. Order is kept: every list is in execution order and a segment is
    // a run of consecutive entries of the list.

    const fn dec_equiv(&self, a: u32, b: u32) -> bool {
        let (a, b) = (a as usize, b as usize);
        a == b
            || (self.dn_node[a] == self.dn_node[b]
                && self.dn_prev[a] == self.dn_prev[b]
                && self.dn_key[a] == self.dn_key[b]
                && self.dn_key[a] != crate::select::GUARD)
    }
    const fn held(&self, d: u32) -> bool {
        let mut k = 0;
        while k < self.ctx_len {
            if self.dec_equiv(self.ctx[k], d) {
                return true;
            }
            k += 1;
        }
        false
    }
    /// Does a decision of term `t` exclude a decision of the context?
    const fn conflicts(&self, t: u32) -> bool {
        let n = self.term_len(t);
        let mut i = 0;
        while i < n {
            let x = self.term_item(t, i) as usize;
            let mut j = 0;
            while j < self.ctx_len {
                let y = self.ctx[j] as usize;
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
    const fn term_held(&self, t: u32) -> bool {
        let n = self.term_len(t);
        let mut i = 0;
        while i < n {
            if !self.held(self.term_item(t, i)) {
                return false;
            }
            i += 1;
        }
        true
    }
    /// (some term is consistent with the context, some such term holds in it)
    const fn consistent_static(&self, i: usize) -> (bool, bool) {
        let mut cons = false;
        let mut k = 0;
        let cn = self.cn[i] as usize;
        while k < cn {
            let t = self.olist[self.co[i] as usize + k];
            if !self.conflicts(t) {
                cons = true;
                if self.term_held(t) {
                    return (true, true);
                }
            }
            k += 1;
        }
        (cons, false)
    }
    /// Has node `x` run, with its output current, whenever the context holds?
    const fn node_ready(&self, x: usize) -> bool {
        if self.kind[x] == KIND_ROOT {
            return true;
        }
        let mut k = 0;
        let cn = self.cn[x] as usize;
        while k < cn {
            let t = self.olist[self.co[x] as usize + k];
            if !self.conflicts(t) && self.term_held(t) {
                return true;
            }
            k += 1;
        }
        false
    }
    /// The unheld keyed decision at place `(x, prev)` in term `t`, or `NONE`.
    const fn dec_at(&self, t: u32, x: u32, prev: u32) -> u32 {
        let n = self.term_len(t);
        let mut i = 0;
        while i < n {
            let d = self.term_item(t, i);
            let du = d as usize;
            if self.dn_node[du] == x
                && self.dn_prev[du] == prev
                && self.dn_key[du] != crate::select::GUARD
                && !self.held(d)
            {
                return d;
            }
            i += 1;
        }
        NONE
    }
    /// Does component `i` wait, in every consistent term, for a keyed decision at `(x, prev)`?
    const fn can_pend_at(&self, i: usize, x: u32, prev: u32) -> bool {
        let mut any = false;
        let cn = self.cn[i] as usize;
        let mut k = 0;
        while k < cn {
            let t = self.olist[self.co[i] as usize + k];
            if !self.conflicts(t) {
                if self.term_held(t) || self.dec_at(t, x, prev) == NONE {
                    return false;
                }
                any = true;
            }
            k += 1;
        }
        any
    }
    /// The first place at which component `i` can wait for a keyed decision.
    const fn first_pend(&self, i: usize) -> (bool, u32, u32) {
        let cn = self.cn[i] as usize;
        let mut k = 0;
        while k < cn {
            let t = self.olist[self.co[i] as usize + k];
            if !self.conflicts(t) {
                let n = self.term_len(t);
                let mut j = 0;
                while j < n {
                    let d = self.term_item(t, j);
                    let du = d as usize;
                    let (x, prev) = (self.dn_node[du], self.dn_prev[du]);
                    if self.dn_key[du] != crate::select::GUARD
                        && !self.held(d)
                        && (prev == NONE || self.held(prev))
                        && self.node_ready(x as usize)
                        && self.can_pend_at(i, x, prev)
                    {
                        return (true, x, prev);
                    }
                    j += 1;
                }
                return (false, 0, 0);
            }
            k += 1;
        }
        (false, 0, 0)
    }

    // -- handler components ----------------------------------------------------
    //
    // A handler's run is `begin`, one `on_update` per handled edge that fired,
    // `finish`. Under the context of decisions known to hold, each handled edge
    // is classified: it fired (its parent ran and its selectors match), it did
    // not (the parent cannot have run, or a selector decision excludes the
    // context), or it may have (then the parent's flag and/or the selector match
    // is tested at run time). Edges that did not fire are not in the list; the
    // others are listed in execution order of their parents.

    /// Does decision `x` exclude a decision of the context?
    const fn dec_conflicts(&self, x: usize) -> bool {
        let mut j = 0;
        while j < self.ctx_len {
            let y = self.ctx[j] as usize;
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
        false
    }
    /// The chain ending at decision `d` under the context: 0 excluded, 1 held, 2 unknown.
    const fn path_state(&self, d: u32) -> u8 {
        let mut all = true;
        let mut x = d;
        while x != NONE {
            if self.dec_conflicts(x as usize) {
                return 0;
            }
            if !self.held(x) {
                all = false;
            }
            x = self.dn_prev[x as usize];
        }
        if all {
            1
        } else {
            2
        }
    }
    /// Did node `u` run, under the context: 0 no, 1 yes, 2 unknown.
    const fn ran_state(&self, u: usize) -> u8 {
        if self.kind[u] == KIND_ROOT {
            return 1;
        }
        let mut any = false;
        let cn = self.cn[u] as usize;
        let mut k = 0;
        while k < cn {
            let t = self.olist[self.co[u] as usize + k];
            if !self.conflicts(t) {
                if self.term_held(t) {
                    return 1;
                }
                any = true;
            }
            k += 1;
        }
        if any {
            2
        } else {
            0
        }
    }

    const fn mk_hrun(&mut self, i: usize, guarded: bool) -> u32 {
        let start = self.hi_len;
        let mut e = self.up_head[i];
        while e != NONE {
            let eu = e as usize;
            let u = self.edge_to[eu] as usize;
            if guarded && self.kind[u] == KIND_COMPONENT && !self.always[u] {
                self.flag_need[u] = true;
            }
            if self.edge_h[eu] {
                let ps = self.ran_state(u);
                let cs = if self.edge_dn[eu] == NONE { 1 } else { self.path_state(self.edge_dn[eu]) };
                if ps != 0 && cs != 0 {
                    if self.hi_len == HPOOL {
                        self.fail("too many handled edges in the push program: ", "raise dataflow_flat's HPOOL", "");
                        return 0;
                    }
                    let mode = (if ps == 2 { 1u8 } else { 0 }) | (if cs == 2 { 2u8 } else { 0 });
                    if ps == 2 && self.kind[u] == KIND_COMPONENT && !self.always[u] {
                        self.flag_need[u] = true;
                    }
                    let key = if self.kind[u] == KIND_ROOT { 0 } else { self.cpos[u] + 1 };
                    // insertion sort by (parent execution position, entry position)
                    let mut at = self.hi_len;
                    while at > start
                        && (self.hi_key[at - 1] > key || (self.hi_key[at - 1] == key && self.hi_pos[at - 1] > self.edge_pos[eu]))
                    {
                        self.hi_key[at] = self.hi_key[at - 1];
                        self.hi_pos[at] = self.hi_pos[at - 1];
                        self.hi_mode[at] = self.hi_mode[at - 1];
                        at -= 1;
                    }
                    self.hi_key[at] = key;
                    self.hi_pos[at] = self.edge_pos[eu];
                    self.hi_mode[at] = mode;
                    self.hi_len += 1;
                }
            }
            e = self.edge_next[eu];
        }
        let n = self.new_node(PG_HRUN, self.cpos[i], guarded as u32);
        self.pg_c[n as usize] = start as u32;
        self.pg_d[n as usize] = (self.hi_len - start) as u32;
        n
    }

    /// Handler run: is it guarded by the firing of its edges.
    pub(crate) const fn hr_guarded(&self, n: usize) -> bool {
        self.pg_b[n] != 0
    }
    /// Handler run: first item and number of items.
    pub(crate) const fn hr_start(&self, n: usize) -> usize {
        self.pg_c[n] as usize
    }
    pub(crate) const fn hr_count(&self, n: usize) -> usize {
        self.pg_d[n] as usize
    }
    /// Item `k`: the entry position of the handled edge, and its mode.
    pub(crate) const fn hi_pos_of(&self, k: usize) -> usize {
        self.hi_pos[k] as usize
    }
    pub(crate) const fn hi_mode_of(&self, k: usize) -> u8 {
        self.hi_mode[k]
    }
    /// Number of handled edges of the node at plan index `i` (for tests).
    #[allow(dead_code)]
    pub(crate) const fn n_handled(&self, i: usize) -> usize {
        let mut e = self.up_head[i];
        let mut n = 0;
        while e != NONE {
            if self.edge_h[e as usize] {
                n += 1;
            }
            e = self.edge_next[e as usize];
        }
        n
    }

    const fn new_node(&mut self, kind: u8, a: u32, b: u32) -> u32 {
        let n = self.pg_len;
        if n == MAX_PROG {
            self.fail("the push program is too large: ", "raise dataflow_flat's MAX_PROG", "");
            return 0;
        }
        self.pg_kind[n] = kind;
        self.pg_a[n] = a;
        self.pg_b[n] = b;
        self.pg_len = n + 1;
        n as u32
    }
    const fn push_item(&mut self, n: u32) {
        if self.ib_len == MAX_LST {
            self.fail("the push program is too large: ", "raise dataflow_flat's list pool", "");
            return;
        }
        self.ib[self.ib_len] = n;
        self.ib_len += 1;
    }
    const fn mk_run(&mut self, i: usize, guarded: bool) -> u32 {
        if self.handler[i] {
            return self.mk_hrun(i, guarded);
        }
        if guarded {
            let mut e = self.up_head[i];
            while e != NONE {
                let u = self.edge_to[e as usize] as usize;
                if self.kind[u] == KIND_COMPONENT && !self.always[u] {
                    self.flag_need[u] = true;
                }
                e = self.edge_next[e as usize];
            }
        }
        let kind = if guarded { PG_RUN_IF } else { PG_RUN };
        self.new_node(kind, self.cpos[i], 0)
    }

    /// Binary search in `kpool[m..m+nk]`.
    const fn find_key(&self, m: usize, nk: usize, key: u32) -> (bool, usize) {
        let (mut lo, mut hi) = (0, nk);
        while lo < hi {
            let mid = (lo + hi) / 2;
            let v = self.kpool[m + mid];
            if v == key {
                return (true, mid);
            } else if v < key {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        (false, lo)
    }

    const fn find_decision(&self, x: u32, prev: u32, key: u32) -> u32 {
        let mut d = 0;
        while d < self.dn_len {
            if self.dn_node[d] == x && self.dn_prev[d] == prev && self.dn_key[d] == key {
                return d as u32;
            }
            d += 1;
        }
        NONE
    }

    /// The program for the components `lst[lo..hi]` under the current context.
    const fn emit(&mut self, lo: usize, hi: usize) -> u32 {
        let mark = self.ib_len;
        let mut k = lo;
        while k < hi && !self.failed {
            let i = self.lst[k] as usize;
            let (cons, stat) = self.consistent_static(i);
            if !cons {
                k += 1;
                continue;
            }
            if stat {
                let n = self.mk_run(i, false);
                self.push_item(n);
                k += 1;
                continue;
            }
            let (found, x, prev) = self.first_pend(i);
            if !found {
                let n = self.mk_run(i, true);
                self.push_item(n);
                k += 1;
                continue;
            }
            let mut j = k + 1;
            while j < hi {
                let i2 = self.lst[j] as usize;
                let (c2, s2) = self.consistent_static(i2);
                if c2 && (s2 || !self.can_pend_at(i2, x, prev)) {
                    break;
                }
                j += 1;
            }
            let n = self.emit_switch(k, j, x, prev);
            self.push_item(n);
            k = j;
        }
        let count = self.ib_len - mark;
        let node = if count == 1 {
            self.ib[mark]
        } else {
            let first = self.ch_len;
            if first + count > MAX_PCH {
                self.fail("the push program is too large: ", "raise dataflow_flat's child pool", "");
                self.ib_len = mark;
                return 0;
            }
            let mut c = 0;
            while c < count {
                self.ch_node[first + c] = self.ib[mark + c];
                c += 1;
            }
            self.ch_len = first + count;
            self.new_node(PG_SEQ, first as u32, count as u32)
        };
        self.ib_len = mark;
        node
    }

    /// A switch for the segment `lst[lo..hi]` at the place `(x, prev)`.
    const fn emit_switch(&mut self, lo: usize, hi: usize, x: u32, prev: u32) -> u32 {
        let m = self.kp_len;
        // 1. the keys, sorted
        let mut nk = 0;
        let mut q = lo;
        while q < hi {
            let i = self.lst[q] as usize;
            let cn = self.cn[i] as usize;
            let mut k = 0;
            while k < cn {
                let t = self.olist[self.co[i] as usize + k];
                if !self.conflicts(t) {
                    let d = self.dec_at(t, x, prev);
                    if d != NONE {
                        let key = self.dn_key[d as usize];
                        let (found, at) = self.find_key(m, nk, key);
                        if !found {
                            if m + nk + 1 >= KPOOL {
                                self.fail("the push program is too large: ", "raise dataflow_flat's key pool", "");
                                return 0;
                            }
                            let mut z = nk;
                            while z > at {
                                self.kpool[m + z] = self.kpool[m + z - 1];
                                z -= 1;
                            }
                            self.kpool[m + at] = key;
                            nk += 1;
                        }
                    }
                }
                k += 1;
            }
            q += 1;
        }
        // layout: keys | start (nk+1) | stamp (nk) | child (nk)
        let st = m + nk;
        let sm = st + nk + 1;
        let ch = sm + nk;
        if ch + nk > KPOOL {
            self.fail("the push program is too large: ", "raise dataflow_flat's key pool", "");
            return 0;
        }
        let mut z = 0;
        while z < nk + 1 {
            self.kpool[st + z] = 0;
            z += 1;
        }
        // 2. count and bucket (pass 0 counts, pass 1 fills)
        let base = self.lst_len;
        let mut pass = 0;
        while pass < 2 {
            let mut z = 0;
            while z < nk {
                self.kpool[sm + z] = 0;
                z += 1;
            }
            if pass == 1 {
                let mut sum = 0u32;
                let mut z = 0;
                while z < nk {
                    let c = self.kpool[st + z];
                    self.kpool[st + z] = sum;
                    self.kpool[ch + z] = sum;
                    sum += c;
                    z += 1;
                }
                self.kpool[st + nk] = sum;
                if base + sum as usize > MAX_LST {
                    self.fail("the push program is too large: ", "raise dataflow_flat's list pool", "");
                    return 0;
                }
            }
            let mut q = lo;
            while q < hi {
                let i = self.lst[q] as usize;
                let ord = (q - lo + 1) as u32;
                let cn = self.cn[i] as usize;
                let mut k = 0;
                while k < cn {
                    let t = self.olist[self.co[i] as usize + k];
                    if !self.conflicts(t) {
                        let d = self.dec_at(t, x, prev);
                        if d != NONE {
                            let (_, r) = self.find_key(m, nk, self.dn_key[d as usize]);
                            if self.kpool[sm + r] != ord {
                                self.kpool[sm + r] = ord;
                                if pass == 0 {
                                    self.kpool[st + r] += 1;
                                } else {
                                    let at = self.kpool[ch + r] as usize;
                                    self.lst[base + at] = i as u32;
                                    self.kpool[ch + r] += 1;
                                }
                            }
                        }
                    }
                    k += 1;
                }
                q += 1;
            }
            pass += 1;
        }
        let total = self.kpool[st + nk] as usize;
        self.lst_len = base + total;
        self.kp_len = ch + nk;
        // 3. the branches
        let mut r = 0;
        while r < nk && !self.failed {
            let key = self.kpool[m + r];
            let d = self.find_decision(x, prev, key);
            if self.ctx_len == MAX_CTX {
                self.fail("selectors are nested too deeply for the push program", "", "");
                return 0;
            }
            self.ctx[self.ctx_len] = d;
            self.ctx_len += 1;
            let (blo, bhi) = (base + self.kpool[st + r] as usize, base + self.kpool[st + r + 1] as usize);
            let c = self.emit(blo, bhi);
            self.ctx_len -= 1;
            self.kpool[ch + r] = c;
            r += 1;
        }
        if self.failed {
            return 0;
        }
        // 4. the node
        let first = self.ch_len;
        if first + nk > MAX_PCH {
            self.fail("the push program is too large: ", "raise dataflow_flat's child pool", "");
            return 0;
        }
        let mut r = 0;
        while r < nk {
            self.ch_node[first + r] = self.kpool[ch + r];
            self.ch_key[first + r] = self.kpool[m + r];
            r += 1;
        }
        self.ch_len = first + nk;
        // the owner of the selector at this place (first key's decision)
        let d0 = self.find_decision(x, prev, self.kpool[m]);
        let owner = self.dn_owner[d0 as usize] as usize;
        let mut depth = 0u32;
        let mut p = prev;
        while p != NONE {
            depth += 1;
            p = self.dn_prev[p as usize];
        }
        let n = self.new_node(PG_SWITCH, first as u32, nk as u32);
        self.pg_c[n as usize] = self.cpos[owner];
        self.pg_d[n as usize] = self.dn_opos[d0 as usize];
        self.pg_e[n as usize] = depth;
        self.lst_len = base;
        self.kp_len = m;
        n
    }

    const fn build_program(&mut self) {
        let n = self.n_comp;
        let mut q = 0;
        while q < n {
            self.lst[q] = self.by_cpos[q];
            q += 1;
        }
        self.lst_len = n;
        self.ctx_len = 0;
        self.root_prog = self.emit(0, n);
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
        if self.flag[i] == NONE || !self.flag_need[i] {
            NO_FLAG
        } else {
            self.flag[i] as usize
        }
    }

    /// Number of `fired` flags a push needs.
    pub(crate) const fn n_flags(&self) -> usize {
        let mut n = 0;
        let mut i = 0;
        while i < self.len {
            if self.flag_need[i] {
                n += 1;
            }
            i += 1;
        }
        n
    }

    // -- the push program -----------------------------------------------------
    pub(crate) const fn root_prog(&self) -> usize {
        self.root_prog as usize
    }
    pub(crate) const fn pg_kind_of(&self, n: usize) -> u8 {
        self.pg_kind[n]
    }
    /// Run: the component position. Seq/Switch: first child entry.
    pub(crate) const fn pg_a_of(&self, n: usize) -> usize {
        self.pg_a[n] as usize
    }
    /// Seq/Switch: number of child entries.
    pub(crate) const fn pg_b_of(&self, n: usize) -> usize {
        self.pg_b[n] as usize
    }
    /// Switch: component position and entry position of the edge that owns the
    /// selector, and the depth of the selector in its chain.
    pub(crate) const fn sw_pos(&self, n: usize) -> usize {
        self.pg_c[n] as usize
    }
    pub(crate) const fn sw_entry(&self, n: usize) -> usize {
        self.pg_d[n] as usize
    }
    pub(crate) const fn sw_depth(&self, n: usize) -> usize {
        self.pg_e[n] as usize
    }
    pub(crate) const fn ch_node_of(&self, k: usize) -> usize {
        self.ch_node[k] as usize
    }
    pub(crate) const fn ch_key_of(&self, k: usize) -> u32 {
        self.ch_key[k]
    }
    /// Number of program nodes.
    #[allow(dead_code)]
    pub(crate) const fn prog_len(&self) -> usize {
        self.pg_len
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
        fn key_of(_: &Poll) -> u32 { 0 }
    }
    impl crate::Select<Poll> for K1 {
        type Out = Poll;
        const KEY: u32 = 1;
        fn select(x: &Poll) -> Option<&Poll> { Some(x) }
        fn key_of(_: &Poll) -> u32 { 1 }
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
        // The push dispatches on K0/K1 and runs X, Y, Z straight-line in each branch:
        // no guarded run reads whether anything fired, so no flags.
        assert_eq!(plan.n_flags(), 0);
        assert_eq!(plan.flag_idx(at(1)), NO_FLAG);
        // Program: one switch with two branches (K0: X, Z; K1: Y, Z).
        assert_eq!(plan.pg_kind_of(plan.root_prog()), PG_SWITCH);
        assert_eq!(plan.pg_b_of(plan.root_prog()), 2);
        assert_eq!((plan.ch_key_of(plan.pg_a_of(plan.root_prog())), plan.ch_key_of(plan.pg_a_of(plan.root_prog()) + 1)), (0, 1));
        let k0 = plan.ch_node_of(plan.pg_a_of(plan.root_prog()));
        assert_eq!((plan.pg_kind_of(k0), plan.pg_b_of(k0)), (PG_SEQ, 2));
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

#[cfg(test)]
mod handler_trace {
    use super::*;
    use crate::sinks::Anchor;
    use crate::{Collect, Component, ComponentSubscription, Handler, Root, UpdateHandler};

    #[derive(Root)]
    pub struct Poll;
    pub struct K0;
    pub struct K1;
    macro_rules! key {
        ($k:ident, $n:expr) => {
            impl crate::Select<Poll> for $k {
                type Out = Poll;
                const KEY: u32 = $n;
                fn select(x: &Poll) -> Option<&Poll> { Some(x) }
                fn key_of(_: &Poll) -> u32 { $n }
            }
        };
    }
    key!(K0, 0);
    key!(K1, 1);
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
    scomp!(Z, [p: Poll]);

    #[derive(Handler)]
    #[handles(X, Y)]
    pub struct H;
    impl Collect for H {
        type Config = ();
        type Output = u8;
        type Buf = ();
        fn new((): ()) -> Self { H }
        fn begin(&mut self) {}
        fn finish(&mut self, (): (), _: Self::Input<'_>) -> u8 { 0 }
    }
    impl UpdateHandler<X> for H { fn on_update(&mut self, _: &mut (), _: &u8) {} }
    impl UpdateHandler<Y> for H { fn on_update(&mut self, _: &mut (), _: &u8) {} }

    #[derive(Handler)]
    #[handles(X, Y)]
    #[subscribe(z: Z)]
    pub struct G;
    impl Collect for G {
        type Config = ();
        type Output = u8;
        type Buf = ();
        fn new((): ()) -> Self { G }
        fn begin(&mut self) {}
        fn finish(&mut self, (): (), _: Self::Input<'_>) -> u8 { 0 }
    }
    impl UpdateHandler<X> for G { fn on_update(&mut self, _: &mut (), _: &u8) {} }
    impl UpdateHandler<Y> for G { fn on_update(&mut self, _: &mut (), _: &u8) {} }

    /// Inside a switch leaf the fired edges are static: one item per leaf and no tests.
    #[test]
    fn leaf_items_are_static() {
        let plan = <Anchor<Poll, (H,)> as Planned>::PLAN;
        plan.check();
        let root = plan.root_prog();
        assert_eq!(plan.pg_kind_of(root), PG_SWITCH);
        for b in 0..2 {
            let leaf = plan.ch_node_of(plan.pg_a_of(root) + b);
            // Leaf: X (or Y), then the handler's run.
            assert_eq!(plan.pg_kind_of(leaf), PG_SEQ);
            let h = plan.ch_node_of(plan.pg_a_of(leaf) + 1);
            assert_eq!(plan.pg_kind_of(h), PG_HRUN);
            assert!(!plan.hr_guarded(h));
            assert_eq!(plan.hr_count(h), 1);
            assert_eq!(plan.hi_mode_of(plan.hr_start(h)), 0);
        }
    }

    /// A handler that always runs (it reads `Z`) tests both edges at run time,
    /// each by its parent's flag.
    #[test]
    fn an_always_running_handler_tests_its_edges() {
        let plan = <Anchor<Poll, (G,)> as Planned>::PLAN;
        plan.check();
        assert!(plan.n_flags() >= 2);
    }
}
