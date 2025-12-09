use crate::core::response::Response;
use crate::core::status_code::StatusCode;

#[derive(Clone, Debug)]
pub struct CorsPolicy {
  pub allow_origin: String,
  pub allow_methods: String,
  pub allow_headers: String,
  pub allow_credentials: bool,
  pub max_age_seconds: Option<u32>,
}

impl Default for CorsPolicy {
  fn default() -> Self {
    CorsPolicy {
      allow_origin: "*".to_string(),
      allow_methods: "GET,POST,PUT,DELETE,OPTIONS".to_string(),
      allow_headers: "Content-Type, Authorization".to_string(),
      allow_credentials: false,
      max_age_seconds: Some(600),
    }
  }
}

impl CorsPolicy {
  /// Parses a comma-separated config string (e.g. "origin=http://app,credentials=true,headers=Content-Type")
  /// and returns a policy. Unknown keys are ignored.
  pub fn from_config_str(config: &str) -> Self {
    let mut policy = CorsPolicy::default();
    for pair in config.split(',') {
      let mut parts = pair.splitn(2, '=');
      let key = parts.next().unwrap_or("").trim().to_ascii_lowercase();
      let value = parts.next().unwrap_or("").trim();
      match key.as_str() {
        "origin" | "allow_origin" => policy.allow_origin = value.to_string(),
        "methods" | "allow_methods" => policy.allow_methods = value.to_string(),
        "headers" | "allow_headers" => policy.allow_headers = value.to_string(),
        "credentials" | "allow_credentials" => {
          policy.allow_credentials = value.eq_ignore_ascii_case("true")
        }
        "max_age" | "max_age_seconds" => {
          policy.max_age_seconds = value.parse::<u32>().ok();
        }
        _ => {}
      }
    }
    policy
  }

  pub fn header_lines(&self) -> Vec<(String, String)> {
    let mut headers = vec![
      ("Access-Control-Allow-Origin".to_string(), self.allow_origin.clone()),
      (
        "Access-Control-Allow-Methods".to_string(),
        self.allow_methods.clone(),
      ),
      (
        "Access-Control-Allow-Headers".to_string(),
        self.allow_headers.clone(),
      ),
    ];
    if self.allow_credentials {
      headers.push((
        "Access-Control-Allow-Credentials".to_string(),
        "true".to_string(),
      ));
    }
    if let Some(max_age) = self.max_age_seconds {
      headers.push((
        "Access-Control-Max-Age".to_string(),
        max_age.to_string(),
      ));
    }
    headers
  }

  pub fn preflight_response(&self) -> Response {
    Response {
      status: StatusCode::NoContent.to_string(),
      content_type: "text/plain".to_string(),
      content: Vec::new(),
    }
  }
}
