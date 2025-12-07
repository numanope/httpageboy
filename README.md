# HTTPageboy

Minimal HTTP server package for handling request/response transmission.
Focuses only on transporting a well formed HTTP message; does not process or decide how the server behaves.
Aspires to become runtime-agnostic, with minimal, solid, and flexible dependencies.

## Example

The core logic resides in `src/lib.rs`.

### See it working out of the box on [this video](https://www.youtube.com/watch?v=VwRYWJ33C4o)

The following example is executable. Run `cargo run` to see the available variants and navigate to [http://127.0.0.1:7878](http://127.0.0.1:7878) in your browser.

A basic server setup:

```rust
#![cfg(feature = "async_tokio")]
use httpageboy::{Rt, Response, Server, StatusCode};

/// Minimal async handler: waits 100ms and replies "ok"
async fn demo(_req: &()) -> Response {
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  Response {
    status: StatusCode::Ok.to_string(),
    content_type: "text/plain".into(),
    content: b"ok".to_vec(),
  }
}

#[tokio::main]
async fn main() {
  let mut srv = Server::new("127.0.0.1:7878", None).await.unwrap();
  srv.add_route("/", Rt::GET, handler!(demo));
  srv.run().await;
}
````

## Testing

The test helpers let you spin servers and hit them with raw HTTP payloads using a single `run_test` function (async or sync, según feature). Ejemplo mínimo (Tokio):

```rust
use httpageboy::test_utils::{active_test_server_url, run_test, setup_test_server};
use httpageboy::Server;

async fn server_factory() -> Server {
  // tu factory real
  Server::new(active_test_server_url(), None).await.unwrap()
}

#[tokio::test]
async fn test_home_ok() {
  setup_test_server(Some(active_test_server_url()), || server_factory()).await;
  let body = run_test(
    b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n",
    b"home",
    Some(active_test_server_url()),
  )
  .await;
  assert!(body.contains(\"home\"));
}
```

Comandos:

```bash
cargo test --features sync --test test_sync
cargo test --features async_tokio --test test_async_tokio
cargo test --features async_std --test test_async_std
cargo test --features async_smol --test test_async_smol
```

## Examples

Additional examples can be found within the tests.

## License

Copyright (c) 2025 [fahedsl](https://gitlab.com/fahedsl).
This project is licensed under the [MIT License](https://opensource.org/licenses/MIT).
