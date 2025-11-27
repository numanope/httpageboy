// examples/api_consumer.rs

use httpageboy::{Request, Response, Rt, Server, StatusCode};

// ---- Synchronous Implementation ----
#[cfg(feature = "sync")]
mod sync_impl {
  use super::*;
  use httpageboy::handler;

  fn demo_handle_home(_request: &Request) -> Response {
    Response {
      status: StatusCode::Ok.to_string(),
      content_type: "text/plain".to_string(),
      content: "Welcome to the SYNC API consumer example!".as_bytes().to_vec(),
    }
  }

  fn demo_handle_get(_request: &Request) -> Response {
    Response {
      status: StatusCode::Ok.to_string(),
      content_type: "text/plain".to_string(),
      content: "This is a SYNC GET response.".as_bytes().to_vec(),
    }
  }

  fn demo_handle_post(request: &Request) -> Response {
    let body_str = String::from_utf8_lossy(request.body.as_bytes());
    let response_body = format!("Received SYNC POST with body: {}", body_str);
    Response {
      status: StatusCode::Ok.to_string(),
      content_type: "text/plain".to_string(),
      content: response_body.as_bytes().to_vec(),
    }
  }

  pub fn main() {
    println!("Starting sync server at http://127.0.0.1:8088");
    let mut server = Server::new("127.0.0.1:8088", 4, None).expect("Failed to create server");

    server.add_route("/", Rt::GET, handler!(demo_handle_home));
    server.add_route("/test", Rt::GET, handler!(demo_handle_get));
    server.add_route("/test", Rt::POST, handler!(demo_handle_post));

    let res_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("res");
    server.add_files_source(res_path.to_str().unwrap());

    server.run();
  }
}

// ---- Asynchronous Implementation ----
#[cfg(any(feature = "async_tokio", feature = "async_std", feature = "async_smol"))]
mod async_impl {
  use super::*;
  use httpageboy::handler;

  async fn demo_handle_home(_request: &Request) -> Response {
    Response {
      status: StatusCode::Ok.to_string(),
      content_type: "text/plain".to_string(),
      content: "Welcome to the ASYNC API consumer example!".as_bytes().to_vec(),
    }
  }

  async fn demo_handle_get(_request: &Request) -> Response {
    Response {
      status: StatusCode::Ok.to_string(),
      content_type: "text/plain".to_string(),
      content: "This is an ASYNC GET response.".as_bytes().to_vec(),
    }
  }

  async fn demo_handle_post(request: &Request) -> Response {
    let body_str = String::from_utf8_lossy(request.body.as_bytes());
    let response_body = format!("Received ASYNC POST with body: {}", body_str);
    Response {
      status: StatusCode::Ok.to_string(),
      content_type: "text/plain".to_string(),
      content: response_body.as_bytes().to_vec(),
    }
  }

  pub async fn main() {
    println!("Starting async server at http://127.0.0.1:8088");
    let mut server = Server::new("127.0.0.1:8088", None)
      .await
      .expect("Failed to create server");

    server.add_route("/", Rt::GET, handler!(demo_handle_home));
    server.add_route("/test", Rt::GET, handler!(demo_handle_get));
    server.add_route("/test", Rt::POST, handler!(demo_handle_post));

    let res_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("res");
    server.add_files_source(res_path.to_str().unwrap());

    server.run().await;
  }
}

// ---- Main function dispatcher ----

#[cfg(feature = "sync")]
fn main() {
  sync_impl::main();
}

#[cfg(all(feature = "async_tokio", not(feature = "sync")))]
#[tokio::main]
async fn main() {
  async_impl::main().await;
}

#[cfg(all(feature = "async_std", not(feature = "sync"), not(feature = "async_tokio")))]
#[async_std::main]
async fn main() {
  async_impl::main().await;
}

#[cfg(all(
  feature = "async_smol",
  not(feature = "sync"),
  not(feature = "async_tokio"),
  not(feature = "async_std")
))]
fn main() {
  smol::block_on(async_impl::main());
}

// Dummy main if no features are selected to provide a clear error.
#[cfg(not(any(
  feature = "sync",
  feature = "async_tokio",
  feature = "async_std",
  feature = "async_smol"
)))]
fn main() {
  panic!("No feature selected for api_consumer example. Please run with --features <feature_name>");
}
