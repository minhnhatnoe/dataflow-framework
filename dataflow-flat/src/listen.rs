//! Listener requests: after the pipeline is constructed, every component
//! declared `#[listener]` is told, once, which keys its subscribers read (the
//! plan collected them; see [`Plan`](crate::plan::Plan)). A third phase over
//! the component positions, like construction and push; for components that
//! are not listeners the step is empty and vanishes.

use std::marker::PhantomData;

use crate::drive::{Phase, StepAt};
use crate::flat::{Loc, NthT};
use crate::gather::Layout;
use crate::node::{Comp, Node};
use crate::store::AtPos;

#[doc(hidden)]
pub struct ListenPh<A>(PhantomData<A>);
#[doc(hidden)]
pub struct ListenCtx<'c, A: Layout> {
    pub(crate) states: &'c mut A::States,
}
impl<A: Layout> Phase for ListenPh<A> {
    type Ctx<'c> = ListenCtx<'c, A>;
}

impl<A: Layout, const P: usize> StepAt<ListenPh<A>, P> for A
where
    A: NthT<P>,
    <A as NthT<P>>::T: Comp,
    A::States: AtPos<<A as NthT<P>>::T, P>,
{
    fn step(ctx: &mut ListenCtx<'_, A>) {
        type C<A, const P: usize> = <A as NthT<P>>::T;
        if <C<A, P> as Node>::LISTENER {
            let keys = A::PLAN.listener_keys(Loc::<A, P>::IDX);
            <C<A, P> as Node>::__subscribe(<A::States as AtPos<C<A, P>, P>>::at_mut(&mut *ctx.states), keys);
        }
    }
}
