# rfeign-core

Core traits and types for the [rfeign](https://crates.io/crates/rfeign) declarative HTTP client.

This crate provides the foundational abstractions:

- `Transport` — HTTP transport layer trait
- `Middleware` — request/response middleware chain
- `RequestInterceptor` / `ResponseInterceptor`
- `Auth` — authentication (Bearer, Basic, Dynamic)
- `UrlResolver` — service discovery abstraction
- `Encoder` / `Decoder` — codec traits
- `Client` / `ClientBuilder` — core client configuration
- `RequestBuilder` — fluent request API

Most users should depend on [`rfeign`](https://crates.io/crates/rfeign) directly.

## License

MIT OR Apache-2.0
