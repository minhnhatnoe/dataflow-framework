//! Set subscriptions: `#[subscribe(quotes: for S in AllSymbols { Quoter<S> })]`.
//!
//! The field `quotes` of `Input` (and of `Init`) is a [`SetIn`]: the elements' reads (what a
//! plain entry of each would be, `&Output` or a handle component's view) in the shape of the
//! catalog's subscription tree, and a type. Nothing is looked up at run time:
//!
//! * [`get::<S>()`](SetIn::get) finds `S` among the catalog's elements by `TypeId` in constant
//!   evaluation ([`CatFind`](crate::catalog::CatFind)), then descends the tree to that entry by
//!   constants (the same `PosSelect` descent the handler lookup uses), so it is a field offset.
//! * [`fold`](SetIn::fold) and [`for_each`](SetIn::for_each) are unrolled over the tree, in
//!   listing order, calling the closure with each element's read (same-typed elements: the
//!   closure has one type; a set of different types gets an error naming `fold`).
//!
//! There is no `iter()`: the outputs of the elements are not contiguous (storage is in execution
//! order, and an element's upstream sits between it and the next), so an iterator would have to
//! carry a run-time index and dispatch on it.
//!
//! The reads are the same references a list of named entries would hand `update`, so a set costs
//! the same as that list; they are formed in the gather (one pointer per element, constant
//! offsets from the storage base) and disappear when `update` is inlined into the push, which
//! is the case for a set reader (it has one call site).

use core::marker::PhantomData;

use crate::catalog::CatFind;
use crate::component::Sub;
use crate::flat::{EntryPos, PosSelect, PosSpan, Target};
use crate::node::Vertex;
use crate::select::SelectChain;

/// The phase marker of a set in `Input` (what `update` reads).
#[doc(hidden)]
pub struct RunPh;
/// The phase marker of a set in `Init` (what `new` reads).
#[doc(hidden)]
pub struct InitPh;

/// The data of a node of the set's subscription tree in a phase: a nested tuple of the entries'
/// reads. For a phase it is `DepList::Inputs` or `DepList::Inits` of the same node.
#[doc(hidden)]
pub trait Walk<'a, Ph>: 'static {
    type D;
}
impl<'a, E: Vertex, C: SelectChain<E::Stored>, const I: usize, X: 'static> Walk<'a, RunPh> for Sub<E, C, I, X> {
    type D = <C as SelectChain<E::Stored>>::Read<'a, E>;
}
impl<'a, E: Vertex, C: SelectChain<E::Stored>, const I: usize, X: 'static> Walk<'a, InitPh> for Sub<E, C, I, X> {
    type D = <C as SelectChain<E::Stored>>::Init<'a, E>;
}
macro_rules! walk_impl {
    ($($g:ident)*) => {
        impl<'a, Ph, $($g: Walk<'a, Ph>),*> Walk<'a, Ph> for ($($g,)*) {
            type D = ($(<$g as Walk<'a, Ph>>::D,)*);
        }
    };
}
dataflow_macros::tuple_sizes!(walk_impl);

/// The read of the entry at position `W` below this node, which holds it in field `S`.
#[doc(hidden)]
pub trait SetAt<'a, Ph, W: Target, const S: usize>: Walk<'a, Ph> {
    type R;
    fn at<'x>(d: &'x Self::D) -> &'x Self::R
    where
        'a: 'x;
}
impl<'a, E: Vertex, C: SelectChain<E::Stored>, const I: usize, X: 'static, W: Target> SetAt<'a, RunPh, W, 0> for Sub<E, C, I, X> {
    type R = <C as SelectChain<E::Stored>>::Read<'a, E>;
    #[inline]
    fn at<'x>(d: &'x Self::D) -> &'x Self::R
    where
        'a: 'x,
    {
        d
    }
}
impl<'a, E: Vertex, C: SelectChain<E::Stored>, const I: usize, X: 'static, W: Target> SetAt<'a, InitPh, W, 0> for Sub<E, C, I, X> {
    type R = <C as SelectChain<E::Stored>>::Init<'a, E>;
    #[inline]
    fn at<'x>(d: &'x Self::D) -> &'x Self::R
    where
        'a: 'x,
    {
        d
    }
}
macro_rules! set_at_impl {
    (($($g:ident)+) $k:tt $s:ident) => {
        impl<'a, Ph: 'static, W: Target + 'static, $($g: PosSpan + Walk<'a, Ph> + 'static),+> SetAt<'a, Ph, W, $k> for ($($g,)+)
        where
            $s: PosSelect<W> + SetAt<'a, Ph, W, { <$s as PosSelect<W>>::POS }>,
        {
            type R = <$s as SetAt<'a, Ph, W, { <$s as PosSelect<W>>::POS }>>::R;
            #[inline]
            fn at<'x>(d: &'x Self::D) -> &'x Self::R
    where
        'a: 'x,
    {
                <$s as SetAt<'a, Ph, W, { <$s as PosSelect<W>>::POS }>>::at(&d.$k)
            }
        }
    };
}
dataflow_macros::get_tuples!(set_at_impl);

