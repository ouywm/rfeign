pub use rfeign_core::args;
pub use rfeign_core::args::ArgsProvider;
pub use rfeign_core::auth;
pub use rfeign_core::client;
pub use rfeign_core::codec;
pub use rfeign_core::error;
pub use rfeign_core::error_decoder;
pub use rfeign_core::interceptor;
pub use rfeign_core::log;
pub use rfeign_core::middleware;
pub use rfeign_core::part;
pub use rfeign_core::request;
pub use rfeign_core::resolver;
pub use rfeign_core::response;
pub use rfeign_core::stream;
pub use rfeign_core::timeout;
pub use rfeign_core::transport;

pub mod resolver_ext;
pub use resolver_ext::*;

pub use rfeign_core::client::{Client, ClientBuilder};
pub use rfeign_core::error::{Error, Result};
pub use rfeign_core::log::LogLevel;
pub use rfeign_core::log::LoggingMiddleware;

pub use ::bytes;
pub use ::http;
pub use ::async_trait::async_trait;
pub use ::serde_urlencoded;

#[cfg(feature = "reqwest")]
pub use rfeign_reqwest::{ReqwestTransport, ReqwestTransportBuilder};

pub use rfeign_macros::*;