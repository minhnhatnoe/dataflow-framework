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

/// Moves every token of generated code to a fixed location (the macro's own
/// source), keeping hygiene. Spans are hashed with their line numbers by
/// incremental compilation, so generated items located at the user's
/// declaration would turn the queries that read them red when the
/// declaration moves.
fn respan(ts: TokenStream2, loc: Span) -> TokenStream2 {
    use proc_macro2::{Group, TokenTree};
    ts.into_iter()
        .map(|tt| match tt {
            TokenTree::Group(g) => {
                let mut n = Group::new(g.delimiter(), respan(g.stream(), loc));
                n.set_span(loc);
                TokenTree::Group(n)
            }
            mut t => {
                t.set_span(loc);
                t
            }
        })
        .collect()
}

fn finish(r: syn::Result<TokenStream2>, loc: Span) -> TokenStream {
    match r {
        Ok(ts) => respan(ts, loc).into(),
        Err(e) => e.to_compile_error().into(),
    }
}
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
const ARITY: usize = 16;
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

/// `arity!()`: the literal `ARITY`, for the library's `const ARITY`.
#[proc_macro]
pub fn arity(_input: TokenStream) -> TokenStream {
    let n = syn::Index::from(ARITY);
    quote!(#n).into()
}

/// `arity_classes!(cb)`: `cb!(0 1 .. ARITY);`, the leaf sizes of the flat storage.
#[proc_macro]
pub fn arity_classes(input: TokenStream) -> TokenStream {
    let cb = callback(input);
    let ks = (0..=ARITY).map(syn::Index::from);
    quote!(#cb!(#(#ks)*);).into()
}

/// `arity_classes1!(cb)`: `cb!(1 .. ARITY);`, the non-empty leaf sizes.
#[proc_macro]
pub fn arity_classes1(input: TokenStream) -> TokenStream {
    let cb = callback(input);
    let ks = (1..=ARITY).map(syn::Index::from);
    quote!(#cb!(#(#ks)*);).into()
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
    /// Required for `#[subscribe]`; a `#[handles]` entry may be unnamed.
    name: Option<Ident>,
    ty: Type,
    /// Selectors applied to the node's output, left to right (`=> A, B`).
    chain: Vec<Type>,
    /// A handled edge (`#[handles]` of a `Handler`).
    handled: bool,
}

impl Subscription {
    fn name_str(&self) -> String {
        self.name.as_ref().map(|n| n.to_string()).unwrap_or_default()
    }
}

/// The entries of one `#[handles(..)]`: `Type` (a plain edge keyed by its node
/// type) or `name: Type [=> selectors]` (keyed by a marker the derive generates).
fn parse_handles(input: ParseStream) -> syn::Result<Vec<Subscription>> {
    let mut out = Vec::new();
    while !input.is_empty() {
        let named = input.peek(Ident) && input.peek2(Token![:]) && !input.peek2(Token![::]);
        if named {
            let mut s: Subscription = input.parse()?;
            s.handled = true;
            out.push(s);
        } else {
            let ty: Type = input.parse()?;
            out.push(Subscription { name: None, ty, chain: Vec::new(), handled: true });
        }
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
    }
    Ok(out)
}

impl Parse for Subscription {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // `name: Source [=> selectors]` starts with an identifier and a single
        // colon; anything else is a pattern `Sel(Sel(name)): Source`.
        if !(input.peek(Ident) && input.peek2(Token![:]) && !input.peek2(Token![::])) {
            let pat = parse_pat(input)?;
            input.parse::<Token![:]>()?;
            let ty: Type = input.parse()?;
            let (name, chain) = pattern_chain(pat)?;
            return Ok(Subscription { name: Some(name), ty, chain, handled: false });
        }
        let name: Ident = input.parse()?;
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
                    fork.is_empty()
                        || (fork.peek(Ident) && fork.peek2(Token![:]) && !fork.peek2(Token![::]))
                        || {
                            let f = fork.fork();
                            parse_pat(&f).is_ok() && f.peek(Token![:]) && !f.peek(Token![::])
                        };
                if next_is_subscription {
                    break;
                }
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Subscription { name: Some(name), ty, chain, handled: false })
    }
}

/// A subscription pattern: selectors outermost first, the binding innermost.
enum Pat {
    Bind(Ident),
    Variant(syn::Path, Box<Pat>),
}

fn pat_err<T>(at: impl quote::ToTokens, msg: &str) -> syn::Result<T> {
    Err(syn::Error::new_spanned(at, format!("subscription pattern: {msg}")))
}

/// Parses `Sel(Sel(name))`. Only enum-variant style patterns with exactly one
/// field are accepted; everything else is rejected with a message.
fn parse_pat(input: ParseStream) -> syn::Result<Pat> {
    if input.peek(Token![&]) || input.peek(Token![&&]) {
        return pat_err(input.cursor().token_stream(), "references are not supported (a pattern matches the value, which is borrowed already)");
    }
    if input.peek(Token![_]) {
        return pat_err(input.cursor().token_stream(), "a wildcard `_` is not a selector; name a selector at each level and bind the innermost value");
    }
    if input.peek(syn::Lit) || input.peek(Token![-]) {
        return pat_err(input.cursor().token_stream(), "literal patterns are not supported; use a hand-written guard selector `S(x): Source`");
    }
    if input.peek(Token![ref]) || input.peek(Token![mut]) || input.peek(syn::token::Paren) || input.peek(syn::token::Bracket) {
        return pat_err(input.cursor().token_stream(), "only `Selector(Selector(name))` patterns are supported (no `ref`, `mut`, tuples or slices)");
    }
    if input.peek(Token![..]) || input.peek(Token![..=]) {
        return pat_err(input.cursor().token_stream(), "range or rest patterns are not supported");
    }
    let path = syn::Path::parse_mod_style(input)?;
    let pat = if input.peek(syn::token::Paren) {
        let content;
        let paren = syn::parenthesized!(content in input);
        if content.is_empty() {
            return Err(syn::Error::new(paren.span.open(), "subscription pattern: a selector pattern needs exactly one field to bind, found none"));
        }
        let inner = parse_pat(&content)?;
        if content.peek(Token![,]) {
            return Err(syn::Error::new(paren.span.open(), "subscription pattern: multi-field variants are not supported (a selector selects exactly one field)"));
        }
        if !content.is_empty() {
            reject_trailing(&content)?;
        }
        Pat::Variant(path, Box::new(inner))
    } else if input.peek(syn::token::Brace) {
        return pat_err(path, "struct patterns are not supported; only tuple-variant patterns `Selector(name)`");
    } else if input.peek(Token![..]) || input.peek(Token![..=]) || input.peek(Token![...]) {
        return pat_err(path, "range patterns are not supported");
    } else if input.peek(Token![@]) {
        return pat_err(path, "`name @ pattern` is not supported");
    } else if path.leading_colon.is_none() && path.segments.len() == 1 && path.segments[0].arguments.is_empty() {
        Pat::Bind(path.segments[0].ident.clone())
    } else {
        return pat_err(path, "the innermost pattern must be a plain name to bind (a unit variant has nothing to bind: use `name: Source => Selector`)");
    };
    reject_trailing_if_nested(input)?;
    Ok(pat)
}

/// `|` and `if` right after a pattern (or-patterns and guards).
fn reject_trailing_if_nested(input: ParseStream) -> syn::Result<()> {
    if input.peek(Token![|]) {
        return pat_err(input.cursor().token_stream(), "or-patterns are not supported; use a hand-written selector");
    }
    if input.peek(Token![if]) {
        return pat_err(input.cursor().token_stream(), "guards are not supported; use a hand-written guard selector `S(x): Source`");
    }
    Ok(())
}

fn reject_trailing(input: ParseStream) -> syn::Result<()> {
    reject_trailing_if_nested(input)?;
    pat_err(input.cursor().token_stream(), "unexpected tokens in pattern")
}

/// `Sel1(Sel2(name))` as the name and the chain `[Sel1, Sel2]`.
fn pattern_chain(mut pat: Pat) -> syn::Result<(Ident, Vec<Type>)> {
    let mut chain = Vec::new();
    loop {
        match pat {
            Pat::Bind(name) => {
                if chain.is_empty() {
                    return pat_err(name, "a bare name is not a pattern");
                }
                return Ok((name, chain));
            }
            Pat::Variant(path, inner) => {
                chain.push(selector(pattern_selector(path)));
                pat = *inner;
            }
        }
    }
}

/// The selector type a pattern path stands for. One segment is a selector type
/// in scope (`Btc`, a generic `S`, or a prelude `Some`/`None`/`Ok`/`Err`);
/// `[path::]Enum::Variant` (the segment before the variant starts with an
/// uppercase letter) is the marker the `Demux` derive generated,
/// `[path::]enum_snake::Variant`; `Option::Some` / `Result::Ok` are the
/// library's; any other path (`module::Marker`) is a type as written.
fn pattern_selector(path: syn::Path) -> Type {
    let n = path.segments.len();
    if n >= 2 {
        let pen = &path.segments[n - 2].ident;
        let last = &path.segments[n - 1].ident;
        if path.segments[n - 2].arguments.is_empty() {
            if pen == "Option" && (last == "Some" || last == "None") {
                return parse_quote!(::dataflow_flat::option::#last);
            }
            if pen == "Result" && (last == "Ok" || last == "Err") {
                return parse_quote!(::dataflow_flat::result::#last);
            }
            if pen.to_string().starts_with(|c: char| c.is_uppercase()) {
                let mut p = path.clone();
                let m = Ident::new(&snake_case(&pen.to_string()), pen.span());
                p.segments[n - 2].ident = m;
                return Type::Path(syn::TypePath { qself: None, path: p });
            }
        }
    }
    Type::Path(syn::TypePath { qself: None, path })
}

/// A bare `Some` / `None` / `Ok` / `Err` in a chain means the library's selectors
/// for `Option` / `Result` (the prelude's names are variants, not types).
fn selector(ty: Type) -> Type {
    if let Type::Path(p) = &ty {
        if p.qself.is_none() && p.path.leading_colon.is_none() && p.path.segments.len() == 1 {
            let seg = &p.path.segments[0];
            if seg.arguments.is_empty() && (seg.ident == "Some" || seg.ident == "None") {
                let id = &seg.ident;
                return parse_quote!(::dataflow_flat::option::#id);
            }
            if seg.arguments.is_empty() && (seg.ident == "Ok" || seg.ident == "Err") {
                let id = &seg.ident;
                return parse_quote!(::dataflow_flat::result::#id);
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

#[proc_macro_derive(ComponentSubscription, attributes(subscribe, listener))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let loc = input.ident.span();
    finish(expand(input, false), loc)
}

/// `#[derive(Handler)]`: a component defined by `Collect` and `UpdateHandler`
/// impls instead of `Component`. `#[handles(..)]` lists the handled edges,
/// `#[subscribe(..)]` the plain inputs `finish` reads. Generates what
/// `ComponentSubscription` does, plus `impl Component` (its `new` builds the
/// initial output through `begin`, `on_update`, `finish`), and for every
/// named handled edge a marker type `<struct_snake>_edges::<Name>`.
#[proc_macro_derive(Handler, attributes(subscribe, handles))]
pub fn derive_handler(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let loc = input.ident.span();
    finish(expand(input, true), loc)
}

fn camel(name: &str) -> String {
    name.split('_')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut c = s.chars();
            c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
        })
        .collect()
}

fn expand(input: DeriveInput, handler: bool) -> syn::Result<TokenStream2> {
    let krate = quote!(::dataflow_flat);
    let name = &input.ident;
    let input_name = format_ident!("__{}Input", name);
    let init_name = format_ident!("__{}Init", name);

    // Collect every #[subscribe(...)] (several are allowed; they concatenate).
    let mut subs: Vec<Subscription> = Vec::new();
    for attr in input.attrs.iter() {
        if attr.path().is_ident("subscribe") {
            let list = attr.parse_args_with(Punctuated::<Subscription, Token![,]>::parse_terminated)?;
            if handler {
                if let Some(s) = list.iter().find(|s| !s.chain.is_empty()) {
                    return Err(syn::Error::new_spanned(
                        &s.ty,
                        "a handler's plain `#[subscribe]` inputs can't use selectors (`finish` is also called at build); use `#[handles(name: Type => Selector)]` for a selected edge",
                    ));
                }
            }
            subs.extend(list);
        } else if handler && attr.path().is_ident("handles") {
            subs.extend(attr.parse_args_with(parse_handles)?);
        }
    }
    for (i, a) in subs.iter().enumerate() {
        if let Some(an) = &a.name {
            if subs[..i].iter().any(|b| b.name.as_ref() == Some(an)) {
                return Err(syn::Error::new(an.span(), "duplicate subscription name"));
            }
        }
    }

    if let Some(lt) = input.generics.lifetimes().next() {
        return Err(syn::Error::new_spanned(lt, "components must be 'static; remove lifetime parameters"));
    }

    // Handled edges: the marker that keys `UpdateHandler` (the node type for an
    // unnamed edge, a generated type for a named one).
    let edges_mod = format_ident!("{}_edges", snake_case(&name.to_string()));
    let ty_args0: Vec<TokenStream2> = input
        .generics
        .params
        .iter()
        .filter_map(|p| match p {
            GenericParam::Type(t) => {
                let id = &t.ident;
                Some(quote!(#id))
            }
            GenericParam::Const(c) => {
                let id = &c.ident;
                Some(quote!(#id))
            }
            _ => None,
        })
        .collect();
    let marker_of = |s: &Subscription| -> TokenStream2 {
        match &s.name {
            Some(n) => {
                let m = format_ident!("{}", camel(&n.to_string()));
                if ty_args0.is_empty() {
                    quote!(#edges_mod::#m)
                } else {
                    quote!(#edges_mod::#m<#(#ty_args0),*>)
                }
            }
            None => {
                let t = &s.ty;
                quote!(#t)
            }
        }
    };

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
            let chain = if s.handled {
                let c = chain_ty(&s.chain);
                let m = marker_of(s);
                quote!(#krate::Hd<#m, #c>)
            } else {
                chain_ty(&s.chain)
            };
            wc.predicates.push(parse_quote!(#ty: #krate::Node));
            // A plain subscription needs no predicate (and an extra obligation per
            // subscription measurably slows type checking).
            if !s.chain.is_empty() || s.handled {
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

    let field_types: Vec<&Type> = subs.iter().map(|s| &s.ty).collect();
    let field_chains: Vec<TokenStream2> = subs
        .iter()
        .map(|s| {
            let c = chain_ty(&s.chain);
            if s.handled {
                let m = marker_of(s);
                quote!(#krate::Hd<#m, #c>)
            } else {
                c
            }
        })
        .collect();
    // The entries that are fields of the input struct (all but the handled edges).
    let keep: Vec<usize> = (0..subs.len()).filter(|&i| !subs[i].handled).collect();
    let field_names: Vec<&Ident> = keep
        .iter()
        .map(|&i| subs[i].name.as_ref().expect("a plain subscription is named"))
        .collect();
    // What `update` reads and `new` gets, per subscription.
    let input_types: Vec<TokenStream2> = keep
        .iter()
        .map(|&i| {
            let (s, c) = (&subs[i], &field_chains[i]);
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
            if s.chain.is_empty() && !s.handled {
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
    // The marker of a handled unique entry.
    let mut handled_of: Vec<Option<TokenStream2>> = Vec::new();
    for (s, (t, c)) in subs.iter().zip(field_types.iter().zip(&field_chains)) {
        let key = format!("{} => {}", quote!(#t), c);
        let i = match unique.iter().position(|(k, _, _)| *k == key) {
            Some(i) => i,
            None => {
                unique.push((key, t, c));
                handled_of.push(if s.handled { Some(marker_of(s)) } else { None });
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
    let field_vars: Vec<&Ident> = keep.iter().map(|&i| &vars[slot_of[i]]).collect();

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


    // `Node` for a component. Its bounds are the component's own.
    let listener = !handler && input.attrs.iter().any(|a| a.path().is_ident("listener"));
    let mut node_items = if listener {
        quote! {
            const LISTENER: bool = true;
            fn __subscribe(this: &mut Self, keys: &[u32]) {
                <Self as #krate::Listener>::subscribe(this, keys)
            }
        }
    } else {
        quote!()
    };
    if handler {
        node_items = quote!(const HANDLER: bool = true;);
    }
    let (nimpl_generics, nty_generics, _) = generics.split_for_impl();
    let mut node_generics = generics.clone();
    node_generics.make_where_clause().predicates.push(parse_quote!(#name #ty_generics: #krate::Component));
    let (_, _, node_where) = node_generics.split_for_impl();

    let node_impls = quote! {
        impl #nimpl_generics #krate::__private::Sealed for #name #nty_generics #node_where {}

        impl #nimpl_generics #krate::Node for #name #nty_generics #node_where {
            type Output = <#name #nty_generics as #krate::Component>::Output;
            type Subs = <#name #nty_generics as #krate::ComponentSubscription>::Deps;
            type Kind = #krate::CompKind;
            const KIND: u8 = #krate::KIND_COMPONENT;
            #node_items
        }
    };

    if handler {
        // Markers of the named handled edges.
        let mut marker_items = Vec::new();
        let mut marker_defs = Vec::new();
        let decl_params: Vec<TokenStream2> = input
            .generics
            .params
            .iter()
            .filter_map(|p| match p {
                GenericParam::Type(t) => {
                    let id = &t.ident;
                    Some(quote!(#id))
                }
                GenericParam::Const(c) => {
                    let (id, ty) = (&c.ident, &c.ty);
                    Some(quote!(const #id: #ty))
                }
                _ => None,
            })
            .collect();
        for s in subs.iter().filter(|s| s.handled && s.name.is_some()) {
            let m = format_ident!("{}", camel(&s.name_str()));
            let ty = &s.ty;
            let chain = chain_ty(&s.chain);
            let full = marker_of(s);
            marker_defs.push(quote! {
                pub struct #m<#(#decl_params),*>(::core::marker::PhantomData<fn() -> (#(#type_params,)*)>);
            });
            marker_items.push(quote! {
                impl #impl_generics #krate::HandledEdge for #full #where_clause {
                    type Value = <#chain as #krate::SelectChain<<#ty as #krate::Node>::Output>>::Out;
                }
            });
        }
        let markers = if marker_defs.is_empty() {
            quote!()
        } else {
            quote! {
                #[allow(non_snake_case)]
                pub mod #edges_mod {
                    #(#marker_defs)*
                }
                #(#marker_items)*
            }
        };

        // The Component impl: `new` builds the initial output through the handlers.
        let mut comp_generics = generics.clone();
        {
            let wc = comp_generics.make_where_clause();
            wc.predicates.push(parse_quote!(#name #ty_generics: #krate::Collect));
            for (i, m) in handled_of.iter().enumerate() {
                if let Some(m) = m {
                    let (t, c) = (types[i], chains[i]);
                    wc.predicates.push(parse_quote!(#name #ty_generics: #krate::UpdateHandler<#m>));
                    wc.predicates.push(parse_quote!(#c: #krate::HdChain<<#t as #krate::Node>::Output>));
                }
            }
        }
        let (_, _, comp_where) = comp_generics.split_for_impl();
        let feeds = handled_of.iter().enumerate().filter_map(|(i, m)| {
            let m = m.as_ref()?;
            let v = &vars[i];
            Some(quote!(#krate::FeedInit::<#name #ty_generics, #m>::feed(#v, &mut __this, &mut __buf);))
        });
        let plain_inits = keep.iter().map(|&i| {
            let f = subs[i].name.as_ref().unwrap();
            let t = &subs[i].ty;
            let v = &vars[slot_of[i]];
            quote!(#f: #krate::InitRef::<'_, <#t as #krate::Node>::Output>::out(#v))
        });
        return Ok(quote! {
            #[doc = #doc]
            #[doc(hidden)]
            #[allow(non_camel_case_types)]
            pub struct #input_name #input_decl_generics #input_where {
                #( pub #field_names: &#lt #input_types, )*
                #[doc(hidden)]
                pub __marker: #marker_ty,
            }

            #markers

            impl #impl_generics #krate::ComponentSubscription for #name #ty_generics #where_clause {
                type Deps = #deps_ty;
                type Input<'a> = #input_name<'a, #(#ty_args),*>;
                type Init<'a> = <#deps_ty as #krate::DepList>::Inits<'a>;

                #[inline]
                #[allow(unused_variables)]
                fn input<'a>(raw: <Self::Deps as #krate::DepList>::Inputs<'a>) -> Self::Input<'a> {
                    let #deps_pat = raw;
                    #input_name { #( #field_names: #field_vars, )* __marker: ::core::marker::PhantomData }
                }

                #[inline]
                fn init<'a>(raw: <Self::Deps as #krate::DepList>::Inits<'a>) -> Self::Init<'a> {
                    raw
                }
            }

            impl #impl_generics #krate::Component for #name #ty_generics #comp_where {
                type Config = <#name #ty_generics as #krate::Collect>::Config;
                type Output = <#name #ty_generics as #krate::Collect>::Output;

                #[allow(unused_variables)]
                fn new(config: Self::Config, init: <Self as #krate::ComponentSubscription>::Init<'_>) -> (Self, Self::Output) {
                    let #deps_pat = init;
                    let mut __this = <#name #ty_generics as #krate::Collect>::new(config);
                    let mut __buf = <#name #ty_generics as #krate::Collect>::begin(&mut __this);
                    #(#feeds)*
                    let __input = #input_name { #(#plain_inits,)* __marker: ::core::marker::PhantomData };
                    let __out = <#name #ty_generics as #krate::Collect>::finish(&mut __this, __buf, __input);
                    (__this, __out)
                }

                fn update(&mut self, _: <Self as #krate::ComponentSubscription>::Input<'_>) -> Self::Output {
                    unreachable!("a handler runs through begin / on_update / finish, never `update`")
                }
            }

            #node_impls
        });
    }

    Ok(quote! {
        #[doc = #doc]
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub struct #input_name #input_decl_generics #input_where {
            #( pub #field_names: &#lt #input_types, )*
            #[doc(hidden)]
            pub __marker: #marker_ty,
        }

        #[doc = #init_doc]
        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        pub struct #init_name #input_decl_generics #input_where {
            #( pub #field_names: #init_types, )*
            #[doc(hidden)]
            pub __marker: #marker_ty,
        }

        impl #impl_generics #krate::ComponentSubscription for #name #ty_generics #where_clause {
            type Deps = #deps_ty;
            type Input<'a> = #input_name<'a, #(#ty_args),*>;
            type Init<'a> = #init_name<'a, #(#ty_args),*>;

            #[inline]
            fn input<'a>(raw: <Self::Deps as #krate::DepList>::Inputs<'a>) -> Self::Input<'a> {
                let #deps_pat = raw;
                #input_name { #( #field_names: #field_vars, )* __marker: ::core::marker::PhantomData }
            }

            #[inline]
            fn init<'a>(raw: <Self::Deps as #krate::DepList>::Inits<'a>) -> Self::Init<'a> {
                let #deps_pat = raw;
                #init_name { #( #field_names: #field_vars, )* __marker: ::core::marker::PhantomData }
            }
        }

        #node_impls
    })
}

/// `#[derive(Root)]`: the type fed to `Pipeline::push`. It is a node with no
/// subscriptions whose output is the value itself; nothing of it is stored.
#[proc_macro_derive(Root)]
pub fn derive_root(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let loc = input.ident.span();
    finish(expand_root(input), loc)
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

    fn sub(src: &str) -> syn::Result<Subscription> {
        syn::parse_str::<Subscription>(src)
    }
    fn chain_of(src: &str) -> (String, Vec<String>) {
        let s = sub(src).unwrap();
        (
            s.name.unwrap().to_string(),
            s.chain.iter().map(|t| quote!(#t).to_string().replace(' ', "")).collect(),
        )
    }

    #[test]
    fn pattern_maps_outermost_first() {
        let (n, c) = chain_of("Some(Btc(t)): Ws");
        assert_eq!(n, "t");
        assert_eq!(c, ["::dataflow_flat::option::Some", "Btc"]);
        // the same chain as the `=>` form
        assert_eq!(chain_of("t: Ws => Some, Btc"), (n, c));
    }

    #[test]
    fn pattern_variant_paths() {
        assert_eq!(chain_of("Some(Packet::Depth(d)): Nic").1, ["::dataflow_flat::option::Some", "packet::Depth"]);
        assert_eq!(chain_of("crate::foo::Packet::X(d): Nic").1, ["crate::foo::packet::X"]);
        assert_eq!(chain_of("Packet::BookUpdate(d): Nic").1, ["packet::BookUpdate"]);
        assert_eq!(chain_of("markets::Btc(d): Nic").1, ["markets::Btc"]);
        assert_eq!(chain_of("Ok(Err(e)): R").1, ["::dataflow_flat::result::Ok", "::dataflow_flat::result::Err"]);
        assert_eq!(chain_of("Option::Some(x): R").1, ["::dataflow_flat::option::Some"]);
        assert_eq!(chain_of("S(t): <X as Exchange>::Ws").1, ["S"]);
    }

    #[test]
    fn old_form_still_parses_in_lists() {
        let p = syn::punctuated::Punctuated::<Subscription, Token![,]>::parse_terminated;
        let l = syn::parse::Parser::parse_str(p, "a: A => Some, B, Some(B(c)): C, d: D").unwrap();
        let names: Vec<_> = l.iter().map(|s| s.name.as_ref().unwrap().to_string()).collect();
        assert_eq!(names, ["a", "c", "d"]);
        assert_eq!(l[0].chain.len(), 2);
    }

    fn msg(src: &str) -> String {
        sub(src).err().expect("must be rejected").to_string()
    }

    #[test]
    fn rejected_forms() {
        assert!(msg("Foo { x }: Ws").contains("struct patterns"));
        assert!(msg("Foo(a, b): Ws").contains("multi-field"));
        assert!(msg("Foo(1..=3): Ws").contains("literal"));
        assert!(msg("Foo(a..=b): Ws").contains("range"));
        assert!(msg("Foo(x) if x > 1: Ws").contains("guards"));
        assert!(msg("Foo(x) | Bar(x): Ws").contains("or-patterns"));
        assert!(msg("Foo(3): Ws").contains("literal"));
        assert!(msg("Foo(_): Ws").contains("wildcard"));
        assert!(msg("_(x): Ws").contains("wildcard"));
        assert!(msg("Foo(&x): Ws").contains("references"));
        assert!(msg("Foo(Bar::Unit): Ws").contains("plain name"));
        assert!(msg("Foo(): Ws").contains("exactly one field"));
    }

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
        assert_eq!(capacity(0), LEAF);
        assert_eq!(capacity(1), LEAF * ARITY);
        assert_eq!(capacity(2), LEAF * ARITY * ARITY);
        assert_eq!(capacity(3), LEAF * ARITY * ARITY * ARITY);
    }

    #[test]
    fn shapes() {
        for n in (0..=600).chain([capacity(3), capacity(3) + 1]) {
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
        let rest = vec!["()"; ARITY - 2].join(", ");
        let leaf = |a: usize, b: usize| format!("({})", (a..b).map(|e| e.to_string()).collect::<Vec<_>>().join(" "));
        assert_eq!(shape(LEAF), leaf(0, LEAF));
        assert_eq!(shape(ARITY), format!("[{}, {}, {}]", leaf(0, LEAF), leaf(LEAF, ARITY), rest));
        assert_eq!(shape(ARITY + 4), format!("[{}, {}, {}]", leaf(0, LEAF), leaf(LEAF, ARITY + 4), rest));
        // Every non-final child is full.
        let n = LEAF * ARITY + 10;
        let s = Shape::build(&(0..n).collect::<Vec<_>>());
        let Shape::Inner(cs) = s else { panic!() };
        let sizes: Vec<usize> = cs.iter().map(|c| { let mut v = vec![]; flatten(c, &mut v); v.len() }).collect();
        let mut want = vec![0; ARITY];
        want[0] = LEAF * ARITY;
        want[1] = 10;
        assert_eq!(sizes, want);
    }
}

/// `#[derive(Demux)]` on an enum: a module named after the enum in snake_case
/// with one selector (a zero-sized type implementing `Select<Enum>`) per
/// variant, usable in subscriptions: `#[subscribe(p: Packets => packet::Depth)]`.
/// Variants must be unit variants (the selected part is `()`) or tuple variants
/// with exactly one field (the selected part is that field). The key of a
/// selector is the index of its variant.
#[proc_macro_derive(Demux, attributes(demux))]
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
    // `#[demux(markers = path)]`: use the existing types `path::Variant` as the selectors.
    let mut markers: Option<syn::Path> = None;
    for attr in input.attrs.iter().filter(|a| a.path().is_ident("demux")) {
        attr.parse_nested_meta(|m| {
            if m.path.is_ident("markers") {
                markers = Some(m.value()?.parse::<syn::Path>()?);
                Ok(())
            } else {
                Err(m.error("unknown demux option; expected `markers = path`"))
            }
        })?;
    }
    // The path of `markers` as seen from inside the generated module.
    let reexport_base = match &markers {
        Some(m) => {
            let first = m.segments.first().map(|s| s.ident.to_string()).unwrap_or_default();
            if m.leading_colon.is_some() || first == "crate" {
                quote!(#m)
            } else if first == "self" {
                let rest = m.segments.iter().skip(1);
                quote!(super #(:: #rest)*)
            } else {
                quote!(super::#m)
            }
        }
        None => quote!(),
    };
    let mut reexports = Vec::new();
    let mut items = Vec::new();
    let mut structs = Vec::new();
    // `key_of`: the variant index of a value, for every selector of the enum.
    let key_arms: Vec<TokenStream2> = data
        .variants
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let vname = &v.ident;
            let i = i as u32;
            match &v.fields {
                syn::Fields::Unit => quote!(#name::#vname => #i),
                _ => quote!(#name::#vname(..) => #i),
            }
        })
        .collect();
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
        let marker = match &markers {
            Some(m) => quote!(#m::#vname),
            None => quote!(#module::#vname),
        };
        structs.push(quote!(pub struct #vname;));
        reexports.push(quote!(#vis use #reexport_base::#vname;));
        items.push(quote! {
            impl #krate::Select<#name> for #marker {
                type Out = #out_ty;
                const KEY: u32 = #key;
                #[inline(always)]
                fn select(__x: &#name) -> ::core::option::Option<&#out_ty> {
                    match __x {
                        #pat => ::core::option::Option::Some(#val),
                        _ => ::core::option::Option::None,
                    }
                }
                #[inline(always)]
                fn key_of(__x: &#name) -> u32 {
                    #name::__demux_key(__x)
                }
            }
        });
    }
    // The impls are outside the module so that the variants' field types
    // resolve where the enum is written (a selector may share a name with a type).
    // With `markers`, the module only re-exports them, so `Enum::Variant` in a
    // pattern names the same (shared) type.
    let module_body = if markers.is_some() { quote!(#(#reexports)*) } else { quote!(#(#structs)*) };
    Ok(quote! {
        #vis mod #module {
            #module_body
        }
        impl #name {
            #[doc(hidden)]
            #[inline(always)]
            fn __demux_key(__x: &#name) -> u32 {
                match __x {
                    #(#key_arms,)*
                }
            }
        }
        #(#items)*
    })
}
