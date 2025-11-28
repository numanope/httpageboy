use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
#[cfg(any(
  feature = "sync",
  feature = "async_tokio",
  feature = "async_smol",
  feature = "async_std"
))]
use std::sync::Once;
use std::sync::OnceLock;
#[allow(unused_imports)]
use std::thread;
use std::time::Duration;

#[cfg(feature = "sync")]
use crate::runtime::sync::server::Server;

#[cfg(all(feature = "async_tokio", not(feature = "sync")))]
use crate::runtime::r#async::tokio::Server;

#[cfg(all(feature = "async_smol", not(any(feature = "sync", feature = "async_tokio"))))]
use crate::runtime::r#async::smol::Server;

#[cfg(all(
  feature = "async_std",
  not(any(feature = "sync", feature = "async_tokio", feature = "async_smol"))
))]
use crate::runtime::r#async::async_std::Server;

pub const POOL_SIZE: u8 = 10;
pub const INTERVAL: Duration = Duration::from_millis(250);
static SERVER_ADDR: OnceLock<String> = OnceLock::new();
pub const SERVER_URL: &str = "127.0.0.1:0";
#[cfg(any(
  feature = "sync",
  feature = "async_tokio",
  feature = "async_smol",
  feature = "async_std"
))]
static INIT: Once = Once::new();

fn compute_server_addr() -> String {
  let listener = TcpListener::bind(SERVER_URL).expect("failed to reserve a loopback port for the test server");
  let addr = listener.local_addr().expect("failed to read reserved loopback address");
  addr.to_string()
}

pub fn active_server_url() -> &'static str {
  SERVER_ADDR.get_or_init(compute_server_addr).as_str()
}

#[cfg(feature = "sync")]
pub fn setup_test_server<F>(server_factory: F)
where
  F: FnOnce() -> Server + Send + 'static,
{
  INIT.call_once(|| {
    active_server_url();
    let server = server_factory();
    thread::spawn(move || {
      server.run();
    });
    thread::sleep(INTERVAL);
  });
}

// async_tokio
#[cfg(all(feature = "async_tokio", not(feature = "sync")))]
pub async fn setup_test_server<F, Fut>(server_factory: F)
where
  F: FnOnce() -> Fut + Send + 'static,
  Fut: std::future::Future<Output = Server> + Send + 'static,
{
  INIT.call_once(|| {
    active_server_url();
    thread::spawn(move || {
      // Arranca un runtime Tokio en este hilo
      let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
      rt.block_on(async move {
        let server = server_factory().await;
        server.run().await;
      });
    });
    thread::sleep(INTERVAL);
  });
}

// async_std
#[cfg(all(feature = "async_std", not(any(feature = "sync", feature = "async_tokio"))))]
pub async fn setup_test_server<F, Fut>(server_factory: F)
where
  F: FnOnce() -> Fut + Send + 'static,
  Fut: std::future::Future<Output = Server> + Send + 'static,
{
  INIT.call_once(|| {
    active_server_url();
    thread::spawn(move || {
      // Arranca async-std en este hilo
      async_std::task::block_on(async move {
        let server = server_factory().await;
        server.run().await;
      });
    });
    thread::sleep(INTERVAL);
  });
}

#[cfg(all(
  feature = "async_smol",
  not(any(feature = "sync", feature = "async_tokio", feature = "async_std"))
))]
pub async fn setup_test_server<F, Fut>(server_factory: F)
where
  F: FnOnce() -> Fut + Send + 'static,
  Fut: std::future::Future<Output = Server> + Send + 'static,
{
  INIT.call_once(|| {
    active_server_url();
    thread::spawn(move || {
      smol::block_on(async move {
        let server = server_factory().await;
        server.run().await;
      });
    });
    thread::sleep(INTERVAL);
  });
}

pub fn run_test(request: &[u8], expected_response: &[u8]) -> String {
  let mut stream = TcpStream::connect(active_server_url()).expect("Failed to connect to test server");

  stream.write_all(request).unwrap();
  stream.shutdown(std::net::Shutdown::Write).unwrap();

  let mut buffer = Vec::new();
  stream.read_to_end(&mut buffer).unwrap();

  let buffer_string = String::from_utf8_lossy(&buffer).to_string();
  let expected_response_string = String::from_utf8_lossy(expected_response).to_string();

  assert!(
    buffer_string.contains(&expected_response_string),
    "ASSERT FAILED:\n\nRECEIVED: {} \nEXPECTED: {} \n\n",
    buffer_string,
    expected_response_string
  );
  buffer_string
}
