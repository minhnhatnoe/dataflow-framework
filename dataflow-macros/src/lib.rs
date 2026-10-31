//! `#[derive(ComponentSubscription)]` with `#[subscribe(name: Type, ...)]`.
//!
//! Generates, for `struct MyComponent<A>`:
//! * `struct MyComponentInput<'a, A>` with one `&'a <Type as Component>::Output`
//!   field per subscription, same names, same order;
//! * `impl ComponentSubscription for MyComponent<A>` with
//!   `type Deps` (the type-level list the runtime walks),
//!   `type Input<'a> = MyComponentInput<'a, A>`, and `fn input`, which turns the
//!   runtime's nested tuple of references into the named struct.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, parse_quote, DeriveInput, GenericParam, Ident, Lifetime, LifetimeParam,
    Token, Type,
};

/// Entries per node, and children per inner node, of a subscription tree.
///
/// The one definition: `dataflow_flat::ARITY` is `arity!()`, which expands to
/// this. The library's per-node-shape impls (`macro_rules!`-generated for
/// leaves of 0 to 4 entries and inner nodes of 4 entries and 4 children)
/// cannot follow it; they assert `ARITY == 4` so a change stops the build.
const ARITY: usize = 4;

/// Expands to [`ARITY`] as a `usize` literal, so `dataflow-flat` shares the
/// constant instead of repeating it.
#[proc_macro]
pub fn arity(_: TokenStream) -> TokenStream {
    let n = ARITY;
    quote!(#n).into()
}

/// The shape of a subscription tree: a leaf of 0 to `ARITY` entries (`()` is
/// the empty leaf), or an inner node of `ARITY` entries and `ARITY` children. Entries are indices into
/// the list of unique subscriptions, in listing order: a node's entries, then
/// its children from left to right.
enum Shape {
    Leaf(Vec<usize>),
    Inner([usize; ARITY], Box<[Shape; ARITY]>),
}

impl Shape {
    /// Balanced and as shallow as possible. Up to `ARITY` items are a leaf.
    /// Otherwise the first `ARITY` items are the node's entries and the rest is
    /// split into `ARITY` contiguous parts of near-equal size (the first
    /// `r` parts one longer, with `q, r = (n - ARITY).divmod(ARITY)`), built
    /// recursively; empty parts are leaves of 0 entries (`()`). The depth is minimal: a tree of
    /// depth `d` holds `n` items iff `n - ARITY <= ARITY * capacity(d - 1)`,
    /// which is the condition for the largest part, `ceil((n - ARITY) / ARITY)`,
    /// to fit at depth `d - 1`.
    fn build(items: &[usize]) -> Shape {
        if items.len() <= ARITY {
            return Shape::Leaf(items.to_vec());
        }
        let (entries, mut rest) = items.split_at(ARITY);
        let (q, r) = (rest.len() / ARITY, rest.len() % ARITY);
        let children = std::array::from_fn(|k| {
            let n = if k < r { q + 1 } else { q };
            let (part, tail) = rest.split_at(n);
            rest = tail;
            Shape::build(part)
        });
        Shape::Inner(entries.try_into().unwrap(), Box::new(children))
    }

    /// The tuple type (or the matching pattern), with `leaf(i)` for entry `i`.
    fn render(&self, leaf: &dyn Fn(usize) -> TokenStream2) -> TokenStream2 {
        match self {
            Shape::Leaf(es) => {
                let es = es.iter().map(|&i| leaf(i));
                quote!((#(#es,)*))
            }
            Shape::Inner(es, children) => {
                let es = es.iter().map(|&i| leaf(i));
                let cs = children.iter().map(|c| c.render(leaf));
                quote!((#(#es,)* #(#cs,)*))
            }
        }
    }
}

struct Subscription {
    name: Ident,
    ty: Type,
}

impl Parse for Subscription {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        Ok(Subscription { name, ty })
    }
}

#[proc_macro_derive(ComponentSubscription, attributes(subscribe))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand(input).unwrap_or_else(|e| e.to_compile_error()).into()
}

fn expand(input: DeriveInput) -> syn::Result<TokenStream2> {
    let krate = quote!(::dataflow_flat);
    let name = &input.ident;
    let input_name = format_ident!("{}Input", name);

    // Collect every #[subscribe(...)] (several are allowed; they concatenate).
    let mut subs: Vec<Subscription> = Vec::new();
    for attr in input.attrs.iter().filter(|a| a.path().is_ident("subscribe")) {
        let list = attr.parse_args_with(Punctuated::<Subscription, Token![,]>::parse_terminated)?;
        subs.extend(list);
    }
    for (i, a) in subs.iter().enumerate() {
        if subs[..i].iter().any(|b| b.name == a.name) {
            return Err(syn::Error::new(a.name.span(), "duplicate subscription name"));
        }
    }

    if let Some(lt) = input.generics.lifetimes().next() {
        return Err(syn::Error::new_spanned(lt, "components must be 'static; remove lifetime parameters"));
    }

    // Bounds shared by the impl and the input struct: every type parameter is
    // 'static, and every subscribed type is a component.
    let mut generics = input.generics.clone();
    let type_params: Vec<Ident> = generics.type_params().map(|t| t.ident.clone()).collect();
    {
        let wc = generics.make_where_clause();
        for p in &type_params {
            wc.predicates.push(parse_quote!(#p: 'static));
        }
        for s in &subs {
            let ty = &s.ty;
            wc.predicates.push(parse_quote!(#ty: #krate::Component));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // The input struct: the component's generics plus a lifetime 'a.
    let lt = Lifetime::new("'__input", Span::call_site());
    let mut input_generics = generics.clone();
    input_generics
        .params
        .insert(0, GenericParam::Lifetime(LifetimeParam::new(lt.clone())));
    // Declaration form keeps inline bounds: `<'__input, U: Bound>`.
    let (input_decl_generics, _, input_where) = input_generics.split_for_impl();

    let field_names: Vec<&Ident> = subs.iter().map(|s| &s.name).collect();
    let field_types: Vec<&Type> = subs.iter().map(|s| &s.ty).collect();

    // Subscriptions with the same type (compared as written) share one
    // dependency: `Deps` lists each type once and both fields get the same
    // reference. (Two spellings of one type, e.g. via an alias, stay two
    // entries, told apart by their position; the plan stores the type once.)
    let mut unique: Vec<(String, &Type)> = Vec::new();
    let mut slot_of: Vec<usize> = Vec::new();
    for t in &field_types {
        let key = quote!(#t).to_string();
        let i = match unique.iter().position(|(k, _)| *k == key) {
            Some(i) => i,
            None => {
                unique.push((key, t));
                unique.len() - 1
            }
        };
        slot_of.push(i);
    }
    let vars: Vec<Ident> = (0..unique.len()).map(|i| format_ident!("__dep{}", i)).collect();
    // The subscriptions as a tree of tuples (see `DepList`): at most `ARITY`
    // entries in a leaf, `ARITY` entries and `ARITY` children in an inner node.
    let all: Vec<usize> = (0..unique.len()).collect();
    let tree = Shape::build(&all);
    let types: Vec<&Type> = unique.iter().map(|(_, t)| *t).collect();
    // Each entry is tagged with its position in the unique list: two entries
    // naming one type through different spellings stay two entries of the plan.
    let deps_ty = tree.render(&|i| {
        let t = types[i];
        quote!(#krate::Sub<#t, #i>)
    });
    let deps_pat = tree.render(&|i| {
        let v = &vars[i];
        quote!(#v)
    });
    let field_vars: Vec<&Ident> = slot_of.iter().map(|&i| &vars[i]).collect();

    // Type parameters may not appear in any field; a marker keeps them used.
    let marker_ty = quote!(::core::marker::PhantomData<(&#lt (), fn() -> (#(#type_params,)*))>);

    // `MyComponentInput<'a, A>` as used inside the impl.
    let ty_args: Vec<TokenStream2> = generics
        .params
        .iter()
        .map(|p| match p {
            GenericParam::Type(t) => {
                let id = &t.ident;
                quote!(#id)
            }
            GenericParam::Const(c) => {
                let id = &c.ident;
                quote!(#id)
            }
            GenericParam::Lifetime(l) => {
                let l = &l.lifetime;
                quote!(#l)
            }
        })
        .collect();

    let doc = format!("Inputs of [`{name}`], one reference per subscription, valid during `update`.");

    // The struct lives in a private module: `pub` in name (required, since
    // it appears in `Sum`'s public trait impl), but not reachable by path.
    // Code gets values of it only as the argument of `update`.
    let module = format_ident!("__{}_input", name);

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #module {
            #[allow(unused_imports)]
            use super::*;

            #[doc = #doc]
            pub struct #input_name #input_decl_generics #input_where {
                #( pub #field_names: &#lt <#field_types as #krate::Component>::Output, )*
                #[doc(hidden)]
                pub __marker: #marker_ty,
            }
        }

        impl #impl_generics #krate::ComponentSubscription for #name #ty_generics #where_clause {
            type Deps = #deps_ty;
            type Input<'a> = #module::#input_name<'a, #(#ty_args),*>;

            #[inline]
            fn input<'a>(raw: <Self::Deps as #krate::DepList>::Inputs<'a>) -> Self::Input<'a> {
                let #deps_pat = raw;
                #module::#input_name { #( #field_names: #field_vars, )* __marker: ::core::marker::PhantomData }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Items a tree of depth `d` (a leaf has depth 0) holds at most.
    fn capacity(d: u32) -> usize {
        if d == 0 { ARITY } else { ARITY + ARITY * capacity(d - 1) }
    }

    fn flatten(s: &Shape, out: &mut Vec<usize>) {
        match s {
            Shape::Leaf(es) => out.extend(es),
            Shape::Inner(es, cs) => {
                out.extend(es);
                for c in cs {
                    flatten(c, out);
                }
            }
        }
    }

    fn count(s: &Shape) -> usize {
        let mut v = Vec::new();
        flatten(s, &mut v);
        v.len()
    }

    /// Check node sizes and balance; returns the depth (a leaf is 0).
    fn check(s: &Shape) -> u32 {
        match s {
            Shape::Leaf(es) => {
                assert!((0..=ARITY).contains(&es.len()));
                0
            }
            Shape::Inner(_, cs) => {
                let sizes: Vec<usize> = cs.iter().map(count).collect();
                let (min, max) = (sizes.iter().min().unwrap(), sizes.iter().max().unwrap());
                assert!(max - min <= 1, "unbalanced parts {sizes:?}");
                assert!(sizes.windows(2).all(|w| w[0] >= w[1]), "larger parts first {sizes:?}");
                1 + cs.iter().map(check).max().unwrap()
            }
        }
    }

    #[test]
    fn capacities() {
        assert_eq!(capacity(0), 4);
        assert_eq!(capacity(1), 20);
        assert_eq!(capacity(2), 84);
        assert_eq!(capacity(3), 340);
    }

    #[test]
    fn shapes() {
        for n in (0..=100).chain([84, 85, 340, 341, 512]) {
            let items: Vec<usize> = (0..n).collect();
            let shape = Shape::build(&items);

            let mut flat = Vec::new();
            flatten(&shape, &mut flat);
            assert_eq!(flat, items, "n = {n}");

            let depth = check(&shape);
            if n > 0 {
                let smallest = (0..).find(|&d| capacity(d) >= n).unwrap();
                assert_eq!(depth, smallest, "n = {n}");
            }
        }
    }
}
