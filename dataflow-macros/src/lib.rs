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

/// The shape of a subscription tree: `()`, a leaf of 1 to 4 entries, or an
/// inner node of 4 entries and 4 children. Entries are indices into the list of
/// unique subscriptions, in listing order: a node's entries, then its children
/// from left to right.
enum Shape {
    Empty,
    Leaf(Vec<usize>),
    Inner(Vec<usize>, Vec<Shape>),
}

impl Shape {
    /// Capacity of a tree of depth `d` (a leaf has depth 0).
    fn capacity(d: u32) -> usize {
        match d {
            0 => 4,
            _ => 4 + 4 * Self::capacity(d - 1),
        }
    }

    /// Fill nodes left to right in the shallowest tree that holds everything.
    fn build(items: &[usize]) -> Shape {
        if items.is_empty() {
            return Shape::Empty;
        }
        if items.len() <= 4 {
            return Shape::Leaf(items.to_vec());
        }
        let mut depth = 1;
        while Self::capacity(depth) < items.len() {
            depth += 1;
        }
        let child_cap = Self::capacity(depth - 1);
        let mut rest = &items[4..];
        let mut children = Vec::new();
        for _ in 0..4 {
            let n = rest.len().min(child_cap);
            children.push(Shape::build(&rest[..n]));
            rest = &rest[n..];
        }
        assert!(rest.is_empty());
        Shape::Inner(items[..4].to_vec(), children)
    }

    /// The tuple type (or the matching pattern), with `leaf(i)` for entry `i`.
    fn render(&self, leaf: &dyn Fn(usize) -> TokenStream2) -> TokenStream2 {
        match self {
            Shape::Empty => quote!(()),
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
    // reference. (Two spellings of one type, e.g. via an alias, are still
    // caught by the plan as a duplicate dependency.)
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
    // The subscriptions as a tree of tuples (see `DepList`): at most 4 entries
    // in a leaf, 4 entries and 4 children in an inner node.
    let all: Vec<usize> = (0..unique.len()).collect();
    let tree = Shape::build(&all);
    let types: Vec<&Type> = unique.iter().map(|(_, t)| *t).collect();
    let deps_ty = tree.render(&|i| {
        let t = types[i];
        quote!(#t)
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

    fn old_capacity(d: u32) -> usize {
        if d <= 1 { 4 } else { 4 + 4 * old_capacity(d - 1) }
    }

    fn old_build(items: &[usize]) -> Shape {
        if items.is_empty() {
            return Shape::Empty;
        }
        if items.len() <= 4 {
            return Shape::Leaf(items.to_vec());
        }
        let mut depth = 2;
        while old_capacity(depth) < items.len() {
            depth += 1;
        }
        let child_cap = old_capacity(depth - 1);
        let mut rest = &items[4..];
        let mut children = Vec::new();
        for _ in 0..4 {
            let n = rest.len().min(child_cap);
            children.push(old_build(&rest[..n]));
            rest = &rest[n..];
        }
        assert!(rest.is_empty());
        Shape::Inner(items[..4].to_vec(), children)
    }

    fn flatten(s: &Shape, out: &mut Vec<usize>) {
        match s {
            Shape::Empty => {}
            Shape::Leaf(es) => out.extend(es),
            Shape::Inner(es, cs) => {
                out.extend(es);
                for c in cs {
                    flatten(c, out);
                }
            }
        }
    }

    /// Check node sizes; returns the depth (empty and leaf are 0).
    fn check(s: &Shape) -> u32 {
        match s {
            Shape::Empty => 0,
            Shape::Leaf(es) => {
                assert!((1..=4).contains(&es.len()));
                0
            }
            Shape::Inner(es, cs) => {
                assert_eq!(es.len(), 4);
                assert_eq!(cs.len(), 4);
                1 + cs.iter().map(check).max().unwrap()
            }
        }
    }

    #[test]
    fn capacities() {
        assert_eq!(Shape::capacity(0), 4);
        assert_eq!(Shape::capacity(1), 20);
        assert_eq!(Shape::capacity(2), 84);
    }

    #[test]
    fn shapes() {
        for n in (0..=100).chain([84, 85, 340, 341]) {
            let items: Vec<usize> = (0..n).collect();
            let shape = Shape::build(&items);

            let mut flat = Vec::new();
            flatten(&shape, &mut flat);
            assert_eq!(flat, items, "n = {n}");

            let depth = check(&shape);
            if n > 0 {
                let smallest = (0..).find(|&d| Shape::capacity(d) >= n).unwrap();
                assert_eq!(depth, smallest, "n = {n}");
            }

            let leaf = |i: usize| quote!(#i);
            assert_eq!(
                shape.render(&leaf).to_string(),
                old_build(&items).render(&leaf).to_string(),
                "n = {n}"
            );
        }
    }
}
