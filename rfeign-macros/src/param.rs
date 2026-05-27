use syn::{FnArg, Pat, Meta};

use crate::symbol::*;

#[derive(Debug, Clone)]
pub enum ParamKind {
    Path { name: String },
    Query { name: Option<String>, format: QueryFormat },
    Body,
    Header { name: String },
    Headers,
    Part { name: String },
    Args,
}

#[derive(Debug, Clone, Default)]
pub enum QueryFormat {
    #[default]
    Default,
    Multi,
    Csv,
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
            let (name, format) = parse_query_args(attr);
            return ParamKind::Query { name, format };
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
        if *path == ARGS {
            return ParamKind::Args;
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

fn parse_query_args(attr: &syn::Attribute) -> (Option<String>, QueryFormat) {
    let mut name = None;
    let mut format = QueryFormat::Default;

    if let Meta::List(list) = &attr.meta {
        let tokens = list.tokens.to_string();
        for part in tokens.split(',') {
            let part = part.trim();
            if let Some(val) = part.strip_prefix("name") {
                let val = val.trim().trim_start_matches('=').trim().trim_matches('"');
                if !val.is_empty() {
                    name = Some(val.to_string());
                }
            } else if let Some(val) = part.strip_prefix("format") {
                let val = val.trim().trim_start_matches('=').trim().trim_matches('"');
                format = match val {
                    "multi" => QueryFormat::Multi,
                    "csv" => QueryFormat::Csv,
                    _ => QueryFormat::Default,
                };
            } else {
                let val = part.trim_matches('"');
                if !val.is_empty() {
                    name = Some(val.to_string());
                }
            }
        }
    }

    (name, format)
}