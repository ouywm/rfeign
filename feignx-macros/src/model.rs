use crate::symbol::Symbol;

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

    pub fn as_ident(&self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Delete => "DELETE",
            Self::Patch => "PATCH",
            Self::Head => "HEAD",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParamKind {
    Path { name: String },
    Query { name: Option<String> },
    Body,
    Header { name: String },
    Headers,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub kind: ParamKind,
}

#[derive(Debug)]
pub struct EndpointMethod {
    pub http_method: HttpMethod,
    pub path: String,
    pub sig: syn::Signature,
    pub params: Vec<Param>,
}

#[derive(Debug)]
pub struct ClientDef {
    pub vis: syn::Visibility,
    pub trait_name: syn::Ident,
    pub base_path: String,
    pub methods: Vec<EndpointMethod>,
}