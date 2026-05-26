use quote::{format_ident, quote};
use proc_macro2::TokenStream;
use syn::FnArg;

use crate::model::*;
use crate::symbol::PARAM_ATTRS;

pub fn generate(def: &ClientDef, clean_trait: &syn::ItemTrait) -> TokenStream {
    let trait_name = &def.trait_name;
    let struct_name = format_ident!("{}Client", trait_name);
    let vis = &def.vis;
    let methods = def.methods.iter().map(generate_method);

    quote! {
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
    }
}

fn generate_method(endpoint: &EndpointMethod) -> TokenStream {
    let clean_sig = strip_param_attrs(&endpoint.sig);
    let http_method = format_ident!("{}", endpoint.http_method.as_ident());
    let url_expr = build_url_expr(&endpoint.path, &endpoint.params);
    let query_stmts = build_query_stmts(&endpoint.params);
    let body_expr = build_body_expr(&endpoint.params);
    let header_stmts = build_header_stmts(&endpoint.params);

    quote! {
        #clean_sig {
            let base_url = self.client.url_resolver().resolve("").await?;
            let mut url = format!("{}{}", base_url, #url_expr);

            #query_stmts

            let mut builder = feignx::http::Request::builder()
                .method(feignx::http::Method::#http_method)
                .uri(&url);

            #header_stmts

            let body_bytes = #body_expr;
            let request = builder.body(body_bytes).unwrap();
            let response = self.client.execute(request).await?;
            let status = response.status().as_u16();
            let body = response.into_body();

            if !self.client.is_success(status) {
                let headers = feignx::http::HeaderMap::new();
                return Err(
                    self.client.error_decoder().decode(status, &headers, &body)
                );
            }

            self.client.decode_response(&body)
        }
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
        if let ParamKind::Path { ref name } = param.kind {
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
    let query_params: Vec<_> = params
        .iter()
        .filter(|p| matches!(p.kind, ParamKind::Query { .. }))
        .collect();

    if query_params.is_empty() {
        return TokenStream::new();
    }

    let pairs: Vec<_> = query_params
        .iter()
        .map(|p| {
            let ident = &p.ident;
            let name = match &p.kind {
                ParamKind::Query { name: Some(n) } => n.clone(),
                _ => ident.to_string(),
            };
            quote! { (#name, &feignx::serde_urlencoded::to_string(&#ident).unwrap_or_default()) }
        })
        .collect();

    quote! {
        let query_string = [#(#pairs),*]
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        if !query_string.is_empty() {
            url = format!("{}?{}", url, query_string);
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