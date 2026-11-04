//! Trading component adapters. Expansion uses the existing static graph machinery.
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    DeriveInput, Ident, ItemStruct, Token, Type,
};

struct Entry {
    name: Ident,
    ty: Type,
    field: Option<Ident>,
}
impl Parse for Entry {
    fn parse(p: ParseStream) -> syn::Result<Self> {
        let name = p.parse()?;
        p.parse::<Token![:]>()?;
        let ty = p.parse()?;
        let field = if p.peek(Token![=>]) {
            p.parse::<Token![=>]>()?;
            Some(p.parse()?)
        } else {
            None
        };
        Ok(Self { name, ty, field })
    }
}
fn entries(item: &mut ItemStruct, name: &str) -> syn::Result<Vec<Entry>> {
    let mut result = Vec::new();
    let mut error = None;
    item.attrs.retain(|a| {
        if !a.path().is_ident(name) {
            return true;
        }
        match a.parse_args_with(Punctuated::<Entry, Token![,]>::parse_terminated) {
            Ok(es) => result.extend(es),
            Err(e) => error = Some(e),
        }
        false
    });
    if let Some(e) = error {
        Err(e)
    } else {
        Ok(result)
    }
}
struct Args {
    exchange: Option<Type>,
    symbol: Option<Type>,
}
impl Parse for Args {
    fn parse(p: ParseStream) -> syn::Result<Self> {
        let mut a = Self {
            exchange: None,
            symbol: None,
        };
        while !p.is_empty() {
            let key: Ident = p.parse()?;
            p.parse::<Token![=]>()?;
            let ty = p.parse()?;
            let slot = match key.to_string().as_str() {
                "exchange" => &mut a.exchange,
                "symbol" => &mut a.symbol,
                _ => return Err(syn::Error::new_spanned(key, "expected exchange or symbol")),
            };
            if slot.replace(ty).is_some() {
                return Err(p.error("duplicate parameter"));
            }
            if !p.is_empty() {
                p.parse::<Token![,]>()?;
            }
        }
        Ok(a)
    }
}
fn expand(kind: &str, args: Args, mut item: ItemStruct) -> syn::Result<proc_macro2::TokenStream> {
    let inputs = entries(&mut item, "inputs")?;
    let sources = entries(&mut item, "source")?;
    let quotes = entries(&mut item, "quotes")?;
    if sources.len() > 1 || (!sources.is_empty() && kind != "book") {
        return Err(syn::Error::new_spanned(
            &item,
            "only Books may declare one root source",
        ));
    }
    if quotes.iter().any(|q| q.field.is_some()) || sources.iter().any(|s| s.field.is_some()) {
        return Err(syn::Error::new_spanned(
            &item,
            "sources and quotes cannot select fields",
        ));
    }
    if kind != "oms" && !quotes.is_empty() {
        return Err(syn::Error::new_spanned(
            &item,
            "quotes are only allowed on an OMS",
        ));
    }
    if kind == "oms" && quotes.is_empty() {
        return Err(syn::Error::new_spanned(
            &item,
            "an OMS must declare its quote set",
        ));
    }
    if (kind == "book" || kind == "signal") && (args.exchange.is_some() || args.symbol.is_some()) {
        return Err(syn::Error::new_spanned(
            &item,
            "identity parameters belong on quote_engine and oms",
        ));
    }
    let mut names = std::collections::HashSet::new();
    for e in inputs.iter().chain(&sources).chain(&quotes) {
        if !names.insert(e.name.to_string()) {
            return Err(syn::Error::new_spanned(&e.name, "duplicate input name"));
        }
    }
    let name = &item.ident;
    let vis = &item.vis;
    let g = &item.generics;
    let (ig, tg, wc) = g.split_for_impl();
    let values = format_ident!("{}Values", name);
    let mut vg = g.clone();
    vg.params.insert(0, syn::parse_quote!('a));
    let (_, vtg, _) = vg.split_for_impl();
    let deps = inputs.iter().chain(&sources).chain(&quotes).map(|e| {
        let n = &e.name;
        let t = &e.ty;
        quote!(#n: #t)
    });
    let fields = inputs.iter().map(|e| {
        let n = &e.name;
        let t = &e.ty;
        if e.field.is_some() {
            quote!(pub #n: &'a f64)
        } else {
            quote!(pub #n: &'a <#t as ::dataflow_trading::ValueNode>::Value)
        }
    });
    let source_fields = sources.iter().map(|e| {
        let n = &e.name;
        let t = &e.ty;
        quote!(pub #n: &'a #t)
    });
    let ready = inputs.iter().map(|e| {
        let n = &e.name;
        quote!(input.#n.get().is_some())
    });
    let fields_init = inputs.iter().map(|e| {
        let n = &e.name;
        if let Some(f) = &e.field {
            quote!(#n: &input.#n.get().unwrap().#f)
        } else {
            quote!(#n: input.#n.get().unwrap())
        }
    });
    let source_init = sources.iter().map(|e| {
        let n = &e.name;
        quote!(#n: input.#n)
    });
    let make_inputs = quote!(#values { #(#fields_init,)* #(#source_init,)* __marker: ::std::marker::PhantomData });
    let checks = inputs.iter().map(|e| {
        let t = &e.ty;
        if kind == "book" || kind == "oms" {
            quote!(::dataflow_trading::require_book::<#t>();)
        } else {
            quote!(::dataflow_trading::require_feature::<#t>();)
        }
    });
    let source_checks = sources.iter().map(|e| {
        let t = &e.ty;
        quote!({ fn check<T: ::dataflow_flat::Root>() {} check::<#t>(); })
    });
    let tr = format_ident!(
        "{}",
        match kind {
            "book" => "Book",
            "signal" => "Signal",
            "quote_engine" => "QuoteEngine",
            _ => "Oms",
        }
    );
    let tr = quote!(::dataflow_trading::#tr);
    let identity = if kind == "quote_engine" || kind == "oms" {
        let e = args
            .exchange
            .ok_or_else(|| syn::Error::new_spanned(name, "exchange is required"))?;
        if kind == "quote_engine" {
            let s = args
                .symbol
                .ok_or_else(|| syn::Error::new_spanned(name, "symbol is required"))?;
            quote!(impl #ig ::dataflow_trading::Market for #name #tg #wc { type Exchange=#e; type Symbol=#s; })
        } else {
            if args.symbol.is_some() {
                return Err(syn::Error::new_spanned(
                    name,
                    "OMS identity is exchange only",
                ));
            }
            quote!(impl #ig ::dataflow_trading::Venue for #name #tg #wc { type Exchange=#e; })
        }
    } else {
        quote!()
    };
    let value_ty = match kind {
        "book" => quote!(<Self as #tr>::State),
        "signal" => quote!(<Self as #tr>::Output),
        _ => quote!(::dataflow_trading::Quote),
    };
    let marker = if kind == "oms" {
        quote!()
    } else {
        let extra = if kind == "book" {
            quote!(impl #ig ::dataflow_trading::BookNode for #name #tg #wc {})
        } else if kind == "signal" {
            quote!(impl #ig ::dataflow_trading::FeatureNode for #name #tg #wc {})
        } else {
            quote!()
        };
        quote!(impl #ig ::dataflow_trading::ValueNode for #name #tg #wc { type Value=#value_ty; } #extra)
    };
    let (output, construct, update) = if kind == "oms" {
        let symbols:Vec<_> = quotes.iter().map(|q| { let t=&q.ty; quote!(::dataflow_trading::quote_symbol::<#t, <Self as ::dataflow_trading::Venue>::Exchange>()) }).collect();
        let qready = quotes.iter().map(|q| {
            let n = &q.name;
            quote!(input.#n.get().is_some())
        });
        let revisions = quotes.iter().map(|q| {
            let n = &q.name;
            quote!(input.#n.revision)
        });
        let entries=quotes.iter().enumerate().map(|(i,q)| { let n=&q.name; let sym=&symbols[i]; quote!(::dataflow_trading::QuoteEntry { symbol: #sym, quote: *input.#n.get().unwrap(), updated: output.revisions.get(#i).copied().unwrap_or(0) != input.#n.revision }) });
        (quote!(::dataflow_trading::Orders<<<Self as ::dataflow_trading::Venue>::Exchange as ::dataflow_trading::Exchange>::Order>), quote!({
            let symbols = [#(#symbols),*];
            for i in 0..symbols.len() { assert!(!symbols[..i].contains(&symbols[i]), "duplicate quote engine for exchange/symbol"); }
            (<Self as #tr>::new(config), Default::default())
        }), quote!({
            output.orders.clear();
            if true #(&& #ready)* #(&& #qready)* {
                let entries = [#(#entries),*];
                output.orders = <Self as #tr>::compute(self, #make_inputs, ::dataflow_trading::Quotes::new(&entries));
                output.revisions.clear(); output.revisions.extend([#(#revisions),*]);
            }
        }))
    } else if kind == "book" {
        (
            quote!(::dataflow_trading::Published<#value_ty>),
            quote!({
                let (inner, state) = <Self as #tr>::new(config);
                let ready = <Self as #tr>::ready(&inner, &state);
                (inner, ::dataflow_trading::Published { value: Some(state), ready, revision: 0 })
            }),
            quote!({
                if true #(&& #ready)* {
                    let state = output.value.as_mut().expect("Book state exists");
                    <Self as #tr>::update(self, #make_inputs, state);
                    output.ready = <Self as #tr>::ready(self, state);
                    output.revision = output.revision.checked_add(1).expect("revision overflow");
                } else { output.ready = false; }
            }),
        )
    } else {
        (
            quote!(::dataflow_trading::Published<#value_ty>),
            quote!((<Self as #tr>::new(config), Default::default())),
            quote!({
                if true #(&& #ready)* { output.publish(<Self as #tr>::compute(self, #make_inputs)); }
                else { output.ready = false; }
            }),
        )
    };
    Ok(quote! {
        #[derive(::dataflow_flat::ComponentSubscription, ::dataflow_flat::UpdateHandler)]
        #[subscribe(#(#deps),*)]
        #item
        #vis struct #values #vg #wc { #(#fields,)* #(#source_fields,)* __marker: ::std::marker::PhantomData<&'a #name #tg> }
        impl #ig ::dataflow_trading::Inputs for #name #tg #wc { type Inputs<'a> = #values #vtg; }
        #identity #marker
        impl #ig ::dataflow_flat::Component for #name #tg #wc {
            type Config = <Self as #tr>::Config;
            type Output = #output;
            fn new(config: Self::Config, _: Self::Init<'_>) -> (Self, Self::Output) { #(#checks)* #(#source_checks)* #construct }
            fn update(&mut self, _: Self::Input<'_>) -> Self::Output { panic!("trading nodes require the Pipeline recycled-output driver") }
            fn take_output(output: &mut Self::Output) -> Option<Self::Output> { Some(::std::mem::take(output)) }
            fn update_recycled(&mut self, input: Self::Input<'_>, previous: Option<Self::Output>) -> Self::Output {
                let mut output = previous.expect("trading output must be recycled"); #update output
            }
        }
    })
}
fn attribute(kind: &str, args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);
    let item = parse_macro_input!(item as ItemStruct);
    expand(kind, args, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
#[proc_macro_attribute]
pub fn book(a: TokenStream, i: TokenStream) -> TokenStream {
    attribute("book", a, i)
}
#[proc_macro_attribute]
pub fn signal(a: TokenStream, i: TokenStream) -> TokenStream {
    attribute("signal", a, i)
}
#[proc_macro_attribute]
pub fn quote_engine(a: TokenStream, i: TokenStream) -> TokenStream {
    attribute("quote_engine", a, i)
}
#[proc_macro_attribute]
pub fn oms(a: TokenStream, i: TokenStream) -> TokenStream {
    attribute("oms", a, i)
}
#[proc_macro_derive(SignalOutputs)]
pub fn outputs(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let result = (|| {
        let syn::Data::Struct(s) = &input.data else {
            return Err(syn::Error::new_spanned(
                &input,
                "signal outputs must be a named struct",
            ));
        };
        let syn::Fields::Named(fields) = &s.fields else {
            return Err(syn::Error::new_spanned(
                &input,
                "signal outputs must be a named struct",
            ));
        };
        if fields.named.is_empty() {
            return Err(syn::Error::new_spanned(
                &input,
                "signal outputs must have at least one field",
            ));
        }
        for f in &fields.named {
            if !matches!(&f.ty, Type::Path(p) if p.path.is_ident("f64"))
                || !matches!(f.vis, syn::Visibility::Public(_))
            {
                return Err(syn::Error::new_spanned(
                    f,
                    "signal output fields must be public f64 values",
                ));
            }
        }
        let labels = fields
            .named
            .iter()
            .map(|f| f.ident.as_ref().unwrap().to_string());
        let n = &input.ident;
        let (ig, tg, wc) = input.generics.split_for_impl();
        Ok(
            quote!(impl #ig ::dataflow_trading::SignalOutputs for #n #tg #wc { const LABELS: &'static [&'static str] = &[#(#labels),*]; }),
        )
    })();
    result.unwrap_or_else(syn::Error::into_compile_error).into()
}
