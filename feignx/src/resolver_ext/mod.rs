#[cfg(feature = "nacos")]
pub mod nacos;

#[cfg(feature = "nacos")]
pub use nacos::{NacosResolver, NacosResolverBuilder};

#[cfg(feature = "consul")]
pub mod consul;

#[cfg(feature = "consul")]
pub use consul::ConsulResolver;
