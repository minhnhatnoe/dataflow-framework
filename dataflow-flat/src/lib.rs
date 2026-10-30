//! Dataflow components wired at compile time, safe runtime. Requires nightly.
//! See the workspace README for the full design and its history.
//!
//! A program is one *root* component, `App`, plus everything it subscribes to,
//! directly or indirectly. Each component declares what it reads with
//! `#[derive(ComponentSubscription)]` and `#[subscribe(name: Type, ...)]` and
//! implements [`Component`] (owned config, one `update` that receives
//! `Self::Input<'_>` with named references). `Pipeline::<App>::build(configs)`
//! discovers the graph, constructs every component once, and
//! [`Pipeline::run`] updates each of them once, in a fixed order, and returns
//! the root's output by value.
//!
//! # How it works
//!
//! Four stages, one module each, read in this order:
//!
//! 1. **Plan** (`plan`, constant evaluation). The depth-first walk from the
//!    root runs once inside constant evaluation (a `const fn` that calls trait
//!    methods, `const_trait_impl`) and records every component with the
//!    parent it was first reached from. That gives, for every edge
//!    `(parent, dependency)`, whether this is the edge that *first* reaches
//!    the dependency (`Edge::FIRST`), and which components each storage
//!    subtree holds (`Plan::holds`).
//! 2. **States tree** (`storage`, types via `generic_const_exprs`).
//!    `Tree<R, C>` holds a component `C` and, for each of its subscriptions,
//!    either the subtree of that dependency (if this edge is the first one
//!    reaching it: `If<{ Edge::FIRST }>` selects `Tree<..>`) or `()`. So every
//!    component is stored exactly once, constructing it is ordinary safe code,
//!    and dropping the pipeline is compiler drop glue.
//! 3. **Fold** (`walk`). `run` walks the states tree structurally. Walking a
//!    subtree runs the dependencies' subtrees first, calls `update` through a
//!    plain `&mut` reborrow of the component field, and returns
//!    `Done { deps, out }`: the outputs by value.
//! 4. **Lookups** (`lookup`). A later component reads the outputs of finished
//!    ones through a *context*, a type-level list of shared references to
//!    finished subtrees living in the enclosing stack frames. An input is
//!    found by following the plan: at each entry a constant says whether the
//!    entry holds the wanted component, and the impl for that constant is
//!    chosen. Every step is a separately monomorphized function and there are
//!    no steps for paths not taken, so after inlining a `run` is straight-line
//!    code, as for a hand-written struct.
//!
//! The rest: `component` (the traits and the dependency list the derive
//! builds), `build` (configs and the two startup passes, the only runtime
//! lookups) and `pipeline`.
//!
//! The price of being `unsafe`-free is that this needs `generic_const_exprs`,
//! which is confined to this crate: user crates need no feature gate.
//!
//! # Guarantees
//!
//! * **Singletons.** Each concrete component type is constructed once,
//!   updated once per run, and dropped once. Generic instantiations are
//!   different types.
//! * **Execution order** (see [`Pipeline`]): depth-first post-order following
//!   subscription listing order.
//! * **No outputs are stored between runs.** Intermediate outputs live only
//!   during `run` and are dropped before it returns; only the root's output
//!   is returned.
//! * `#![forbid(unsafe_code)]`, no `dyn`, no lookups at run time (the `TypeId`
//!   maps are used only in `Pipeline::build`).
//!
//! Checked at compile time: every dependency is a component, `update`'s inputs
//! match the upstream outputs, the graph is acyclic (a cycle is E0275 "overflow
//! evaluating the requirement `(B, ()): Upstream`", naming a component of the
//! cycle), no component lists the same dependency twice, and the graph fits
//! [`MAX_COMPONENTS`].
//! Checked when the pipeline is built: no config is supplied for a component
//! outside the graph.

#![forbid(unsafe_code)]
// The plan's constants are hashed from type names.
#![feature(const_type_name)]
// The plan runs the dependency walk in constant evaluation, which calls trait
// methods from a `const fn`.
#![feature(const_trait_impl)]
// The plan's constants select types (`If<{ Edge::FIRST }>`). Incomplete, but
// confined to this crate: users of `Pipeline` need no feature gate.
#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

// Lets the derive's `::dataflow_flat::...` paths resolve inside this crate too.
extern crate self as dataflow_flat;

mod build;
mod component;
mod lookup;
mod pipeline;
mod plan;
mod storage;
mod walk;

pub use build::{BuildError, Configs};
pub use component::{Component, ComponentSubscription, DepList};
pub use pipeline::Pipeline;
pub use plan::MAX_COMPONENTS;
pub use walk::Root;

// The derive macro shares its name with the trait, in the macro namespace.
pub use dataflow_macros::ComponentSubscription;
