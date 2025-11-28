use std::any::type_name;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Mutex, OnceLock};
#[allow(unused_imports)]
use std::thread;
use std::thread_local;
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
type ServerKey = &'static str;
static SERVER_URLS: OnceLock<Mutex<HashMap<ServerKey, &'static str>>> = OnceLock::new();

thread_local! {
  static THREAD_SERVER_URL: RefCell<Option<&'static str>> = RefCell::new(None);
}

#[allow(dead_code)]
fn reserve_loopback_addr() -> String {
  let listener = TcpListener::bind("127.0.0.1:0").expect("failed to reserve a loopback port for the test server");
  let addr = listener.local_addr().expect("failed to read reserved loopback address");
  addr.to_string()
}

fn remember_server_url<F>(key: ServerKey, initializer: F) -> (&'static str, bool)
where
  F: FnOnce() -> String,
{
  let mut registry = SERVER_URLS.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap();
  if let Some(url) = registry.get(key) {
    (*url, false)
  } else {
    let url = initializer();
    let leaked = Box::leak(url.into_boxed_str());
    registry.insert(key, leaked);
    (leaked, true)
  }
}

fn set_active_server_url(url: &'static str) {
  THREAD_SERVER_URL.with(|slot| {
    *slot.borrow_mut() = Some(url);
  });
}

pub fn active_server_url() -> &'static str {
  THREAD_SERVER_URL
    .with(|slot| slot.borrow().clone())
    .expect("test server url not set for this thread")
}

#[cfg(feature = "sync")]
pub fn setup_test_server<F>(server_factory: F)
where
  F: FnOnce() -> Server + Send + 'static,
{
  let key = type_name::<F>();
  let mut pending_server: Option<Server> = None;
  let (url, should_start) = remember_server_url(key, || {
    let server = server_factory();
    let addr = server.local_addr().expect("failed to read bound address for the test server").to_string();
    pending_server = Some(server);
    addr
  });

  set_active_server_url(url);

  if should_start {
    let server = pending_server.expect("expected a server instance to start");
    thread::spawn(move || {
      server.run();
    });
    thread::sleep(INTERVAL);
  }
}

// async_tokio
#[cfg(all(feature = "async_tokio", not(feature = "sync")))]
pub async fn setup_test_server<F, Fut>(server_factory: F)
where
  F: FnOnce() -> Fut + Send + 'static,
  Fut: std::future::Future<Output = Server> + Send + 'static,
{
  let key = type_name::<F>();
  let (url, should_start) = remember_server_url(key, reserve_loopback_addr);

  set_active_server_url(url);

  if should_start {
    let url_for_thread = url;
    thread::spawn(move || {
      set_active_server_url(url_for_thread);
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
  }
}

// async_std
#[cfg(all(feature = "async_std", not(any(feature = "sync", feature = "async_tokio"))))]
pub async fn setup_test_server<F, Fut>(server_factory: F)
where
  F: FnOnce() -> Fut + Send + 'static,
  Fut: std::future::Future<Output = Server> + Send + 'static,
{
  let key = type_name::<F>();
  let (url, should_start) = remember_server_url(key, reserve_loopback_addr);

  set_active_server_url(url);

  if should_start {
    let url_for_thread = url;
    thread::spawn(move || {
      set_active_server_url(url_for_thread);
      async_std::task::block_on(async move {
        let server = server_factory().await;
        server.run().await;
      });
    });
    thread::sleep(INTERVAL);
  }
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
  let key = type_name::<F>();
  let (url, should_start) = remember_server_url(key, reserve_loopback_addr);

  set_active_server_url(url);

  if should_start {
    let url_for_thread = url;
    thread::spawn(move || {
      set_active_server_url(url_for_thread);
      smol::block_on(async move {
        let server = server_factory().await;
        server.run().await;
      });
    });
    thread::sleep(INTERVAL);
  }
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
