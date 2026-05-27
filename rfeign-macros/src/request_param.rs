use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Meta};

pub fn expand(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Data::Struct(data) = &input.data else {
        return syn::Error::new_spanned(&input, "RequestParam can only be derived for structs")
            .to_compile_error()
            .into();
    };

    let Fields::Named(fields) = &data.fields else {
        return syn::Error::new_spanned(&input, "RequestParam requires named fields")
            .to_compile_error()
            .into();
    };

    let field_serializations: Vec<_> = fields
        .named
        .iter()
        .map(|f| {
            let ident = f.ident.as_ref().expect("named field");
            let param_name = match extract_param_name(f) {
                Some(n) => n,
                None => ident.to_string(),
            };
            let is_option = is_option_type(&f.ty);

            if is_option {
                quote! {
                    if let Some(ref v) = self.#ident {
                        pairs.push((#param_name.to_string(), v.to_string()));
                    }
                }
            } else {
                quote! {
                    pairs.push((#param_name.to_string(), self.#ident.to_string()));
                }
            }
        })
        .collect();

    let output = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn to_query_pairs(&self) -> Vec<(String, String)> {
                let mut pairs = Vec::new();
                #(#field_serializations)*
                pairs
            }
        }
    };

    output.into()
}

fn extract_param_name(field: &syn::Field) -> Option<String> {
    for attr in &field.attrs {
        if !attr.path().is_ident("param") {
            continue;
        }
        if let Meta::List(list) = &attr.meta {
            let tokens = list.tokens.to_string();
            if let Some(rest) = tokens.strip_prefix("name") {
                let name = rest.trim().trim_start_matches('=').trim().trim_matches('"');
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
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
