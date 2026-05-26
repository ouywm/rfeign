use syn::Ident;

#[derive(Copy, Clone)]
pub struct Symbol(pub &'static str);

impl PartialEq<Symbol> for &Ident {
    fn eq(&self, symbol: &Symbol) -> bool {
        *self == symbol.0
    }
}

impl PartialEq<Symbol> for syn::Path {
    fn eq(&self, symbol: &Symbol) -> bool {
        self.is_ident(symbol.0)
    }
}

impl Symbol {
    pub fn matches_last_segment(&self, path: &syn::Path) -> bool {
        path.segments
            .last()
            .map(|s| s.ident == self.0)
            .unwrap_or(false)
    }
}

pub const GET: Symbol = Symbol("get");
pub const POST: Symbol = Symbol("post");
pub const PUT: Symbol = Symbol("put");
pub const DELETE: Symbol = Symbol("delete");
pub const PATCH: Symbol = Symbol("patch");
pub const HEAD: Symbol = Symbol("head");

pub const PATH: Symbol = Symbol("path");
pub const QUERY: Symbol = Symbol("query");
pub const BODY: Symbol = Symbol("body");
pub const HEADER: Symbol = Symbol("header");
pub const HEADERS: Symbol = Symbol("headers");

pub const HTTP_METHODS: &[Symbol] = &[GET, POST, PUT, DELETE, PATCH, HEAD];
pub const PARAM_ATTRS: &[Symbol] = &[PATH, QUERY, BODY, HEADER, HEADERS];