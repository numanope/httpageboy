#![cfg(feature = "async_tokio")]

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
  let res_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("res");
  server.add_files_source(res_path.to_str().unwrap());
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

#[tokio::test]
async fn test_home() {
  setup_test_server(|| create_test_server()).await;
  let request = b"GET / HTTP/1.1\r\n\r\n";
  let expected = b"home";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_get() {
  setup_test_server(|| create_test_server()).await;
  let request = b"GET /test HTTP/1.1\r\n\r\n";
  let expected = b"get";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_get_with_query() {
  setup_test_server(|| create_test_server()).await;
  let request = b"GET /test?foo=bar&baz=qux HTTP/1.1\r\n\r\n";
  let expected = b"get";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_post() {
  setup_test_server(|| create_test_server()).await;
  let request = b"POST /test HTTP/1.1\r\n\r\nmueve tu cuerpo";
  let expected = b"Method: POST\nUri: /test\nParams: {}\nBody: \"mueve tu cuerpo\"";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_post_with_query() {
  setup_test_server(|| create_test_server()).await;
  let request = b"POST /test?foo=bar HTTP/1.1\r\n\r\nmueve tu cuerpo";
  let expected = b"Method: POST\nUri: /test\nParams: {\"foo\": \"bar\"}\nBody: \"mueve tu cuerpo\"";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_post_with_content_length() {
  setup_test_server(|| create_test_server()).await;
  let request = b"POST /test HTTP/1.1\r\nContent-Length: 15\r\n\r\nmueve tu cuerpo";
  let expected = b"Method: POST\nUri: /test\nParams: {}\nBody: \"mueve tu cuerpo\"";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_post_with_params() {
  setup_test_server(|| create_test_server()).await;
  let request = b"POST /test/hola/que?param4=hoy&param3=hace HTTP/1.1\r\n\r\nmueve tu cuerpo";
  let expected =
    b"Method: POST\nUri: /test/hola/que\nParams: {\"param1\": \"hola\", \"param2\": \"que\", \"param3\": \"hace\", \"param4\": \"hoy\"}\nBody: \"mueve tu cuerpo\"";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_post_with_incomplete_path_params() {
  setup_test_server(|| create_test_server()).await;
  let request = b"POST /test/hola HTTP/1.1\r\n\r\nmueve tu cuerpo";
  let expected = b"Method: POST\nUri: /test/hola\nParams: {\"param1\": \"hola\"}\nBody: \"mueve tu cuerpo\"";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_put() {
  setup_test_server(|| create_test_server()).await;
  let request = b"PUT /test HTTP/1.1\r\n\r\nmueve tu cuerpo";
  let expected = b"Method: PUT\nUri: /test\nParams: {}\nBody: \"mueve tu cuerpo\"";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_delete() {
  setup_test_server(|| create_test_server()).await;
  let request = b"DELETE /test HTTP/1.1\r\n\r\n";
  let expected = b"delete";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_file_exists() {
  setup_test_server(|| create_test_server()).await;
  let request = b"GET /numano.png HTTP/1.1\r\nHost: localhost\r\n\r\n";
  let expected = b"HTTP/1.1 200 OK";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_file_not_found() {
  setup_test_server(|| create_test_server()).await;
  let request = b"GET /no_file_here.png HTTP/1.1\r\n\r\n";
  let expected = b"HTTP/1.1 404 Not Found";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_method_not_allowed() {
  setup_test_server(|| create_test_server()).await;
  let request = b"BREW /coffee HTTP/1.1\r\n\r\n";
  let expected = b"HTTP/1.1 405 Method Not Allowed";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_empty_request() {
  setup_test_server(|| create_test_server()).await;
  let request = b"";
  let expected = b"HTTP/1.1 400 Bad Request";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_malformed_request() {
  setup_test_server(|| create_test_server()).await;
  let request = b"THIS_IS_NOT_HTTP\r\n\r\n";
  let expected = b"HTTP/1.1 400 Bad Request";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_unsupported_http_version() {
  setup_test_server(|| create_test_server()).await;
  let request = b"GET / HTTP/0.9\r\n\r\n";
  let expected = b"HTTP/1.1 505 HTTP Version Not Supported";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}

#[tokio::test]
async fn test_long_path() {
  setup_test_server(|| create_test_server()).await;
  let long_path = "/".to_string() + &"a".repeat(10_000);
  let request = format!("GET {} HTTP/1.1\r\n\r\n", long_path);
  let expected = b"HTTP/1.1 414 URI Too Long";
  run_test(request.as_bytes(), expected);
}

#[tokio::test]
async fn test_missing_method() {
  setup_test_server(|| create_test_server()).await;
  let request = b"/ HTTP/1.1\r\n\r\n";
  let expected = b"HTTP/1.1 400 Bad Request";
  tokio::time::sleep(std::time::Duration::from_millis(100)).await;
  run_test(request, expected);
}
