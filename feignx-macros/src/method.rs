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

pub fn generate(endpoint: &EndpointMethod) -> TokenStream {
    let sig = strip_param_attrs(&endpoint.sig);
    let http_method = endpoint.http_method.to_ident();
    let url_expr = build_url_expr(&endpoint.path, &endpoint.params);
    let query_stmts = build_query_stmts(&endpoint.params);
    let body_expr = build_body_expr(&endpoint.params);
    let header_stmts = build_header_stmts(&endpoint.params);

    quote! {
        #sig {
            let mut url = self.client.resolve_url(&#url_expr).await?;
            #query_stmts

            let mut builder = feignx::http::Request::builder()
                .method(feignx::http::Method::#http_method)
                .uri(&url);
            #header_stmts

            let body = #body_expr;
            let request = match builder.body(body) {
                Ok(r) => r,
                Err(e) => return Err(feignx::Error::Other(e.to_string())),
            };
            self.client.send_and_decode(request).await
        }
    }
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

fn build_query_stmts(params: &[Param]) -> TokenStream {
    let pairs: Vec<_> = params
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
                (#key, feignx::serde_urlencoded::to_string(&#ident).unwrap_or_default())
            })
        })
        .collect();

    if pairs.is_empty() {
        return TokenStream::new();
    }

    quote! {
        let qs = [#(#pairs),*]
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        if !qs.is_empty() {
            url.push('?');
            url.push_str(&qs);
        }
    }
}

fn build_body_expr(params: &[Param]) -> TokenStream {
    for param in params {
        if matches!(param.kind, ParamKind::Body) {
            let ident = &param.ident;
            return quote! { self.client.encode_body(&#ident)? };
        }
    }
    quote! { feignx::bytes::Bytes::new() }
}

fn build_header_stmts(params: &[Param]) -> TokenStream {
    let stmts: Vec<_> = params
        .iter()
        .filter_map(|p| match &p.kind {
            ParamKind::Header { name } => {
                let ident = &p.ident;
                Some(quote! {
                    builder = builder.header(#name, #ident.to_string());
                })
            }
            ParamKind::Headers => {
                let ident = &p.ident;
                Some(quote! {
                    for (k, v) in &#ident {
                        builder = builder.header(k.as_str(), v.as_str());
                    }
                })
            }
            _ => None,
        })
        .collect();

    quote! { #(#stmts)* }
}