use proc_macro::TokenStream;

mod client;
mod method;
mod multipart;
mod param;
mod request_param;
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
    client::expand(attr, item)
}

/// Derives `to_query_pairs()` for a struct, enabling it as a query parameter object.
///
/// # Example
///
/// ```ignore
/// #[derive(feignx::RequestParam)]
/// struct ListParams {
///     page: u32,
///     #[param(name = "q")]
///     keyword: Option<String>,
/// }
/// ```
#[proc_macro_derive(RequestParam, attributes(param))]
pub fn derive_request_param(item: TokenStream) -> TokenStream {
    request_param::expand(item)
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
    multipart,
    timeout,
}