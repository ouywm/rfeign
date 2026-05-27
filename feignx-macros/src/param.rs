use syn::{FnArg, Pat, Meta};

use crate::symbol::*;

#[derive(Debug, Clone)]
pub enum ParamKind {
    Path { name: String },
    Query { name: Option<String> },
    Body,
    Header { name: String },
    Headers,
    Part { name: String },
}

#[derive(Debug, Clone)]
pub struct Param {
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub kind: ParamKind,
}

pub fn extract_params(sig: &syn::Signature) -> Vec<Param> {
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
            let kind = resolve_kind(&pat_type.attrs, &ident);

            Some(Param { ident, ty, kind })
        })
        .collect()
}

fn resolve_kind(attrs: &[syn::Attribute], ident: &syn::Ident) -> ParamKind {
    for attr in attrs {
        let path = attr.path();

        if *path == PATH {
            let name = match parse_name_arg(attr) {
                Some(n) => n,
                None => ident.to_string(),
            };
            return ParamKind::Path { name };
        }
        if *path == QUERY {
            return ParamKind::Query { name: parse_name_arg(attr) };
        }
        if *path == BODY {
            return ParamKind::Body;
        }
        if *path == HEADER {
            let name = match parse_name_arg(attr) {
                Some(n) => n,
                None => ident.to_string(),
            };
            return ParamKind::Header { name };
        }
        if *path == HEADERS {
            return ParamKind::Headers;
        }
        if *path == PART {
            let name = match parse_name_arg(attr) {
                Some(n) => n,
                None => ident.to_string(),
            };
            return ParamKind::Part { name };
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