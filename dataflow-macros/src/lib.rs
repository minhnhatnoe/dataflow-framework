//! `#[derive(ComponentSubscription)]` with `#[subscribe(name: Type, ...)]`, and
//! `#[derive(Root)]`.
//!
//! `ComponentSubscription` generates, for `struct MyComponent<A>`:
//! * `struct MyComponentInput<'a, A>` with one `&'a <Type as Node>::Output`
//!   field per subscription, same names, same order;
//! * `struct MyComponentInit<'a, A>` with one `InitOf<'a, Type>` field per
//!   subscription (`&'a Output` for a component, `()` for the root);
//! * `impl ComponentSubscription for MyComponent<A>` with `type Deps` (the
//!   type-level list the plan walks), `type Input<'a>`, `type Init<'a>`, and the
//!   functions that turn the runtime's nested tuples of references into the
//!   named structs;
//! * `impl Node for MyComponent<A>` (a component is a node), when it is a
//!   `Component`.
//!
//! `Root` generates `impl Node` (output = the type itself, no subscriptions,
//! nothing stored) and `impl Root`.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, parse_quote, DeriveInput, GenericParam, Ident, Lifetime, LifetimeParam,
    Token, Type,
};

/// Children per inner node of a subscription tree. The one constant that sets
/// the shape: a node with more than `LEAF` subscriptions is an inner node,
/// exactly a tuple of `ARITY` child subtrees (no entries of its own); a leaf is
/// a tuple of up to `LEAF = ARITY - 1` entries, so a tuple of `ARITY` is always
/// an inner node and anything shorter a leaf. The library's per-shape impls are
/// generated from it by the callback macros below ([`leaf_shapes`] and friends),
/// so changing this constant is the only edit needed.
const ARITY: usize = 8;
/// Entries a leaf holds at most.
const LEAF: usize = ARITY - 1;

fn idents(prefix: &str, n: usize) -> Vec<Ident> {
    (0..n).map(|i| format_ident!("{}{}", prefix, i)).collect()
}

fn callback(input: TokenStream) -> Ident {
    syn::parse(input).expect("expected the name of a macro_rules! macro")
}

