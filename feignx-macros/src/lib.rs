use proc_macro::TokenStream;

mod codegen;
mod model;
mod parse;
mod symbol;

/// Declares a declarative HTTP client from a trait definition.
///
/// Generates a `{TraitName}Client` struct that implements the trait,
/// with each method performing the corresponding HTTP request.
///
/// # Example
///
/// ```ignore
/// #[feignx::http_client(base_url = "http://localhost:8080")]
/// trait UserApi {
///     #[feignx::get("/users/{id}")]
///     async fn get_user(&self, #[path] id: i64) -> feignx::Result<User>;
/// }
///
/// let api = UserApiClient::new(client);
/// ```
#[proc_macro_attribute]
pub fn http_client(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse::expand_http_client(attr, item)
}

macro_rules! define_method_macro {
    ($($(#[$meta:meta])* $name:ident),* $(,)?) => {
        $(
            $(#[$meta])*
            #[proc_macro_attribute]
            pub fn $name(_attr: TokenStream, item: TokenStream) -> TokenStream {
                item
            }
        )*
    };
}

define_method_macro! {
    get,
    post,
    put,
    delete,
    patch,
    head,
}
