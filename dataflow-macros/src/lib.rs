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

/// `leaf_shapes!(cb)`: `cb!([A0 I0 A1 I1 ..]);` for each leaf size 0..=LEAF
/// (an entry is a component and its position constant).
#[proc_macro]
pub fn leaf_shapes(input: TokenStream) -> TokenStream {
    let cb = callback(input);
    let items = (0..=LEAF).map(|n| {
        let rows = (0..n).flat_map(|i| [format_ident!("A{}", i), format_ident!("I{}", i)]);
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

/// `walk_leaves!(cb)`: `cb!(E0 e0 0 O0 T X0; E1 e1 1 O1 X0 X1; ..);` for n in
/// 1..=ARITY: a tuple of walkers, each seeing what the previous ones finished.
/// Leaves and inner nodes of the states tree are both such tuples.
#[proc_macro]
pub fn walk_leaves(input: TokenStream) -> TokenStream {
    let cb = callback(input);
    let mut out = TokenStream2::new();
    for n in 1..=ARITY {
        let rows = (0..n).map(|j| {
            let (e, v, o, x) = (format_ident!("E{}", j), format_ident!("e{}", j), format_ident!("O{}", j), format_ident!("X{}", j));
            let cin = if j == 0 { format_ident!("T") } else { format_ident!("X{}", j - 1) };
            let idx = syn::Index::from(j);
            quote!(#e #v #idx #o #cin #x;)
        });
        out.extend(quote!(#cb!(#(#rows)*);));
    }
    out.into()
}

/// The storage of a leaf: `MaskN<..>` keeps the entries whose edge is the first
/// one. For each leaf size N in 0..=LEAF: the `MaskN` struct, the `StoredN`
/// trait (`stored_trait!`), one `stored_impl!` per mask (2^N of them, 255 in
/// all) and the `dep_storage_leaf!` impl. The macros are defined in `storage.rs`.
#[proc_macro]
pub fn leaf_storage(_: TokenStream) -> TokenStream {
    let mut out = TokenStream2::new();
    for n in 0..=LEAF {
        let a = idents("A", n);
        let ia = idents("IA", n);
        let mask = format_ident!("Mask{}", n);
        let stored = format_ident!("Stored{}", n);
        if n == 0 {
            out.extend(quote!(#[doc(hidden)] pub struct #mask;));
        } else {
            let b = idents("B", n);
            out.extend(quote!(#[doc(hidden)] pub struct #mask<#(const #b: bool),*>;));
        }
        out.extend(quote!(stored_trait!(#stored [#(#a)*]);));
        for bits in 0..(1usize << n) {
            let flags: Vec<TokenStream2> =
                (0..n).map(|i| if bits >> i & 1 == 1 { quote!(true) } else { quote!(false) }).collect();
            let kept: Vec<&Ident> = (0..n).filter(|i| bits >> i & 1 == 1).map(|i| &a[i]).collect();
            let ty = if n == 0 { quote!(#mask) } else { quote!(#mask<#(#flags),*>) };
            out.extend(quote!(stored_impl!(#stored, #ty, [#(#a)*] => [#(#kept)*]);));
        }
        let rows = a.iter().zip(&ia).flat_map(|(x, y)| [x, y]);
        out.extend(quote!(dep_storage_leaf!(#stored, #mask, [#(#rows)*]);));
    }
    out.into()
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
    // The subscriptions as a tree of tuples (see `DepList`): at most `LEAF`
    // entries in a leaf, exactly `ARITY` children in an inner node.
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