/// `leaf_shapes!(cb)`: `cb!([A0 C0 I0 A1 C1 I1 ..]);` for each leaf size 0..=LEAF
/// (an entry is a node, its selector chain and its position constant).
#[proc_macro]
pub fn leaf_shapes(input: TokenStream) -> TokenStream {
    let cb = callback(input);
    let items = (0..=LEAF).map(|n| {
        let rows = (0..n).flat_map(|i| [format_ident!("A{}", i), format_ident!("C{}", i), format_ident!("I{}", i)]);
        quote!(#cb!([#(#rows)*]);)
    });
    quote!(#(#items)*).into()
}

/// `inner_shape!(cb)`: `cb!([K0 .. K{ARITY-1}]);`, the inner node.
#[proc_macro]
pub fn inner_shape(input: TokenStream) -> TokenStream {
    let cb = callback(input);
    let k = idents("K", ARITY);
    quote!(#cb!([#(#k)*]);).into()
}

/// `tuple_sizes!(cb)`: `cb!(A0 .. An-1);` for n in 0..=ARITY.
#[proc_macro]
pub fn tuple_sizes(input: TokenStream) -> TokenStream {
    let cb = callback(input);
    let items = (0..=ARITY).map(|n| {
        let a = idents("A", n);
        quote!(#cb!(#(#a)*);)
    });
    quote!(#(#items)*).into()
}

/// `select_tuples!(cb)`: `cb!(A0 0 A1 1 ..);` for n in 1..=ARITY.
#[proc_macro]
pub fn select_tuples(input: TokenStream) -> TokenStream {
    let cb = callback(input);
    let items = (1..=ARITY).map(|n| {
        let a = idents("A", n);
        let i = (0..n).map(syn::Index::from);
        quote!(#cb!(#(#a #i)*);)
    });
    quote!(#(#items)*).into()
}

/// `get_tuples!(cb)`: `cb!((A0 .. An-1) k Ak);` for n in 1..=ARITY, k in 0..n.
#[proc_macro]
pub fn get_tuples(input: TokenStream) -> TokenStream {
    let cb = callback(input);
    let mut out = TokenStream2::new();
    for n in 1..=ARITY {
        let a = idents("A", n);
        for k in 0..n {
            let ak = &a[k];
            let ki = syn::Index::from(k);
            out.extend(quote!(#cb!((#(#a)*) #ki #ak);));
        }
    }
    out.into()
}

/// `leaf_idx!(cb)`: `cb!(n; 0 1 .. n-1);` for n in 0..=ARITY. The flat storage
/// and the drivers over it use nodes of up to `ARITY` elements, with literal
/// element indices.
#[proc_macro]
pub fn leaf_idx(input: TokenStream) -> TokenStream {
    let cb = callback(input);
    let items = (0..=ARITY).map(|n| {
        let ks = (0..n).map(syn::Index::from);
        let n = syn::Index::from(n);
        quote!(#cb!(#n; #(#ks)*);)
    });
    quote!(#(#items)*).into()
}

/// `inner_idx!(cb)`: `cb!(0 1 .. ARITY-1);`, the children of an inner node of
/// the flat storage, by literal index.
#[proc_macro]
pub fn inner_idx(input: TokenStream) -> TokenStream {
    let cb = callback(input);
    let ks = (0..ARITY).map(syn::Index::from);
    quote!(#cb!(#(#ks)*);).into()
}

/// `sink_tuples!(cb)`: `cb!(A0 0 A1 1 ..);` for n in 1..=LEAF: the tuples
/// accepted as the sinks of a pipeline (as many as fit one leaf of a
/// subscription tree).
#[proc_macro]
pub fn sink_tuples(input: TokenStream) -> TokenStream {
    let cb = callback(input);
    let items = (1..=LEAF).map(|n| {
        let a = idents("A", n);
        let i = (0..n).map(syn::Index::from);
        quote!(#cb!(#(#a #i)*);)
    });
    quote!(#(#items)*).into()
}

/// The shape of a subscription tree: a leaf of 0 to `LEAF` entries (`()` is the
/// empty leaf), or an inner node that is exactly `ARITY` child subtrees. Entries
/// are indices into the list of unique subscriptions, in listing order: leaves
/// from left to right.
enum Shape {
    Leaf(Vec<usize>),
    Inner(Box<[Shape; ARITY]>),
}

/// Items a tree of depth `d` (a leaf has depth 0) holds at most.
fn capacity(d: u32) -> usize {
    LEAF * ARITY.pow(d)
}

impl Shape {
    /// Fill-first and as shallow as possible. Up to `LEAF` items are a leaf.
    /// Otherwise, with `d` the least depth whose capacity (`LEAF * ARITY^d`)
    /// holds the items, the `ARITY` children are filled left to right with
    /// `capacity(d - 1)` items each (the last non-empty one with what is left),
    /// built recursively; children that get nothing are leaves of 0 entries (`()`).
    fn build(items: &[usize]) -> Shape {
        if items.len() <= LEAF {
            return Shape::Leaf(items.to_vec());
        }
        let d = (1..).find(|&d| capacity(d) >= items.len()).unwrap();
        let per_child = capacity(d - 1);
        let mut rest = items;
        let children = std::array::from_fn(|_| {
            let (part, tail) = rest.split_at(rest.len().min(per_child));
            rest = tail;
            Shape::build(part)
        });
        Shape::Inner(Box::new(children))
    }

    /// The tuple type (or the matching pattern), with `leaf(i)` for entry `i`.
    fn render(&self, leaf: &dyn Fn(usize) -> TokenStream2) -> TokenStream2 {
        match self {
            Shape::Leaf(es) => {
                let es = es.iter().map(|&i| leaf(i));
                quote!((#(#es,)*))
            }
            Shape::Inner(children) => {
                let cs = children.iter().map(|c| c.render(leaf));
                quote!((#(#cs,)*))
            }
        }
    }
}

struct Subscription {
    name: Ident,
    ty: Type,
    /// Selectors applied to the node's output, left to right (`=> A, B`).
    chain: Vec<Type>,
}

impl Parse for Subscription {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        let mut chain = Vec::new();
        if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            loop {
                chain.push(selector(input.parse::<Type>()?));
                // `, name: Type` starts the next subscription; any other
                // `, Type` continues the chain.
                if !input.peek(Token![,]) {
                    break;
                }
                let fork = input.fork();
                fork.parse::<Token![,]>()?;
                let next_is_subscription =
                    fork.is_empty() || (fork.peek(Ident) && fork.peek2(Token![:]) && !fork.peek2(Token![::]));
                if next_is_subscription {
                    break;
                }
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Subscription { name, ty, chain })
    }
}

/// A bare `Some` / `None` in a chain means the library's selectors for `Option`
/// (the prelude's names are variants, not types).
fn selector(ty: Type) -> Type {
    if let Type::Path(p) = &ty {
        if p.qself.is_none() && p.path.leading_colon.is_none() && p.path.segments.len() == 1 {
            let seg = &p.path.segments[0];
            if seg.arguments.is_empty() && (seg.ident == "Some" || seg.ident == "None") {
                let id = &seg.ident;
                return parse_quote!(::dataflow_flat::option::#id);
            }
        }
    }
    ty
}

/// The chain as the nested tuple `(S1, (S2, ()))`.
fn chain_ty(chain: &[Type]) -> TokenStream2 {
    let mut t = quote!(());
    for s in chain.iter().rev() {
        t = quote!((#s, #t));
    }
    t
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
    let init_name = format_ident!("{}Init", name);

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
            let chain = chain_ty(&s.chain);
            wc.predicates.push(parse_quote!(#ty: #krate::Node));
            // A plain subscription needs no predicate (and an extra obligation per
            // subscription measurably slows type checking).
            if !s.chain.is_empty() {
                wc.predicates
                    .push(parse_quote!(#chain: #krate::SelectChain<<#ty as #krate::Node>::Output>));
            }
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
    let field_chains: Vec<TokenStream2> = subs.iter().map(|s| chain_ty(&s.chain)).collect();
    // What `update` reads and `new` gets, per subscription.
    let input_types: Vec<TokenStream2> = subs
        .iter()
        .zip(&field_chains)
        .map(|(s, c)| {
            let t = &s.ty;
            if s.chain.is_empty() {
                quote!(<#t as #krate::Node>::Output)
            } else {
                quote!(<#c as #krate::SelectChain<<#t as #krate::Node>::Output>>::Out)
            }
        })
        .collect();
    let init_types: Vec<TokenStream2> = subs
        .iter()
        .zip(&field_chains)
        .map(|(s, c)| {
            let t = &s.ty;
            if s.chain.is_empty() {
                quote!(#krate::InitOf<#lt, #t>)
            } else {
                quote!(<#c as #krate::SelectChain<<#t as #krate::Node>::Output>>::Init<#lt, #t>)
            }
        })
        .collect();

    // Subscriptions with the same type (compared as written) share one
    // dependency: `Deps` lists each type once and both fields get the same
    // reference. (Two spellings of one type, e.g. via an alias, stay two
    // entries, told apart by their position; the plan stores the type once.)
    let mut unique: Vec<(String, &Type, &TokenStream2)> = Vec::new();
    let mut slot_of: Vec<usize> = Vec::new();
    for (t, c) in field_types.iter().zip(&field_chains) {
        let key = format!("{} => {}", quote!(#t), c);
        let i = match unique.iter().position(|(k, _, _)| *k == key) {
            Some(i) => i,
            None => {
                unique.push((key, t, c));
                unique.len() - 1
            }
        };
        slot_of.push(i);
    }
    let vars: Vec<Ident> = (0..unique.len()).map(|i| format_ident!("__dep{}", i)).collect();
    // The subscriptions as a tree of tuples (see `DepList`): at most `LEAF`
    // entries in a leaf, exactly `ARITY` children in an inner node.
    let all: Vec<usize> = (0..unique.len()).collect();
    let tree = Shape::build(&all);
    let types: Vec<&Type> = unique.iter().map(|(_, t, _)| *t).collect();
    let chains: Vec<&TokenStream2> = unique.iter().map(|(_, _, c)| *c).collect();
    // Each entry is tagged with its position in the unique list: two entries
    // naming one type through different spellings stay two entries of the plan.
    let deps_ty = tree.render(&|i| {
        let t = types[i];
        let c = chains[i];
        quote!(#krate::Sub<#t, #c, #i>)
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
    let init_doc = format!("What [`{name}`]'s `new` receives, one entry per subscription.");

    // The structs live in a private module: `pub` in name (required, since
    // they appear in a public trait impl), but not reachable by path.
    // Code gets values of them only as the argument of `update` / `new`.
    let module = format_ident!("__{}_input", name);

    // `Node` for a component. Its bounds are the component's own.
    let (nimpl_generics, nty_generics, _) = generics.split_for_impl();
    let mut node_generics = generics.clone();
    node_generics.make_where_clause().predicates.push(parse_quote!(#name #ty_generics: #krate::Component));
    let (_, _, node_where) = node_generics.split_for_impl();

    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #module {
            #[allow(unused_imports)]
            use super::*;

            #[doc = #doc]
            pub struct #input_name #input_decl_generics #input_where {
                #( pub #field_names: &#lt #input_types, )*
                #[doc(hidden)]
                pub __marker: #marker_ty,
            }

            #[doc = #init_doc]
            pub struct #init_name #input_decl_generics #input_where {
                #( pub #field_names: #init_types, )*
                #[doc(hidden)]
                pub __marker: #marker_ty,
            }
        }

        impl #impl_generics #krate::ComponentSubscription for #name #ty_generics #where_clause {
            type Deps = #deps_ty;
            type Input<'a> = #module::#input_name<'a, #(#ty_args),*>;
            type Init<'a> = #module::#init_name<'a, #(#ty_args),*>;

            #[inline]
            fn input<'a>(raw: <Self::Deps as #krate::DepList>::Inputs<'a>) -> Self::Input<'a> {
                let #deps_pat = raw;
                #module::#input_name { #( #field_names: #field_vars, )* __marker: ::core::marker::PhantomData }
            }

            #[inline]
            fn init<'a>(raw: <Self::Deps as #krate::DepList>::Inits<'a>) -> Self::Init<'a> {
                let #deps_pat = raw;
                #module::#init_name { #( #field_names: #field_vars, )* __marker: ::core::marker::PhantomData }
            }
        }

        impl #nimpl_generics #krate::__private::Sealed for #name #nty_generics #node_where {}

        impl #nimpl_generics #krate::Node for #name #nty_generics #node_where {
            type Output = <#name #nty_generics as #krate::Component>::Output;
            type Subs = <#name #nty_generics as #krate::ComponentSubscription>::Deps;
            type Kind = #krate::CompKind;
            const KIND: u8 = #krate::KIND_COMPONENT;
        }
    })
}

/// `#[derive(Root)]`: the type fed to `Pipeline::push`. It is a node with no
/// subscriptions whose output is the value itself; nothing of it is stored.
#[proc_macro_derive(Root)]
pub fn derive_root(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_root(input).unwrap_or_else(|e| e.to_compile_error()).into()
}

fn expand_root(input: DeriveInput) -> syn::Result<TokenStream2> {
    let krate = quote!(::dataflow_flat);
    let name = &input.ident;
    if let Some(lt) = input.generics.lifetimes().next() {
        return Err(syn::Error::new_spanned(lt, "a root must be 'static; remove lifetime parameters"));
    }
    let mut generics = input.generics.clone();
    let type_params: Vec<Ident> = generics.type_params().map(|t| t.ident.clone()).collect();
    {
        let wc = generics.make_where_clause();
        for p in &type_params {
            wc.predicates.push(parse_quote!(#p: 'static));
        }
    }
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics #krate::__private::Sealed for #name #ty_generics #where_clause {}
        impl #impl_generics #krate::Node for #name #ty_generics #where_clause {
            type Output = Self;
            type Subs = ();
            type Kind = #krate::RootKind;
            const KIND: u8 = #krate::KIND_ROOT;
        }
        impl #impl_generics #krate::Root for #name #ty_generics #where_clause {}
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flatten(s: &Shape, out: &mut Vec<usize>) {
        match s {
            Shape::Leaf(es) => out.extend(es),
            Shape::Inner(cs) => cs.iter().for_each(|c| flatten(c, out)),
        }
    }

    /// Check node sizes; returns the depth (a leaf is 0).
    fn check(s: &Shape) -> u32 {
        match s {
            Shape::Leaf(es) => {
                assert!(es.len() <= LEAF);
                0
            }
            Shape::Inner(cs) => 1 + cs.iter().map(check).max().unwrap(),
        }
    }

    #[test]
    fn capacities() {
        assert_eq!(capacity(0), 7);
        assert_eq!(capacity(1), 56);
        assert_eq!(capacity(2), 448);
        assert_eq!(capacity(3), 3584);
    }

    #[test]
    fn shapes() {
        for n in (0..=600).chain([3584, 3585]) {
            let items: Vec<usize> = (0..n).collect();
            let shape = Shape::build(&items);

            let mut flat = Vec::new();
            flatten(&shape, &mut flat);
            assert_eq!(flat, items, "order, n = {n}");

            let depth = check(&shape);
            let smallest = (0..).find(|&d| capacity(d) >= n).unwrap();
            assert_eq!(depth, smallest, "minimal depth, n = {n}");
        }
    }

    /// Render the shape of `n` items as a string: `(0 1 2) ..`.
    fn show(s: &Shape) -> String {
        match s {
            Shape::Leaf(es) => format!("({})", es.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(" ")),
            Shape::Inner(cs) => format!("[{}]", cs.iter().map(show).collect::<Vec<_>>().join(", ")),
        }
    }

    #[test]
    fn fill_first() {
        let shape = |n: usize| show(&Shape::build(&(0..n).collect::<Vec<_>>()));
        assert_eq!(shape(7), "(0 1 2 3 4 5 6)");
        assert_eq!(shape(8), "[(0 1 2 3 4 5 6), (7), (), (), (), (), (), ()]");
        assert_eq!(shape(12), "[(0 1 2 3 4 5 6), (7 8 9 10 11), (), (), (), (), (), ()]");
        // Every non-final child is full.
        let s = Shape::build(&(0..100).collect::<Vec<_>>());
        let Shape::Inner(cs) = s else { panic!() };
        let sizes: Vec<usize> = cs.iter().map(|c| { let mut v = vec![]; flatten(c, &mut v); v.len() }).collect();
        assert_eq!(sizes, [56, 44, 0, 0, 0, 0, 0, 0]);
    }
}

/// `#[derive(Demux)]` on an enum: a module named after the enum in snake_case
/// with one selector (a zero-sized type implementing `Select<Enum>`) per
/// variant, usable in subscriptions: `#[subscribe(p: Packets => packet::Depth)]`.
/// Variants must be unit variants (the selected part is `()`) or tuple variants
/// with exactly one field (the selected part is that field). The key of a
/// selector is the index of its variant.
#[proc_macro_derive(Demux)]
pub fn derive_demux(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_demux(input).unwrap_or_else(|e| e.to_compile_error()).into()
}

fn snake_case(name: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            let prev_lower = i > 0 && (chars[i - 1].is_lowercase() || chars[i - 1].is_ascii_digit());
            let acronym_end = i > 0 && chars[i - 1].is_uppercase() && chars.get(i + 1).is_some_and(|n| n.is_lowercase());
            if prev_lower || acronym_end {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn expand_demux(input: DeriveInput) -> syn::Result<TokenStream2> {
    let krate = quote!(::dataflow_flat);
    let name = &input.ident;
    let syn::Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(&input.ident, "Demux can only be derived for enums"));
    };
    if !input.generics.params.is_empty() {
        return Err(syn::Error::new_spanned(&input.generics, "Demux does not support generic enums"));
    }
    let vis = &input.vis;
    let module = Ident::new(&snake_case(&name.to_string()), name.span());
    let mut items = Vec::new();
    let mut structs = Vec::new();
    for (key, v) in data.variants.iter().enumerate() {
        let key = key as u32;
        let vname = &v.ident;
        let (out_ty, pat, val) = match &v.fields {
            syn::Fields::Unit => (quote!(()), quote!(#name::#vname), quote!(&())),
            syn::Fields::Unnamed(f) if f.unnamed.len() == 1 => {
                let t = &f.unnamed[0].ty;
                (quote!(#t), quote!(#name::#vname(__m)), quote!(__m))
            }
            other => {
                return Err(syn::Error::new_spanned(
                    other,
                    "Demux supports unit variants and tuple variants with exactly one field",
                ))
            }
        };
        structs.push(quote!(pub struct #vname;));
        items.push(quote! {
            impl #krate::Select<#name> for #module::#vname {
                type Out = #out_ty;
                const KEY: u32 = #key;
                #[inline(always)]
                fn select(__x: &#name) -> ::core::option::Option<&#out_ty> {
                    match __x {
                        #pat => ::core::option::Option::Some(#val),
                        _ => ::core::option::Option::None,
                    }
                }
            }
        });
    }
    // The impls are outside the module so that the variants' field types
    // resolve where the enum is written (a selector may share a name with a type).
    Ok(quote! {
        #vis mod #module {
            #(#structs)*
        }
        #(#items)*
    })
}
