#![cfg(feature = "sync")]

use crate::core::handler::Handler;
use crate::core::request::{Request, handle_request_sync};
use crate::core::request_handler::Rh;
use crate::core::request_type::Rt;
use crate::core::response::Response;
use crate::runtime::shared::print_server_info;
use crate::runtime::sync::threadpool::ThreadPool;
use std::collections::HashMap;
use std::io::prelude::Write;
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Server {
  url: &str,
  listener: TcpListener,
  pool: Arc<Mutex<ThreadPool>>,
  routes: HashMap<(Rt, String), Rh>,
  files_sources: Vec<String>,
  auto_close: bool,
}

impl Server {
  pub fn new(
    serving_url: &str,
    pool_size: u8,
    routes_list: Option<HashMap<(Rt, String), Rh>>,
  ) -> Result<Server, std::io::Error> {
    let listener = TcpListener::bind(serving_url)?;
    let pool = Arc::new(Mutex::new(ThreadPool::new(pool_size as usize)));
    let routes = routes_list.unwrap_or_default();

    Ok(Server {
      listener,
      pool,
      routes,
      files_sources: Vec::new(),
      auto_close: true,
    })
  }

  pub fn set_auto_close(&mut self, state: bool) {
    self.auto_close = state;
  }

  pub fn local_addr(&self) -> std::io::Result<SocketAddr> {
    self.listener.local_addr()
  }

  pub fn add_route(&mut self, path: &str, rt: Rt, handler: Arc<dyn Handler>) {
    let key = (rt, path.to_string());
    self.routes.insert(key, Rh { handler });
  }

  pub fn add_files_source<S>(&mut self, base: S)
  where
    S: Into<String>,
  {
    let s = base.into();
    let canonical = PathBuf::from(&s)
      .canonicalize()
      .map(|p| p.to_string_lossy().to_string())
      .unwrap_or(s.clone());
    self.files_sources.push(canonical);
  }

  pub fn run(&self) {
    print_server_info(self.listener.local_addr().unwrap(), self.auto_close);
    for stream in self.listener.incoming() {
      match stream {
        Ok(stream) => {
          let routes_local = self.routes.clone();
          let sources_local = self.files_sources.clone();
          let close_flag = self.auto_close;
          let pool = Arc::clone(&self.pool);
          pool.lock().unwrap().run(move || {
            let (mut request, early_resp) = Request::parse_stream_sync(&stream, &routes_local, &sources_local);
            let answer = if let Some(resp) = early_resp {
              Some(resp)
            } else {
              handle_request_sync(&mut request, &routes_local, &sources_local)
            };
            match answer {
              Some(response) => Self::send_response(stream, &response, close_flag),
              None => Self::send_response(stream, &Response::new(), close_flag),
            }
          });
        }
        Err(_err) => {
          // could log error here
        }
      }
    }
  }

  pub fn stop(&self) {
    let mut pool = self.pool.lock().unwrap();
    pool.stop();
  }

  fn send_response(mut stream: TcpStream, response: &Response, close: bool) {
    let connection_header = if close { "Connection: close\r\n" } else { "" };
    let header = format!(
      "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n{}\r\n",
      response.status,
      response.content_type,
      response.content.len(),
      connection_header
    );
    let _ = stream.write_all(header.as_bytes());

    if response.content_type.starts_with("image/") {
      let _ = stream.write_all(&response.content);
    } else {
      let text = String::from_utf8_lossy(&response.content);
      let _ = stream.write_all(text.as_bytes());
    }

    let _ = stream.flush();
    if close {
      let _ = stream.shutdown(Shutdown::Both);
    }
  }
}
