use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Meta};

pub fn expand(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => return quote! { compile_error!("Args only supports named fields"); }.into(),
        },
        _ => return quote! { compile_error!("Args can only be derived on structs"); }.into(),
    };

    let mut path_arms = Vec::new();
    let mut query_arms = Vec::new();
    let mut header_arms = Vec::new();
    let mut body_expr = None;

    for field in fields {
        let ident = field.ident.as_ref().unwrap();
        let field_name = ident.to_string();

        for attr in &field.attrs {
            let path = attr.path();

            if path.is_ident("path") {
                let param_name = extract_name_arg(attr).unwrap_or(field_name.clone());
                path_arms.push(quote! {
                    (#param_name, self.#ident.to_string())
                });
                break;
            }
            if path.is_ident("query") {
                let param_name = extract_name_arg(attr).unwrap_or(field_name.clone());
                if is_option_type(&field.ty) {
                    query_arms.push(quote! {
                        if let Some(ref v) = self.#ident {
                            pairs.push((#param_name.to_string(), v.to_string()));
                        }
                    });
                } else {
                    query_arms.push(quote! {
                        pairs.push((#param_name.to_string(), self.#ident.to_string()));
                    });
                }
                break;
            }
            if path.is_ident("body") {
                body_expr = Some(quote! {
                    Some(rfeign::bytes::Bytes::from(
                        serde_json::to_vec(&self.#ident).unwrap_or_default()
                    ))
                });
                break;
            }
            if path.is_ident("header") {
                let header_name = extract_name_arg(attr).unwrap_or(field_name.clone());
                header_arms.push(quote! {
                    (#header_name, self.#ident.to_string())
                });
                break;
            }
        }
    }

    let body_impl = body_expr.unwrap_or(quote! { None });

    let expanded = quote! {
        impl rfeign::ArgsProvider for #name {
            fn path_params(&self) -> Vec<(&str, String)> {
                vec![#(#path_arms),*]
            }

            fn query_pairs(&self) -> Vec<(String, String)> {
                let mut pairs = Vec::new();
                #(#query_arms)*
                pairs
            }

            fn headers(&self) -> Vec<(&str, String)> {
                vec![#(#header_arms),*]
            }

            fn body_bytes(&self) -> Option<rfeign::bytes::Bytes> {
                #body_impl
            }
        }
    };

    expanded.into()
}

fn extract_name_arg(attr: &syn::Attribute) -> Option<String> {
    if let Meta::List(list) = &attr.meta {
        let tokens = list.tokens.to_string();
        let trimmed = tokens.trim().trim_matches('"');
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

fn is_option_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        return type_path.path.segments.last()
            .is_some_and(|seg| seg.ident == "Option");
    }
    false
}
