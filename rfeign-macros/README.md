# rfeign-macros

Procedural macros for the [rfeign](https://crates.io/crates/rfeign) declarative HTTP client.

Provides:

- `#[http_client]` — define HTTP clients as traits
- `#[get]`, `#[post]`, `#[put]`, `#[delete]`, `#[patch]`, `#[head]` — HTTP methods
- `#[multipart]` — multipart request
- `#[timeout(ms)]` — method-level timeout
- `#[header("key", "value")]` — static headers
- `#[derive(RequestParam)]` — struct to query parameters

Most users should depend on [`rfeign`](https://crates.io/crates/rfeign) directly.

## License

MIT OR Apache-2.0
