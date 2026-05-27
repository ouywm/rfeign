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

    pub fn to_ident(self) -> syn::Ident {
        let name = match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
        };
        format_ident!("{}", name)
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
}

pub fn parse(method: &syn::TraitItemFn, base_path: &str) -> Option<EndpointMethod> {
    let (http_method, route) = extract_http_attr(method)?;
    let path = format!("{}{}", base_path, route);
    let params = extract_params(&method.sig);

    Some(EndpointMethod {
        http_method,
        path,
        sig: method.sig.clone(),
        params,
    })
}

pub fn generate(endpoint: &EndpointMethod, class_headers: &[(String, String)]) -> TokenStream {
    let sig = strip_param_attrs(&endpoint.sig);
    let method_name = endpoint.http_method.to_method_name();
    let url_expr = build_url_expr(&endpoint.path, &endpoint.params);
    let query_chain = build_query_chain(&endpoint.params);
    let class_header_chain = build_class_header_chain(class_headers);
    let header_chain = build_header_chain(&endpoint.params);
    let body_chain = build_body_chain(&endpoint.params);

    quote! {
        #sig {
            self.client.#method_name(#url_expr)
                #class_header_chain
                #query_chain
                #header_chain
                #body_chain
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