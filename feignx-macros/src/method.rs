use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, Meta};

use crate::param::{extract_params, Param, ParamKind};
use crate::symbol::*;

#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
}

impl HttpMethod {
    pub fn from_symbol(sym: Symbol) -> Option<Self> {
        match sym.0 {
            "get" => Some(Self::Get),
            "post" => Some(Self::Post),
            "put" => Some(Self::Put),
            "delete" => Some(Self::Delete),
            "patch" => Some(Self::Patch),
            "head" => Some(Self::Head),
            _ => None,
        }
    }

    pub fn to_method_name(self) -> syn::Ident {
        let name = match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Put => "put",
            Self::Delete => "delete",
            Self::Patch => "patch",
            Self::Head => "head",
        };
        format_ident!("{}", name)
    }
}

pub struct EndpointMethod {
    pub http_method: HttpMethod,
    pub path: String,
    pub sig: syn::Signature,
    pub params: Vec<Param>,
    pub is_multipart: bool,
    pub timeout_ms: Option<u64>,
    pub static_headers: Vec<(String, String)>,
}

pub fn parse(method: &syn::TraitItemFn, base_path: &str) -> Option<EndpointMethod> {
    let (http_method, route) = extract_http_attr(method)?;
    let path = format!("{}{}", base_path, route);
    let params = extract_params(&method.sig);
    let is_multipart = has_attr(&method.attrs, MULTIPART);
    let timeout_ms = extract_timeout(&method.attrs);
    let static_headers = extract_method_headers(&method.attrs);

    Some(EndpointMethod {
        http_method,
        path,
        sig: method.sig.clone(),
        params,
        is_multipart,
        timeout_ms,
        static_headers,
    })
}

pub fn generate(endpoint: &EndpointMethod, class_headers: &[(String, String)]) -> TokenStream {
    let sig = strip_param_attrs(&endpoint.sig);
    let method_name = endpoint.http_method.to_method_name();
    let url_expr = build_url_expr(&endpoint.path, &endpoint.params);
    let query_chain = build_query_chain(&endpoint.params);
    let class_header_chain = build_class_header_chain(class_headers);
    let method_header_chain = build_class_header_chain(&endpoint.static_headers);
    let header_chain = build_header_chain(&endpoint.params);
    let timeout_chain = build_timeout_chain(endpoint.timeout_ms);

    let body_or_parts = if endpoint.is_multipart {
        build_multipart_chain(&endpoint.params)
    } else {
        build_body_chain(&endpoint.params)
    };

    quote! {
        #sig {
            self.client.#method_name(#url_expr)
                #class_header_chain
                #method_header_chain
                #query_chain
                #header_chain
                #timeout_chain
                #body_or_parts
                .send()
                .await?
                .json()
        }
    }
}

