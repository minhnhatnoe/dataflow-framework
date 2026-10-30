//! Opt-in dependency cycle diagnostics (cargo feature `cycle-diagnostics`).

use crate::component::ComponentSubscription;

/// A component whose upstream graph has no cycle. Never written by hand: it
/// is implemented for every `ComponentSubscription`, and the *proof* is what
/// matters (a cycle makes the proof depend on itself, which the trait solver
/// rejects).
///
/// By default nothing requires the proof, and a cycle shows up as E0391
/// "cycle detected when simplifying constant ... `DepList>::HEADS`", from
/// the plan's own constants. With the cargo feature `cycle-diagnostics`
/// every [`Component`](crate::Component) must satisfy it, so a cycle is
/// reported where it is written, as "overflow evaluating the requirement
/// `A: NoDependencyCycle`", before any plan constant is evaluated. The price
/// is compile time: the trait solver re-proves the whole upstream graph for
/// every component (measured: no cost at 100 components, +0.4 s at 200,
/// +2 s at 400). (The check cannot be made once per pipeline instead: a bound
/// on `Pipeline`, `Root` or `build` is evaluated after, or concurrently with,
/// the plan's constants, which hit E0391 first, and a bound on the plan's
/// trait works but re-proves the whole graph at every use and is slower
/// still.)
pub trait NoDependencyCycle {}
impl<C: ComponentSubscription> NoDependencyCycle for C where C::Deps: NoDependencyCycles {}

/// A dependency list in which every component is [`NoDependencyCycle`]. A
/// cycle makes the proof depend on itself, which the trait solver rejects.
#[doc(hidden)]
pub trait NoDependencyCycles {}
impl NoDependencyCycles for () {}
impl<H: NoDependencyCycle, T: NoDependencyCycles> NoDependencyCycles for (H, T) {}

/// The supertrait of [`Component`](crate::Component) that switches the cycle
/// check on: see [`NoDependencyCycle`]. Implemented for every type; with the
/// feature `cycle-diagnostics` it additionally requires `NoDependencyCycle`.
#[doc(hidden)]
#[cfg(feature = "cycle-diagnostics")]
pub trait NoDependencyCycleCheck: NoDependencyCycle {}
#[cfg(feature = "cycle-diagnostics")]
impl<C: NoDependencyCycle> NoDependencyCycleCheck for C {}
#[doc(hidden)]
#[cfg(not(feature = "cycle-diagnostics"))]
pub trait NoDependencyCycleCheck {}
#[cfg(not(feature = "cycle-diagnostics"))]
impl<C> NoDependencyCycleCheck for C {}
