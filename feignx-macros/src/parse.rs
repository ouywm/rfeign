use proc_macro::TokenStream;
use darling::FromMeta;
use syn::{parse_macro_input, ItemTrait, TraitItem, FnArg, Pat, Meta};

use crate::codegen;
use crate::model::*;
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

pub fn expand_http_client(attr: TokenStream, item: TokenStream) -> TokenStream {
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
    let def = parse_client(&input, base_path);
    let clean = clean_trait(&input);
    let output = codegen::generate(&def, &clean);

    output.into()
}

fn parse_client(input: &ItemTrait, base_path: String) -> ClientDef {
    let methods = input
        .items
        .iter()
        .filter_map(|item| {
            let TraitItem::Fn(method) = item else {
                return None;
            };
            parse_endpoint(method, &base_path)
        })
        .collect();

    ClientDef {
        vis: input.vis.clone(),
        trait_name: input.ident.clone(),
        base_path,
        methods,
    }
}

fn parse_endpoint(method: &syn::TraitItemFn, base_path: &str) -> Option<EndpointMethod> {
    let (http_method, path) = extract_http_attr(method)?;
    let full_path = format!("{}{}", base_path, path);
    let params = parse_params(&method.sig);

    Some(EndpointMethod {
        http_method,
        path: full_path,
        sig: method.sig.clone(),
        params,
    })
}

fn extract_http_attr(method: &syn::TraitItemFn) -> Option<(HttpMethod, String)> {
    for attr in &method.attrs {
        let path = attr.path();
        for &sym in HTTP_METHODS {
            if sym.matches_last_segment(path) {
                let route = extract_route_path(attr);
                let http_method = HttpMethod::from_symbol(sym)?;
                return Some((http_method, route));
            }
        }
    }
    None
}

fn extract_route_path(attr: &syn::Attribute) -> String {
    match &attr.meta {
        Meta::List(list) => {
            list.tokens.to_string().trim_matches('"').to_string()
        }
        _ => String::new(),
    }
}

fn parse_params(sig: &syn::Signature) -> Vec<Param> {
    sig.inputs
        .iter()
        .filter_map(|arg| {
            let FnArg::Typed(pat_type) = arg else {
                return None;
            };
            let Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
                return None;
            };

            let ident = pat_ident.ident.clone();
            let ty = (*pat_type.ty).clone();
            let kind = resolve_param_kind(&pat_type.attrs, &ident);

            Some(Param { ident, ty, kind })
        })
        .collect()
}

fn resolve_param_kind(attrs: &[syn::Attribute], ident: &syn::Ident) -> ParamKind {
    for attr in attrs {
        let path = attr.path();

        if *path == PATH {
            let name = parse_name_arg(attr)
                .unwrap_or_else(|| ident.to_string());
            return ParamKind::Path { name };
        }
        if *path == QUERY {
            let name = parse_name_arg(attr);
            return ParamKind::Query { name };
        }
        if *path == BODY {
            return ParamKind::Body;
        }
        if *path == HEADER {
            let name = parse_name_arg(attr)
                .unwrap_or_else(|| ident.to_string());
            return ParamKind::Header { name };
        }
        if *path == HEADERS {
            return ParamKind::Headers;
        }
    }
    ParamKind::Body
}

fn parse_name_arg(attr: &syn::Attribute) -> Option<String> {
    match &attr.meta {
        Meta::List(list) => {
            let tokens = list.tokens.to_string();
            let trimmed = tokens.trim().trim_matches('"');
            if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
        }
        _ => None,
    }
}

fn clean_trait(input: &ItemTrait) -> ItemTrait {
    let mut clean = input.clone();
    for item in &mut clean.items {
        if let TraitItem::Fn(method) = item {
            method.attrs.retain(|attr| {
                !HTTP_METHODS.iter().any(|s| s.matches_last_segment(attr.path()))
            });
            for arg in &mut method.sig.inputs {
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