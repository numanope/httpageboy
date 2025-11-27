#![cfg(feature = "async_smol")]

use httpageboy::test_utils::{active_server_url, run_test, setup_test_server};
use httpageboy::{Request, Response, Rt, Server, StatusCode, handler};
use std::collections::BTreeMap;

async fn create_test_server() -> Server {
  let mut server = Server::new(active_server_url(), None).await.unwrap();
  server.add_route("/", Rt::GET, handler!(demo_handle_home));
  server.add_route("/test", Rt::GET, handler!(demo_handle_get));
  server.add_route("/test", Rt::POST, handler!(demo_handle_post));
  server.add_route("/test/{param1}", Rt::POST, handler!(demo_handle_post));
  server.add_route("/test/{param1}/{param2}", Rt::POST, handler!(demo_handle_post));
  server.add_route("/test", Rt::PUT, handler!(demo_handle_put));
  server.add_route("/test", Rt::DELETE, handler!(demo_handle_delete));
  server.add_files_source("res");
  server
}

async fn demo_handle_home(_request: &Request) -> Response {
  Response {
    status: StatusCode::Ok.to_string(),
    content_type: String::new(),
    content: b"home".to_vec(),
  }
}

async fn demo_handle_get(_request: &Request) -> Response {
  Response {
    status: StatusCode::Ok.to_string(),
    content_type: String::new(),
    content: b"get".to_vec(),
  }
}

async fn demo_handle_post(_request: &Request) -> Response {
  let mut ordered: BTreeMap<&String, &String> = BTreeMap::new();
  for (k, v) in &_request.params {
    ordered.insert(k, v);
  }
  let body = format!(
    "Method: {}\nUri: {}\nParams: {:?}\nBody: {:?}",
    _request.method, _request.path, ordered, _request.body
  );
  Response {
    status: StatusCode::Ok.to_string(),
    content_type: String::new(),
    content: body.into_bytes(),
  }
}

async fn demo_handle_put(_request: &Request) -> Response {
  let body = format!(
    "Method: {}\nUri: {}\nParams: {:?}\nBody: {:?}",
    _request.method, _request.path, _request.params, _request.body
  );
  Response {
    status: StatusCode::Ok.to_string(),
    content_type: String::new(),
    content: body.into_bytes(),
  }
}

async fn demo_handle_delete(_request: &Request) -> Response {
  Response {
    status: StatusCode::Ok.to_string(),
    content_type: String::new(),
    content: b"delete".to_vec(),
  }
}

#[test]
fn test_home() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"GET / HTTP/1.1\r\n\r\n";
    let expected = b"home";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_get() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"GET /test HTTP/1.1\r\n\r\n";
    let expected = b"get";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_get_with_query() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"GET /test?foo=bar&baz=qux HTTP/1.1\r\n\r\n";
    let expected = b"get";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_post() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"POST /test HTTP/1.1\r\n\r\nmueve tu cuerpo";
    let expected = b"Method: POST\nUri: /test\nParams: {}\nBody: \"mueve tu cuerpo\"";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_post_with_query() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"POST /test?foo=bar HTTP/1.1\r\n\r\nmueve tu cuerpo";
    let expected = b"Method: POST\nUri: /test\nParams: {\"foo\": \"bar\"}\nBody: \"mueve tu cuerpo\"";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_post_with_content_length() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"POST /test HTTP/1.1\r\nContent-Length: 15\r\n\r\nmueve tu cuerpo";
    let expected = b"Method: POST\nUri: /test\nParams: {}\nBody: \"mueve tu cuerpo\"";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_post_with_params() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"POST /test/hola/que?param4=hoy&param3=hace HTTP/1.1\r\n\r\nmueve tu cuerpo";
    let expected =
      b"Method: POST\nUri: /test/hola/que\nParams: {\"param1\": \"hola\", \"param2\": \"que\", \"param3\": \"hace\", \"param4\": \"hoy\"}\nBody: \"mueve tu cuerpo\"";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_post_with_incomplete_path_params() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"POST /test/hola HTTP/1.1\r\n\r\nmueve tu cuerpo";
    let expected = b"Method: POST\nUri: /test/hola\nParams: {\"param1\": \"hola\"}\nBody: \"mueve tu cuerpo\"";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_put() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"PUT /test HTTP/1.1\r\n\r\nmueve tu cuerpo";
    let expected = b"Method: PUT\nUri: /test\nParams: {}\nBody: \"mueve tu cuerpo\"";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_delete() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"DELETE /test HTTP/1.1\r\n\r\n";
    let expected = b"delete";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_file_exists() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"GET /numano.png HTTP/1.1\r\nHost: localhost\r\n\r\n";
    let expected = b"HTTP/1.1 200 OK";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_file_not_found() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"GET /no_file_here.png HTTP/1.1\r\n\r\n";
    let expected = b"HTTP/1.1 404 Not Found";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_method_not_allowed() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"BREW /coffee HTTP/1.1\r\n\r\n";
    let expected = b"HTTP/1.1 405 Method Not Allowed";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_empty_request() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"";
    let expected = b"HTTP/1.1 400 Bad Request";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_malformed_request() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"THIS_IS_NOT_HTTP\r\n\r\n";
    let expected = b"HTTP/1.1 400 Bad Request";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_unsupported_http_version() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"GET / HTTP/0.9\r\n\r\n";
    let expected = b"HTTP/1.1 505 HTTP Version Not Supported";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}

#[test]
fn test_long_path() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let long_path = "/".to_string() + &"a".repeat(10_000);
    let request = format!("GET {} HTTP/1.1\r\n\r\n", long_path);
    let expected = b"HTTP/1.1 414 URI Too Long";
    run_test(request.as_bytes(), expected);
  });
}

#[test]
fn test_missing_method() {
  smol::block_on(async {
    setup_test_server(|| create_test_server()).await;
    let request = b"/ HTTP/1.1\r\n\r\n";
    let expected = b"HTTP/1.1 400 Bad Request";
    smol::Timer::after(std::time::Duration::from_millis(100)).await;
    run_test(request, expected);
  });
}
