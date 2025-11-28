#![cfg(feature = "sync")]
use httpageboy::test_utils::{POOL_SIZE, run_test, setup_test_server};
use httpageboy::{Request, Response, Rt, Server, StatusCode, handler};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::thread;
use std::time::Duration;

fn common_server_definition(server_url: &str) -> Server {
  let mut server = Server::new(server_url, POOL_SIZE, None).unwrap();
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

fn regular_server_definition() -> Server {
  let mut server = common_server_definition("127.0.0.1:0");
  server
}

fn strict_server_definition() -> String {
  let mut server = common_server_definition("127.0.0.1:1");
  server.with_strict_content_length(true);
  server
}

fn demo_handle_home(_request: &Request) -> Response {
  Response {
    status: StatusCode::Ok.to_string(),
    content_type: String::new(),
    content: "home".as_bytes().to_vec(),
  }
}

fn demo_handle_post(_request: &Request) -> Response {
  // build a BTreeMap to get params in sorted key order
  let mut ordered: BTreeMap<&String, &String> = BTreeMap::new();
  for (k, v) in &_request.params {
    ordered.insert(k, v);
  }

  let request_string = format!(
    "Method: {}\nUri: {}\nParams: {:?}\nBody: {:?}",
    _request.method, _request.path, ordered, _request.body
  );

  Response {
    status: StatusCode::Ok.to_string(),
    content_type: String::new(),
    content: request_string.as_bytes().to_vec(),
  }
}

fn demo_handle_get(_request: &Request) -> Response {
  Response {
    status: StatusCode::Ok.to_string(),
    content_type: String::new(),
    content: "get".as_bytes().to_vec(),
  }
}

fn demo_handle_put(_request: &Request) -> Response {
  let request_string = format!(
    "Method: {}\nUri: {}\nParams: {:?}\nBody: {:?}",
    _request.method, _request.path, _request.params, _request.body
  );
  Response {
    status: StatusCode::Ok.to_string(),
    content_type: String::new(),
    content: request_string.as_bytes().to_vec(),
  }
}

fn demo_handle_delete(_request: &Request) -> Response {
  Response {
    status: StatusCode::Ok.to_string(),
    content_type: String::new(),
    content: "delete".as_bytes().to_vec(),
  }
}

#[test]
fn test_home() {
  setup_test_server(regular_server_definition);
  let request = b"GET / HTTP/1.1\r\n\r\n";
  let expected_response = b"home";
  run_test(request, expected_response);
}

#[test]
fn test_get() {
  setup_test_server(regular_server_definition);
  let request = b"GET /test HTTP/1.1\r\n\r\n";
  let expected_response = b"get";
  run_test(request, expected_response);
}

#[test]
fn test_get_with_query() {
  setup_test_server(regular_server_definition);
  let request = b"GET /test?foo=bar&baz=qux HTTP/1.1\r\n\r\n";
  let expected_response = b"get"; // mismo handler
  run_test(request, expected_response);
}

#[test]
fn test_get_no_content_length() {
  setup_test_server(regular_server_definition);
  run_test(b"GET /test HTTP/1.1\r\n\r\n", b"get");
}

#[test]
fn test_get_with_content_length_matching_body() {
  setup_test_server(regular_server_definition);
  run_test(b"GET /test HTTP/1.1\r\nContent-Length: 4\r\n\r\nping", b"get");
}

#[test]
fn test_get_with_content_length_smaller_than_body() {
  setup_test_server(regular_server_definition);
  run_test(b"GET /test HTTP/1.1\r\nContent-Length: 1\r\n\r\npong", b"get");
}

#[test]
fn test_get_with_content_length_larger_than_body() {
  setup_test_server(regular_server_definition);
  run_test(b"GET /test HTTP/1.1\r\nContent-Length: 10\r\n\r\nhi", b"get");
}

#[test]
fn test_post() {
  setup_test_server(regular_server_definition);
  let request = b"POST /test HTTP/1.1\r\n\r\nmueve tu cuerpo";
  let expected_response = b"Method: POST\nUri: /test\nParams: {}\nBody: \"mueve tu cuerpo\"";
  run_test(request, expected_response);
}

#[test]
fn test_post_with_query() {
  setup_test_server(regular_server_definition);
  let request = b"POST /test?foo=bar HTTP/1.1\r\n\r\nmueve tu cuerpo";
  let expected_response = b"Method: POST\nUri: /test\nParams: {\"foo\": \"bar\"}\nBody: \"mueve tu cuerpo\"";
  run_test(request, expected_response);
}

#[test]
fn test_post_with_content_length() {
  setup_test_server(regular_server_definition);
  let request = b"POST /test HTTP/1.1\r\nContent-Length: 15\r\n\r\nmueve tu cuerpo";
  let expected_response = b"Method: POST\nUri: /test\nParams: {}\nBody: \"mueve tu cuerpo\"";
  run_test(request, expected_response);
}

#[test]
fn test_post_with_params() {
  setup_test_server(regular_server_definition);
  let request = b"POST /test/hola/que?param4=hoy&param3=hace HTTP/1.1\r\n\r\nmueve tu cuerpo";
  let expected_response = b"Method: POST\n\
      Uri: /test/hola/que\n\
      Params: {\"param1\": \"hola\", \"param2\": \"que\", \"param3\": \"hace\", \"param4\": \"hoy\"}\n\
      Body: \"mueve tu cuerpo\"";
  run_test(request, expected_response);
}

#[test]
fn test_post_with_incomplete_path_params() {
  setup_test_server(regular_server_definition);
  let request = b"POST /test/hola HTTP/1.1\r\n\r\nmueve tu cuerpo";
  let expected_response = b"Method: POST\nUri: /test/hola\nParams: {\"param1\": \"hola\"}\nBody: \"mueve tu cuerpo\"";
  run_test(request, expected_response);
}

#[test]
fn test_post_content_length_variations() {
  setup_test_server(regular_server_definition);
  let expected_full_body = b"Method: POST\nUri: /test\nParams: {}\nBody: \"body\"";
  run_test(b"POST /test HTTP/1.1\r\n\r\nbody", expected_full_body);
  run_test(
    b"POST /test HTTP/1.1\r\nContent-Length: 4\r\n\r\nbody",
    expected_full_body,
  );
  run_test(
    b"POST /test HTTP/1.1\r\nContent-Length: 2\r\n\r\nbody",
    b"Method: POST\nUri: /test\nParams: {}\nBody: \"bo\"",
  );
  run_test(
    b"POST /test HTTP/1.1\r\nContent-Length: 10\r\n\r\nbody",
    b"HTTP/1.1 200 OK",
  );
}

#[test]
fn test_put() {
  setup_test_server(regular_server_definition);
  let request = b"PUT /test HTTP/1.1\r\n\r\nmueve tu cuerpo";
  let expected_response = b"Method: PUT\nUri: /test\nParams: {}\nBody: \"mueve tu cuerpo\"";
  run_test(request, expected_response);
}

#[test]
fn test_put_content_length_variations() {
  setup_test_server(regular_server_definition);
  let expected_full_body = b"Method: PUT\nUri: /test\nParams: {}\nBody: \"put\"";
  run_test(b"PUT /test HTTP/1.1\r\n\r\nput", expected_full_body);
  run_test(
    b"PUT /test HTTP/1.1\r\nContent-Length: 3\r\n\r\nput",
    expected_full_body,
  );
  run_test(
    b"PUT /test HTTP/1.1\r\nContent-Length: 1\r\n\r\nput",
    b"Method: PUT\nUri: /test\nParams: {}\nBody: \"p\"",
  );
  run_test(
    b"PUT /test HTTP/1.1\r\nContent-Length: 8\r\n\r\nput",
    b"HTTP/1.1 200 OK",
  );
}

#[test]
fn test_delete() {
  setup_test_server(regular_server_definition);
  let request = b"DELETE /test HTTP/1.1\r\n\r\n";
  let expected_response = b"delete";
  run_test(request, expected_response);
}

#[test]
fn test_delete_no_content_length() {
  setup_test_server(regular_server_definition);
  run_test(b"DELETE /test HTTP/1.1\r\n\r\n", b"delete");
}

#[test]
fn test_delete_with_content_length_matching_body() {
  setup_test_server(regular_server_definition);
  run_test(b"DELETE /test HTTP/1.1\r\nContent-Length: 4\r\n\r\nping", b"delete");
}

#[test]
fn test_delete_with_content_length_smaller_than_body() {
  setup_test_server(regular_server_definition);
  run_test(b"DELETE /test HTTP/1.1\r\nContent-Length: 1\r\n\r\nping", b"delete");
}

#[test]
fn test_delete_with_content_length_larger_than_body() {
  setup_test_server(regular_server_definition);
  run_test(b"DELETE /test HTTP/1.1\r\nContent-Length: 20\r\n\r\nping", b"delete");
}

#[test]
fn test_strict_mode_without_content_length() {
  let addr = strict_server_definition();
  let request = b"POST /test HTTP/1.1\r\n\r\npayload";
  run_test(&addr, request, b"HTTP/1.1 411 Length Required");
}

#[test]
fn test_strict_mode_with_content_length() {
  let addr = strict_server_definition();
  let request = b"POST /test HTTP/1.1\r\nContent-Length: 7\r\n\r\npayload";
  run_test(
    &addr,
    request,
    b"Method: POST\nUri: /test\nParams: {}\nBody: \"payload\"",
  );
}

#[test]
fn test_strict_mode_get_without_content_length() {
  let addr = strict_server_definition();
  let request = b"GET /test HTTP/1.1\r\n\r\n";
  run_test(&addr, request, b"get");
}

#[test]
fn test_file_exists() {
  setup_test_server(regular_server_definition);
  let request = b"GET /numano.png HTTP/1.1\r\nHost: localhost\r\n\r\n";
  let expected_response = b"HTTP/1.1 200 OK";
  run_test(request, expected_response);
}

#[test]
fn test_file_not_found() {
  setup_test_server(regular_server_definition);
  let request = b"GET /test.png HTTP/1.1\r\n\r\n";
  let expected_response = b"HTTP/1.1 404 Not Found";
  run_test(request, expected_response);
}

#[test]
fn test_method_not_allowed() {
  setup_test_server(regular_server_definition);
  let request = b"BREW /coffee HTTP/1.1\r\n\r\n";
  let expected_response = b"HTTP/1.1 405 Method Not Allowed";
  run_test(request, expected_response);
}

#[test]
fn test_empty_request() {
  setup_test_server(regular_server_definition);
  let request = b"";
  let expected_response = b"HTTP/1.1 400 Bad Request";
  run_test(request, expected_response);
}

#[test]
fn test_malformed_request() {
  setup_test_server(regular_server_definition);
  let request = b"THIS_IS_NOT_HTTP\r\n\r\n";
  let expected_response = b"HTTP/1.1 400 Bad Request";
  run_test(request, expected_response);
}

#[test]
fn test_unsupported_http_version() {
  setup_test_server(regular_server_definition);
  let request = b"GET / HTTP/0.9\r\n\r\n";
  let expected_response = b"HTTP/1.1 505 HTTP Version Not Supported";
  run_test(request, expected_response);
}

#[test]
fn test_long_path() {
  setup_test_server(regular_server_definition);
  let long_path = "/".to_string() + &"a".repeat(10_000);
  let request = format!("GET {} HTTP/1.1\r\n\r\n", long_path);
  let expected_response = b"HTTP/1.1 414 URI Too Long";
  run_test(request.as_bytes(), expected_response);
}

#[test]
fn test_missing_method() {
  setup_test_server(regular_server_definition);
  let request = b"/ HTTP/1.1\r\n\r\n";
  let expected_response = b"HTTP/1.1 400 Bad Request";
  run_test(request, expected_response);
}