fn build_class_header_chain(headers: &[(String, String)]) -> TokenStream {
    let calls: Vec<_> = headers
        .iter()
        .map(|(k, v)| quote! { .header(#k, #v) })
        .collect();

    quote! { #(#calls)* }
}

fn extract_http_attr(method: &syn::TraitItemFn) -> Option<(HttpMethod, String)> {
    for attr in &method.attrs {
        for &sym in HTTP_METHODS {
            if sym.matches_last_segment(attr.path()) {
                let route = parse_route(attr);
                return HttpMethod::from_symbol(sym).map(|m| (m, route));
            }
        }
    }
    None
}

fn parse_route(attr: &syn::Attribute) -> String {
    match &attr.meta {
        Meta::List(list) => list.tokens.to_string().trim_matches('"').to_string(),
        _ => String::new(),
    }
}

fn strip_param_attrs(sig: &syn::Signature) -> syn::Signature {
    let mut clean = sig.clone();
    for input in &mut clean.inputs {
        if let FnArg::Typed(pat_type) = input {
            pat_type.attrs.retain(|attr| {
                !PARAM_ATTRS.iter().any(|s| *attr.path() == *s)
            });
        }
    }
    clean
}

fn build_url_expr(path: &str, params: &[Param]) -> TokenStream {
    let mut fmt = path.to_string();
    let mut args = Vec::new();

    for param in params {
        if let ParamKind::Path { name } = &param.kind {
            let placeholder = format!("{{{}}}", name);
            if fmt.contains(&placeholder) {
                fmt = fmt.replace(&placeholder, "{}");
                let ident = &param.ident;
                args.push(quote! { #ident });
            }
        }
    }

    if args.is_empty() {
        quote! { #fmt }
    } else {
        quote! { format!(#fmt, #(#args),*) }
    }
}

fn build_query_chain(params: &[Param]) -> TokenStream {
    let calls: Vec<_> = params
        .iter()
        .filter_map(|p| {
            let ParamKind::Query { name } = &p.kind else {
                return None;
            };
            let ident = &p.ident;
            let key = match name {
                Some(n) => n.clone(),
                None => ident.to_string(),
            };
            Some(quote! {
                .query_pair(#key, &feignx::serde_urlencoded::to_string(&#ident).unwrap_or_default())
            })
        })
        .collect();

    quote! { #(#calls)* }
}

fn build_body_chain(params: &[Param]) -> TokenStream {
    for param in params {
        if matches!(param.kind, ParamKind::Body) {
            let ident = &param.ident;
            return quote! { .json(&#ident)? };
        }
    }
    TokenStream::new()
}

fn build_header_chain(params: &[Param]) -> TokenStream {
    let calls: Vec<_> = params
        .iter()
        .filter_map(|p| match &p.kind {
            ParamKind::Header { name } => {
                let ident = &p.ident;
                Some(quote! { .header(#name, &#ident.to_string()) })
            }
            ParamKind::Headers => {
                let ident = &p.ident;
                Some(quote! { .headers_map(&#ident) })
            }
            _ => None,
        })
        .collect();

    quote! { #(#calls)* }
}

fn build_multipart_chain(params: &[Param]) -> TokenStream {
    let calls: Vec<_> = params
        .iter()
        .filter_map(|p| {
            if let ParamKind::Part { name } = &p.kind {
                let ident = &p.ident;
                if is_string_type(&p.ty) {
                    Some(quote! { .text_part(#name, #ident) })
                } else {
                    Some(quote! { .part(#name, #ident) })
                }
            } else {
                None
            }
        })
        .collect();

    quote! { #(#calls)* }
}

fn build_timeout_chain(timeout_ms: Option<u64>) -> TokenStream {
    match timeout_ms {
        Some(ms) => quote! {
            .timeout(::std::time::Duration::from_millis(#ms))
        },
        None => TokenStream::new(),
    }
}

fn has_attr(attrs: &[syn::Attribute], sym: Symbol) -> bool {
    attrs.iter().any(|a| sym.matches_last_segment(a.path()))
}

fn extract_timeout(attrs: &[syn::Attribute]) -> Option<u64> {
    for attr in attrs {
        if !TIMEOUT.matches_last_segment(attr.path()) {
            continue;
        }
        if let syn::Meta::List(list) = &attr.meta {
            let tokens = list.tokens.to_string();
            if let Ok(ms) = tokens.trim().parse::<u64>() {
                return Some(ms);
            }
        }
    }
    None
}

fn is_string_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(p) = ty {
        return p.path.segments.last()
            .is_some_and(|seg| seg.ident == "String" || seg.ident == "str");
    }
    false
}

fn extract_method_headers(attrs: &[syn::Attribute]) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    for attr in attrs {
        if !HEADER.matches_last_segment(attr.path()) {
            continue;
        }
        if let syn::Meta::List(list) = &attr.meta {
            let parser =
                syn::punctuated::Punctuated::<syn::LitStr, syn::Token![,]>::parse_terminated;
            if let Ok(lits) = syn::parse::Parser::parse2(parser, list.tokens.clone()) {
                let parts: Vec<_> = lits.into_iter().collect();
                if parts.len() == 2 {
                    headers.push((parts[0].value(), parts[1].value()));
                }
            }
        }
    }
    headers
}