/// The entry position of the element `S` of the set `Cat` whose entries start at `BASE` (`BASE`
/// itself if `S` is not there; then [`Present`] fails).
#[doc(hidden)]
pub struct SetPos<Cat, S, const BASE: usize>(PhantomData<fn() -> (Cat, S)>);
impl<Cat: CatFind<S>, S: 'static, const BASE: usize> Target for SetPos<Cat, S, BASE> {
    const POS: usize = {
        let p = <Cat as CatFind<S>>::P;
        if p == usize::MAX {
            BASE
        } else {
            BASE + p
        }
    };
}
#[doc(hidden)]
pub struct Flag<const B: bool>;
/// `S` is an element of the set's catalog `Cat`.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "`{S}` is not an element of this set",
    label = "`get::<{S}>()` names a type that is not in the set's catalog",
    note = "the set is `{Cat}`; `get` takes one of its elements, the type the catalog lists (`S` of `for S in Cat {{ Node<S> }}`, not the node)"
)]
pub trait Member<Cat, S> {}
impl<Cat, S> Member<Cat, S> for Flag<true> {}
/// `S` is an element of `Cat` (the bound `get` has).
#[doc(hidden)]
pub trait Present {}
impl<Cat: CatFind<S>, S: 'static, const BASE: usize> Present for SetPos<Cat, S, BASE> where Flag<{ <Cat as CatFind<S>>::P != usize::MAX }>: Member<Cat, S> {}

/// The read of the element `S` of the set `Cat` whose entries start at position `BASE`.
#[doc(hidden)]
pub trait Locate<'a, Ph, W: Target>: Walk<'a, Ph> {
    type R;
    fn locate<'x>(d: &'x Self::D) -> &'x Self::R
    where
        'a: 'x;
}
impl<'a, Ph: 'static, W: Target + 'static, T: Walk<'a, Ph> + 'static> Locate<'a, Ph, W> for T
where
    T: PosSelect<W> + SetAt<'a, Ph, W, { <T as PosSelect<W>>::POS }>,
{
    type R = <T as SetAt<'a, Ph, W, { <T as PosSelect<W>>::POS }>>::R;
    #[inline]
    fn locate<'x>(d: &'x Self::D) -> &'x Self::R
    where
        'a: 'x,
    {
        <T as SetAt<'a, Ph, W, { <T as PosSelect<W>>::POS }>>::at(d)
    }
}

/// Fold the entries of a node in listing order, calling `f` with each read, which is `R`.
#[doc(hidden)]
#[diagnostic::on_unimplemented(
    message = "`fold`/`for_each` on a set need every element to be read as the same type (`{R}`)",
    label = "this element is not read as `{R}`",
    note = "elements of different output types can be read with `get::<S>()`; `fold` and `for_each` give the closure one type, so they take a set whose elements all have the same output"
)]
pub trait Fold<'a, Ph, R>: Walk<'a, Ph> {
    fn fold<Acc, F: FnMut(Acc, R) -> Acc>(d: Self::D, acc: Acc, f: &mut F) -> Acc;
}
impl<'a, E: Vertex, C: SelectChain<E::Stored>, const I: usize, X: 'static> Fold<'a, RunPh, <C as SelectChain<E::Stored>>::Read<'a, E>> for Sub<E, C, I, X> {
    #[inline]
    fn fold<Acc, F: FnMut(Acc, <C as SelectChain<E::Stored>>::Read<'a, E>) -> Acc>(d: Self::D, acc: Acc, f: &mut F) -> Acc {
        f(acc, d)
    }
}
impl<'a, E: Vertex, C: SelectChain<E::Stored>, const I: usize, X: 'static> Fold<'a, InitPh, <C as SelectChain<E::Stored>>::Init<'a, E>> for Sub<E, C, I, X> {
    #[inline]
    fn fold<Acc, F: FnMut(Acc, <C as SelectChain<E::Stored>>::Init<'a, E>) -> Acc>(d: Self::D, acc: Acc, f: &mut F) -> Acc {
        f(acc, d)
    }
}
macro_rules! fold_impl {
    ($($g:ident $i:tt)*) => {
        impl<'a, Ph, R, $($g: Fold<'a, Ph, R>),*> Fold<'a, Ph, R> for ($($g,)*) {
            #[inline]
            #[allow(unused_variables, unused_mut)]
            fn fold<Acc, F: FnMut(Acc, R) -> Acc>(d: Self::D, mut acc: Acc, f: &mut F) -> Acc {
                $(acc = <$g as Fold<'a, Ph, R>>::fold(d.$i, acc, f);)*
                acc
            }
        }
    };
}
dataflow_macros::select_tuples!(fold_impl);
impl<'a, Ph, R> Fold<'a, Ph, R> for () {
    #[inline]
    fn fold<Acc, F: FnMut(Acc, R) -> Acc>(_: (), acc: Acc, _: &mut F) -> Acc {
        acc
    }
}

