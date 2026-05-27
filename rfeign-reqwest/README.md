# rfeign-reqwest

[reqwest](https://crates.io/crates/reqwest) transport implementation for [rfeign](https://crates.io/crates/rfeign).

Features:

- `ReqwestTransport` — default HTTP transport
- `ReqwestTransportBuilder` — configure retry, tracing, custom reqwest client
- Multipart file upload support
- Streaming response support

Most users should depend on [`rfeign`](https://crates.io/crates/rfeign) directly.

## License

MIT OR Apache-2.0
