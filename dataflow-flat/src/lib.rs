//! Event-driven dataflow, wired at compile time, safe at run time. Requires
//! nightly. **Status: redesign in progress (phase 1a); see the README and the
//! event spec.**
//!
//! A program is a [`Pipeline<R, Sinks>`](Pipeline): one *root* type `R`
//! (`#[derive(Root)]`), fed with `pipeline.push(value)`, and *components*
//! (`#[derive(ComponentSubscription)]` + `#[subscribe(name: Node, ...)]` +
//! `impl Component`). The graph is everything upstream of the sinks, a tuple of
//! components. Each component declares what it reads; `new(config, init)` builds
//! it and its first output, `update(input)` produces its next output. A push
//! updates every component downstream of the root once, in a fixed order, and
//! the outputs are stored in the pipeline between pushes.
//!
//! # How it works
//!
//! 1. **Plan** (`plan`, constant evaluation). The depth-first walk from the
//!    sinks runs once inside constant evaluation (a `const fn` that calls trait
//!    methods, `const_trait_impl`) and records every node with its kind, the
//!    node it was first reached from, its execution position (the post-order)
//!    and the upstream edges. A node's identity is its `TypeId` (`identity`,
//!    `table`). The checks that need only the walk (a second root, a component
//!    not reachable from the root) are const panics there.
//! 2. **Flat storage** (`flat`, `store`, types via `generic_const_exprs`). The
//!    states of the components and their stored outputs are two balanced
//!    tuples (8-ary, so as deep as log8 of the number of components) with one
//!    `Slot` per component in execution order. The type at each position is
//!    found by a halving descent of the first-parent tree (`flat::Down`), so
//!    the solver depth is about log2 of the graph depth. A position is read or
//!    written by a constant descent (`store::Access`): one field offset after
//!    inlining.
//! 3. **Construction** (`drive`, `gather`). The pass visits positions in
//!    execution order; each `new` is given the outputs built so far.
//! 4. **Push** (`drive`, `gather`). The same driver visits the positions again:
//!    for each, if its condition holds (`drive::Cond`, "always" in phase 1a),
//!    gather its inputs as references into the stored outputs and the pushed
//!    root, call `update`, store the output. Every step is a separately
//!    monomorphized `#[inline(always)]` function, so a push is straight-line
//!    code.
//!
//! The price of being `unsafe`-free is that this needs `generic_const_exprs`,
//! which is confined to this crate: user crates need no feature gate.
//!
//! # Guarantees
//!
//! * **Singletons.** Each concrete component type is constructed once, updated
//!   once per push, and dropped once. Generic instantiations are different
//!   types. Subscribing to the same type twice, also through an alias
//!   (`type Y = X;`, `a: X, b: Y`), is one dependency: both fields are the same
//!   reference.
//! * **Execution order** (see [`Pipeline`]): depth-first post-order following
//!   subscription listing order.
//! * **Stored outputs.** A component's output persists between pushes; the
//!   root's value does not.
//! * `#![forbid(unsafe_code)]`, no `dyn`, no lookups at run time (the `TypeId`
//!   map of configs is used only in `Pipeline::build`).
//!
//! Checked at compile time: every subscription is a node, `update`'s inputs
//! match the upstream outputs, the graph is acyclic (a cycle is E0275 "overflow
//! evaluating the requirement `(B,): Upstream`", naming a component of the
//! cycle), the graph fits [`MAX_COMPONENTS`], and every component is reachable
//! from the root and the root is `R` (an evaluation error that names the
//! component). Checked when the pipeline is built: no config is supplied for a
//! component outside the graph.


#![forbid(unsafe_code)]
// The plan's table starts probing at a hash of the component's type name (a
// hint; the match is decided by `TypeId`).
#![feature(const_type_name)]
// The plan runs the dependency walk in constant evaluation, which calls trait
// methods from a `const fn`, and compares components by `TypeId` (`==` in a
// `const fn`).
#![feature(const_trait_impl, const_cmp)]
// A const panic with a component's name (`panic!` can't format in a `const fn`).
#![feature(panic_internals)]
#![allow(internal_features)]
// The plan's constants select types (`Slot<{ LO + 3 }, ..>`, `Access<.., { SelP::POS }>`). Incomplete, but
// confined to this crate: users of `Pipeline` need no feature gate.
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

// Lets the derive's `::dataflow_flat::...` paths resolve inside this crate too.
extern crate self as dataflow_flat;

mod build;
mod component;
mod drive;
mod flat;
mod gather;
mod identity;
mod node;
mod pipeline;
mod plan;
mod select;
mod sinks;
mod store;
mod table;

pub use build::{BuildError, Configs};
pub use component::{Component, ComponentSubscription, DepList, Sub};
pub use node::{InitOf, Node, Root};
pub use pipeline::Pipeline;
pub use plan::{MAX_COMPONENTS, MAX_DECISIONS, MAX_EDGES, MAX_TERMS};
pub use select::{option, Select, GUARD};
#[doc(hidden)]
pub use select::SelectChain;
pub use sinks::{SinkList, Sinks};

// Used by the derives' output.
#[doc(hidden)]
pub use node::{CompKind, RootKind, __private, KIND_COMPONENT, KIND_ROOT};

// The derive macros share their names with the traits, in the macro namespace.
pub use dataflow_macros::{ComponentSubscription, Demux, Root};
