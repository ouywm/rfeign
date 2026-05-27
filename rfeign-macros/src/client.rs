use proc_macro::TokenStream;
use darling::FromMeta;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemTrait, TraitItem};

use crate::method;
use crate::symbol::*;

#[derive(Debug, FromMeta)]
struct HttpClientAttr {
    #[darling(default)]
    base_url: Option<String>,
    #[darling(default)]
    service: Option<String>,
    #[darling(default)]
    path: Option<String>,
}

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemTrait);

    let attr_meta: syn::punctuated::Punctuated<syn::Meta, syn::Token![,]> =
        parse_macro_input!(attr with syn::punctuated::Punctuated::parse_terminated);
    let nested: Vec<darling::ast::NestedMeta> = attr_meta
        .into_iter()
        .map(darling::ast::NestedMeta::Meta)
        .collect();

    let client_attr = match HttpClientAttr::from_list(&nested) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let base_path = client_attr.path.clone().unwrap_or_default();
    let class_headers = extract_class_headers(&input);
    let endpoints: Vec<_> = input
        .items
        .iter()
        .filter_map(|item| match item {
            TraitItem::Fn(m) => method::parse(m, &base_path),
            _ => None,
        })
        .collect();

    let trait_name = &input.ident;
    let struct_name = format_ident!("{}Client", trait_name);
    let vis = &input.vis;
    let clean_trait = strip_trait_attrs(&input);
    let methods = endpoints.iter().map(|e| method::generate(e, &class_headers));
    let metadata = build_metadata_methods(&client_attr);

    let output = quote! {
        #[rfeign::async_trait]
        #clean_trait

        #[derive(Clone)]
        #vis struct #struct_name {
            client: rfeign::client::Client,
        }

        impl #struct_name {
            pub fn new(client: rfeign::client::Client) -> Self {
                Self { client }
            }

            #metadata
        }

        #[rfeign::async_trait]
        impl #trait_name for #struct_name {
            #(#methods)*
        }
    };

    output.into()
}

fn build_metadata_methods(attr: &HttpClientAttr) -> proc_macro2::TokenStream {
    let base_url = match &attr.base_url {
        Some(v) => quote! { Some(#v) },
        None => quote! { None },
    };
    let service = match &attr.service {
        Some(v) => quote! { Some(#v) },
        None => quote! { None },
    };
    let path = match &attr.path {
        Some(v) => v.clone(),
        None => String::new(),
    };

    quote! {
        pub fn base_url() -> Option<&'static str> { #base_url }
        pub fn service_name() -> Option<&'static str> { #service }
        pub fn path() -> &'static str { #path }
    }
}

fn strip_trait_attrs(input: &ItemTrait) -> ItemTrait {
    let mut clean = input.clone();
    clean.attrs.retain(|attr| !attr.path().is_ident("headers"));
    for item in &mut clean.items {
        if let TraitItem::Fn(m) = item {
            m.attrs.retain(|attr| {
                !HTTP_METHODS.iter().any(|s| s.matches_last_segment(attr.path()))
                && !METHOD_ATTRS.iter().any(|s| s.matches_last_segment(attr.path()))
            });
            for arg in &mut m.sig.inputs {
                if let FnArg::Typed(pat_type) = arg {
                    pat_type.attrs.retain(|attr| {
                        !PARAM_ATTRS.iter().any(|s| *attr.path() == *s)
                    });
                }
            }
        }
    }
    clean
}

/// Extracts `#[headers("Key: Value", ...)]` from the trait-level attributes.
/// Returns a list of (name, value) pairs.
pub fn extract_class_headers(input: &ItemTrait) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    for attr in &input.attrs {
        if !attr.path().is_ident("headers") {
            continue;
        }
        if let syn::Meta::List(list) = &attr.meta {
            let parser = syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated;
            if let Ok(lits) = syn::parse::Parser::parse2(parser, list.tokens.clone()) {
                for lit in lits {
                    let s = lit.value();
                    if let Some((k, v)) = s.split_once(':') {
                        headers.push((k.trim().to_string(), v.trim().to_string()));
                    }
                }
            }
        }
    }
    headers
}