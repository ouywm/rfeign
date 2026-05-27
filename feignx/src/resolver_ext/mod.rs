#[cfg(feature = "nacos")]
pub mod nacos;

#[cfg(feature = "nacos")]
pub use nacos::{NacosResolver, NacosResolverBuilder};
