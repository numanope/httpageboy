use crate::core::handler::Handler;
use crate::core::request_handler::Rh;
use crate::core::request_type::Rt;
use crate::core::response::Response;
use async_trait::async_trait;
use std::collections::HashMap;
use std::io::Result;
use std::sync::Arc;

/// A trait that abstracts over the different async TCP streams.
/// This allows us to write generic code that can work with any of the supported runtimes.
#[async_trait]
pub trait AsyncStream: Send + Sync {
  async fn write_all(&mut self, buf: &[u8]) -> Result<()>;
  async fn flush(&mut self) -> Result<()>;
  async fn shutdown(&mut self) -> Result<()>;
}

/// Sends a response to the client over the given stream.
pub async fn send_response<S: AsyncStream>(stream: &mut S, resp: &Response, close: bool) {
  let conn_hdr = if close { "Connection: close\r\n" } else { "" };
  let head = format!(
    "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n{}\r\n",
    resp.status,
    resp.content_type,
    resp.content.len(),
    conn_hdr,
  );
  let _ = stream.write_all(head.as_bytes()).await;
  if resp.content_type.starts_with("image/") {
    let _ = stream.write_all(&resp.content).await;
  } else {
    let text = String::from_utf8_lossy(&resp.content);
    let _ = stream.write_all(text.as_bytes()).await;
  }
  let _ = stream.flush().await;
  if close {
    let _ = stream.shutdown().await;
  }
}

/// A generic server implementation that is parameterized over a listener type.
/// This allows us to share the server logic between the different async runtimes.
pub struct GenericServer<L> {
  pub listener: L,
  pub routes: Arc<HashMap<(Rt, String), Rh>>,
  pub files_sources: Arc<Vec<String>>,
  pub auto_close: bool,
  pub body_read_limit_bytes: u64,
  pub body_read_timeout_ms: u64,
  pub strict_content_length: bool,
}

impl<L> GenericServer<L> {
  /// Toggles the `Connection: close` header.
  pub fn set_auto_close(&mut self, active: bool) {
    self.auto_close = active;
  }

  /// Adds a new route to the server.
  pub fn add_route(&mut self, path: &str, rt: Rt, handler: Arc<dyn Handler>) {
    Arc::get_mut(&mut self.routes)
      .unwrap()
      .insert((rt, path.to_string()), Rh { handler });
  }

  /// Adds a new directory to serve static files from.
  pub fn add_files_source<S>(&mut self, base: S)
  where
    S: Into<String>,
  {
    Arc::get_mut(&mut self.files_sources).unwrap().push(base.into());
  }

  pub fn with_body_read_limit(mut self, limit_bytes: u64) -> Self {
    self.body_read_limit_bytes = limit_bytes;
    self
  }

  pub fn with_body_read_timeout(mut self, timeout_ms: u64) -> Self {
    self.body_read_timeout_ms = timeout_ms;
    self
  }

  pub fn with_strict_content_length(mut self, strict: bool) -> Self {
    self.strict_content_length = strict;
    self
  }
}
