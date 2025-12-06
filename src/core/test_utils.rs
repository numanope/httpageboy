use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
#[cfg(any(feature = "async_tokio", feature = "async_std", feature = "async_smol"))]
use std::sync::mpsc;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

static LAST_ACTIVE_URL: OnceLock<Mutex<Option<&'static str>>> = OnceLock::new();

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
static SERVER_REGISTRY: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();

thread_local! {
  static ACTIVE_SERVER_URL: RefCell<Option<&'static str>> = RefCell::new(None);
}

fn server_registry() -> &'static Mutex<HashMap<String, &'static str>> {
  SERVER_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn registry_guard() -> std::sync::MutexGuard<'static, HashMap<String, &'static str>> {
  server_registry()
    .lock()
    .unwrap_or_else(|err| err.into_inner())
}

fn set_active_url(url: &'static str) {
  ACTIVE_SERVER_URL.with(|slot| {
    *slot.borrow_mut() = Some(url);
  });
  LAST_ACTIVE_URL
    .get_or_init(|| Mutex::new(None))
    .lock()
    .unwrap_or_else(|err| err.into_inner())
    .replace(url);
}

fn active_server_url() -> &'static str {
  if let Some(url) = ACTIVE_SERVER_URL.with(|slot| slot.borrow().clone()) {
    return url;
  }

  let cached = LAST_ACTIVE_URL
    .get_or_init(|| Mutex::new(None))
    .lock()
    .unwrap_or_else(|err| err.into_inner())
    .clone();

  if let Some(url) = cached {
    set_active_url(url);
    return url;
  }

  let fallback = registry_guard().values().next().copied();
  if let Some(url) = fallback {
    set_active_url(url);
    return url;
  }

  panic!("setup_test_server must be called before run_test");
}

#[cfg(feature = "sync")]
pub fn setup_test_server<F>(server_url: &str, server_factory: F)
where
  F: FnOnce() -> Server + Send + 'static,
{
  let mut registry = registry_guard();
  if let Some(url) = registry.get(server_url) {
    set_active_url(*url);
    return;
  }

  let server = server_factory();
  let leaked_url: &'static str = Box::leak(server.url().to_owned().into_boxed_str());
  registry.insert(server_url.to_string(), leaked_url);
  let active_url = leaked_url;
  drop(registry);

  thread::spawn(move || {
    server.run();
  });
  thread::sleep(INTERVAL);
  set_active_url(active_url);
}

// async_tokio
#[cfg(all(feature = "async_tokio", not(feature = "sync")))]
pub async fn setup_test_server<F, Fut>(server_url: &str, server_factory: F)
where
  F: FnOnce() -> Fut + Send + 'static,
  Fut: std::future::Future<Output = Server> + Send + 'static,
{
  let mut registry = registry_guard();
  if let Some(url) = registry.get(server_url) {
    set_active_url(*url);
    return;
  }

  let (tx, rx) = mpsc::channel();
  thread::spawn(move || {
    let rt = tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()
      .unwrap();
    rt.block_on(async move {
      let server = server_factory().await;
      let leaked_url: &'static str = Box::leak(server.url().to_owned().into_boxed_str());
      let _ = tx.send(leaked_url);
      server.run().await;
    });
  });

  let leaked_url = rx.recv().expect("server url not sent");
  registry.insert(server_url.to_string(), leaked_url);
  drop(registry);
  thread::sleep(INTERVAL);
  set_active_url(leaked_url);
}

// async_std
#[cfg(all(feature = "async_std", not(any(feature = "sync", feature = "async_tokio"))))]
pub async fn setup_test_server<F, Fut>(server_url: &str, server_factory: F)
where
  F: FnOnce() -> Fut + Send + 'static,
  Fut: std::future::Future<Output = Server> + Send + 'static,
{
  let mut registry = registry_guard();
  if let Some(url) = registry.get(server_url) {
    set_active_url(*url);
    return;
  }

  let (tx, rx) = mpsc::channel();
  thread::spawn(move || {
    async_std::task::block_on(async move {
      let server = server_factory().await;
      let leaked_url: &'static str = Box::leak(server.url().to_owned().into_boxed_str());
      let _ = tx.send(leaked_url);
      server.run().await;
    });
  });

  let leaked_url = rx.recv().expect("server url not sent");
  registry.insert(server_url.to_string(), leaked_url);
  drop(registry);
  thread::sleep(INTERVAL);
  set_active_url(leaked_url);
}

#[cfg(all(
  feature = "async_smol",
  not(any(feature = "sync", feature = "async_tokio", feature = "async_std"))
))]
pub async fn setup_test_server<F, Fut>(server_url: &str, server_factory: F)
where
  F: FnOnce() -> Fut + Send + 'static,
  Fut: std::future::Future<Output = Server> + Send + 'static,
{
  let mut registry = registry_guard();
  if let Some(url) = registry.get(server_url) {
    set_active_url(*url);
    return;
  }

  let (tx, rx) = mpsc::channel();
  thread::spawn(move || {
    smol::block_on(async move {
      let server = server_factory().await;
      let leaked_url: &'static str = Box::leak(server.url().to_owned().into_boxed_str());
      let _ = tx.send(leaked_url);
      server.run().await;
    });
  });

  let leaked_url = rx.recv().expect("server url not sent");
  registry.insert(server_url.to_string(), leaked_url);
  drop(registry);
  thread::sleep(INTERVAL);
  set_active_url(leaked_url);
}

pub fn run_test(request: &[u8], expected_response: &[u8]) -> String {
  let mut stream = TcpStream::connect(active_server_url()).expect("Failed to connect to server");

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
