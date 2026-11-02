//! Listener requests as types. A *listener* (`#[listener]` + `impl Listener`)
//! is a component whose output is demuxed by its subscribers (`Some(Btc(t)):
//! BinanceWs`). Before anything is constructed, `Pipeline::build` runs a first
//! pass over the component positions in execution order: for every
//! subscription entry that reads a listener, each *requested* selector of its
//! chain (the selectors `#[derive(Demux)]` generates; `Some` and friends are
//! not) calls `Request<S>::add` on the listener's [`Listener::Requests`] value,
//! once per distinct selector `S` per listener. The value is then handed to the
//! listener's `new` as `init.requests`.
//!
//! * Order of the `add` calls: components in execution order, within a
//!   component its subscriptions in listing order, within a chain left to
//!   right; a selector already added for that listener is skipped. So the
//!   order is that of the first reader.
//! * A selector read downstream of a listener whose `Requests` has no
//!   `Request<S>` impl is a compile error.
//! * Nothing is constructed or run in the first pass: it only needs the types
//!   (the same typed walk as construction, over the component positions), and
//!   the values live in the construction pass, which exists only during
//!   `build`.

use std::any::{Any, TypeId};
use std::marker::PhantomData;

use crate::build::ConstructPass;
use crate::component::{DepList, Sub};
use crate::drive::{Phase, StepAt};
use crate::flat::NthT;
use crate::gather::Layout;
use crate::node::{Comp, Node};
use crate::select::{Select, SelectChain};

/// A component that is told, at build, which selectors its subscribers read.
/// Declared with `#[listener]` on the `ComponentSubscription` derive.
///
/// ```ignore
/// #[derive(Default)]
/// struct Streams(Vec<&'static str>);
/// impl<S: BinanceStream> Request<S> for Streams {
///     fn add(&mut self) { self.0.push(S::STREAM); }
/// }
/// impl Listener for BinanceWs { type Requests = Streams; }
/// // in `new`: init.requests.0
/// ```
pub trait Listener {
    /// Collects the requests. `Default` is the empty collection (what a
    /// listener nobody reads gets).
    type Requests: Default + 'static;
}

/// `Self` can take the request of selector `S` (a marker such as
/// `markets::Btc`, or a generated `packet::Binance`). `add` is called once for
/// each distinct selector read downstream of the listener.
#[diagnostic::on_unimplemented(
    message = "the listener's requests type `{Self}` cannot take the request of the selector `{S}`",
    label = "a subscriber reads `{S}` from this listener, so add `impl Request<{S}> for {Self}`"
)]
pub trait Request<S>: 'static {
    fn add(&mut self);
}

/// The requests of a node that is not a listener: accepts everything, keeps nothing.
#[derive(Default)]
#[doc(hidden)]
pub struct NoRequests;
impl<S> Request<S> for NoRequests {
    #[inline(always)]
    fn add(&mut self) {}
}

#[doc(hidden)]
pub trait ReqIf<const B: bool, S, R> {
    fn add(pass: &mut ConstructPass, listener: TypeId);
}
impl<S: 'static, R: Request<S> + Default + 'static> ReqIf<true, S, R> for () {
    fn add(pass: &mut ConstructPass, listener: TypeId) {
        if pass.seen.insert((listener, TypeId::of::<S>())) {
            let r = pass.requests.entry(listener).or_insert_with(|| Box::new(R::default()) as Box<dyn Any>);
            <R as Request<S>>::add(r.downcast_mut::<R>().expect("keyed by listener"));
        }
    }
}
impl<S, R> ReqIf<false, S, R> for () {
    #[inline(always)]
    fn add(_: &mut ConstructPass, _: TypeId) {}
}

/// One selector's contribution: nothing unless it is a requested one.
#[doc(hidden)]
pub trait SelReq<In: 'static, R: 'static>: Select<In> {
    fn req(pass: &mut ConstructPass, listener: TypeId);
}
impl<In: 'static, R: 'static, S: Select<In>> SelReq<In, R> for S
where
    (): ReqIf<{ <S as Select<In>>::REQUEST }, S, R>,
{
    fn req(pass: &mut ConstructPass, listener: TypeId) {
        <() as ReqIf<{ <S as Select<In>>::REQUEST }, S, R>>::add(pass, listener)
    }
}

/// The requests of a whole chain applied to a value of type `In`.
#[doc(hidden)]
pub trait ChainReq<In: 'static, R: 'static>: SelectChain<In> {
    fn collect(pass: &mut ConstructPass, listener: TypeId);
}
impl<In: 'static, R: 'static> ChainReq<In, R> for () {
    #[inline(always)]
    fn collect(_: &mut ConstructPass, _: TypeId) {}
}
impl<In: 'static, R: 'static, S: SelReq<In, R>, T: ChainReq<S::Out, R>> ChainReq<In, R> for (S, T) {
    fn collect(pass: &mut ConstructPass, listener: TypeId) {
        <S as SelReq<In, R>>::req(pass, listener);
        <T as ChainReq<S::Out, R>>::collect(pass, listener);
    }
}

/// A handled edge requests what its chain does.
impl<In: 'static, R: 'static, Em: 'static, C: ChainReq<In, R>> ChainReq<In, R> for crate::handler::Hd<Em, C> {
    fn collect(pass: &mut ConstructPass, listener: TypeId) {
        <C as ChainReq<In, R>>::collect(pass, listener)
    }
}

/// The requests of one component's subscription tree, in listing order.
#[doc(hidden)]
pub trait ReqList: DepList {
    fn collect(pass: &mut ConstructPass);
}
macro_rules! req_leaf {
    ([$($E:ident $C:ident $I:ident)*]) => {
        impl<$($E: Node, $C: ChainReq<<$E as Node>::Output, <$E as Node>::Requests>, const $I: usize,)*> ReqList for ($(Sub<$E, $C, $I>,)*) {
            #[allow(unused_variables)]
            fn collect(pass: &mut ConstructPass) {
                $(if <$E as Node>::LISTENS {
                    <$C as ChainReq<<$E as Node>::Output, <$E as Node>::Requests>>::collect(pass, TypeId::of::<$E>());
                })*
            }
        }
    };
}
dataflow_macros::leaf_shapes!(req_leaf);
macro_rules! req_inner {
    ([$($K:ident)*]) => {
        impl<$($K: ReqList,)*> ReqList for ($($K,)*) {
            fn collect(pass: &mut ConstructPass) {
                $(<$K as ReqList>::collect(pass);)*
            }
        }
    };
}
dataflow_macros::inner_shape!(req_inner);

#[doc(hidden)]
pub struct ReqPh<A>(PhantomData<A>);
#[doc(hidden)]
pub struct ReqCtx<'c> {
    pub(crate) pass: &'c mut ConstructPass,
}
impl<A: Layout> Phase for ReqPh<A> {
    type Ctx<'c> = ReqCtx<'c>;
}

impl<A: Layout, const P: usize> StepAt<ReqPh<A>, P> for A
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    <<A as NthT<P>>::T as crate::component::ComponentSubscription>::Deps: ReqList,
{
    fn step(ctx: &mut ReqCtx<'_>) {
        <<<A as NthT<P>>::T as crate::component::ComponentSubscription>::Deps as ReqList>::collect(&mut *ctx.pass);
    }
}
