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

    let base_path = client_attr.path.unwrap_or_default();
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
    let methods = endpoints.iter().map(method::generate);

    let output = quote! {
        #[feignx::async_trait]
        #clean_trait

        #[derive(Clone)]
        #vis struct #struct_name {
            client: feignx::client::Client,
        }

        impl #struct_name {
            pub fn new(client: feignx::client::Client) -> Self {
                Self { client }
            }
        }

        #[feignx::async_trait]
        impl #trait_name for #struct_name {
            #(#methods)*
        }
    };

    output.into()
}

fn strip_trait_attrs(input: &ItemTrait) -> ItemTrait {
    let mut clean = input.clone();
    for item in &mut clean.items {
        if let TraitItem::Fn(m) = item {
            m.attrs.retain(|attr| {
                !HTTP_METHODS.iter().any(|s| s.matches_last_segment(attr.path()))
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