/// The reads of a set's elements: the field of `Input` (and `Init`) that `name: for S in Cat { N<S> }`
/// makes. `Cat` is the catalog, `T` the subscription tree of its elements.
pub struct SetIn<'a, Ph, Cat, T: Walk<'a, Ph>, const BASE: usize> {
    d: <T as Walk<'a, Ph>>::D,
    _m: PhantomData<fn() -> Cat>,
}
impl<'a, Ph, Cat, T: Walk<'a, Ph>, const BASE: usize> Clone for SetIn<'a, Ph, Cat, T, BASE>
where
    <T as Walk<'a, Ph>>::D: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        SetIn { d: self.d.clone(), _m: PhantomData }
    }
}
impl<'a, Ph, Cat, T: Walk<'a, Ph>, const BASE: usize> Copy for SetIn<'a, Ph, Cat, T, BASE> where <T as Walk<'a, Ph>>::D: Copy {}

impl<'a, Ph, Cat, T: Walk<'a, Ph>, const BASE: usize> SetIn<'a, Ph, Cat, T, BASE> {
    #[doc(hidden)]
    #[inline]
    pub fn __new(d: <T as Walk<'a, Ph>>::D) -> Self {
        SetIn { d, _m: PhantomData }
    }

    /// The number of elements.
    pub fn len(&self) -> usize
    where
        Cat: crate::catalog::Catalog,
    {
        <Cat as crate::catalog::Catalog>::LEN
    }

    /// The read of the element `S` (a type the catalog lists): `&Output` of its node, or the view
    /// of a handle component. `S` not in the set is `E0277`, "`S` is not an element of this set".
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    pub fn get<S: 'static>(&self) -> <T as Locate<'a, Ph, SetPos<Cat, S, BASE>>>::R
    where
        Cat: CatFind<S>,
        T: Locate<'a, Ph, SetPos<Cat, S, BASE>>,
        SetPos<Cat, S, BASE>: Present,
        <T as Locate<'a, Ph, SetPos<Cat, S, BASE>>>::R: Copy,
    {
        *<T as Locate<'a, Ph, SetPos<Cat, S, BASE>>>::locate(&self.d)
    }

    /// As [`get`](Self::get), as a reference to the read (for a view that is not `Copy`).
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    pub fn get_ref<S: 'static>(&self) -> &<T as Locate<'a, Ph, SetPos<Cat, S, BASE>>>::R
    where
        Cat: CatFind<S>,
        T: Locate<'a, Ph, SetPos<Cat, S, BASE>>,
        SetPos<Cat, S, BASE>: Present,
    {
        <T as Locate<'a, Ph, SetPos<Cat, S, BASE>>>::locate(&self.d)
    }

    /// Fold over the elements in listing order. The loop is unrolled at compile time, and the
    /// closure is called with each element's read (`&Output` for a node, a handle component's
    /// view) as it is, so every element must be read as the same type.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    pub fn fold<R, Acc, F: FnMut(Acc, R) -> Acc>(self, init: Acc, mut f: F) -> Acc
    where
        T: Fold<'a, Ph, R>,
    {
        <T as Fold<'a, Ph, R>>::fold(self.d, init, &mut f)
    }

    /// Call `f` with each element's read, in listing order (unrolled; see [`fold`](Self::fold)).
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(debug_assertions, inline)]
    pub fn for_each<R, F: FnMut(R)>(self, mut f: F)
    where
        T: Fold<'a, Ph, R>,
    {
        <T as Fold<'a, Ph, R>>::fold(self.d, (), &mut |(), r| f(r))
    }
}
