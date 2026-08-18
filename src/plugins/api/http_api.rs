//! Pure-Rust HTTP client API for Rhai scripts with timeouts and memory limits.

use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Maximum response body size captured in bytes (5 MB).
pub const MAX_HTTP_RESPONSE_BYTES: usize = 5_242_880;
/// Default timeout for HTTP operations (3 seconds to minimize UI latency).
pub const HTTP_TIMEOUT_SECONDS: u64 = 3;

/// Internal state for tracking HTTP request execution and results.
#[derive(Debug, Clone, Default)]
pub struct HttpHandleInner {
    pub last_status_code: i64,
    pub last_error: String,
}

/// HTTP client proxy object exposed to Rhai scripts as `http`.
#[derive(Debug, Clone, Default)]
pub struct HttpHandle {
    inner: Arc<Mutex<HttpHandleInner>>,
}

impl HttpHandle {
    /// Creates a new HTTP handle instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the HTTP status code of the most recent request (or 0 if failed).
    pub fn status_code(&mut self) -> i64 {
        self.inner.lock().map(|i| i.last_status_code).unwrap_or(0)
    }

    /// Returns the last error message, if any.
    pub fn last_error(&mut self) -> String {
        self.inner
            .lock()
            .map(|i| i.last_error.clone())
            .unwrap_or_default()
    }

    /// Performs an HTTP GET request and returns the response body as a string.
    pub fn get(&mut self, url: String) -> String {
        self.get_with_headers(url, rhai::Map::new())
    }

    /// Performs an HTTP GET request with custom headers.
    pub fn get_with_headers(&mut self, url: String, headers: rhai::Map) -> String {
        let trimmed_url = url.trim();
        if !trimmed_url.starts_with("http://") && !trimmed_url.starts_with("https://") {
            self.set_error("Invalid URL schema: must start with http:// or https://");
            return String::new();
        }

        let client = match reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECONDS))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                self.set_error(format!("Failed to build HTTP client: {}", e));
                return String::new();
            }
        };

        let mut req = client.get(trimmed_url);
        for (k, v) in headers {
            if let Ok(val_str) = v.into_string() {
                req = req.header(k.as_str(), val_str);
            }
        }

        match req.send() {
            Ok(mut response) => {
                let status = response.status().as_u16() as i64;
                self.set_status(status);
                use std::io::Read;
                let mut b = Vec::new();
                match response
                    .by_ref()
                    .take(MAX_HTTP_RESPONSE_BYTES as u64)
                    .read_to_end(&mut b)
                {
                    Ok(_) => String::from_utf8_lossy(&b).to_string(),
                    Err(e) => {
                        self.set_error(format!("Failed to read response body: {}", e));
                        String::new()
                    }
                }
            }
            Err(e) => {
                self.set_error(format!("HTTP GET failed: {}", e));
                String::new()
            }
        }
    }

    /// Performs an HTTP POST request with a raw text body.
    pub fn post(&mut self, url: String, body: String) -> String {
        self.post_json(url, body, rhai::Map::new())
    }

    /// Performs an HTTP POST request with a JSON/raw body and custom headers.
    pub fn post_json(&mut self, url: String, body: String, headers: rhai::Map) -> String {
        let trimmed_url = url.trim();
        if !trimmed_url.starts_with("http://") && !trimmed_url.starts_with("https://") {
            self.set_error("Invalid URL schema: must start with http:// or https://");
            return String::new();
        }

        let client = match reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(HTTP_TIMEOUT_SECONDS))
            .build()
        {
            Ok(c) => c,
            Err(e) => {
                self.set_error(format!("Failed to build HTTP client: {}", e));
                return String::new();
            }
        };

        let mut req = client.post(trimmed_url).body(body);
        let mut has_content_type = false;

        for (k, v) in headers {
            if k.eq_ignore_ascii_case("content-type") {
                has_content_type = true;
            }
            if let Ok(val_str) = v.into_string() {
                req = req.header(k.as_str(), val_str);
            }
        }

        if !has_content_type {
            req = req.header("Content-Type", "application/json");
        }

        match req.send() {
            Ok(mut response) => {
                let status = response.status().as_u16() as i64;
                self.set_status(status);
                use std::io::Read;
                let mut b = Vec::new();
                match response
                    .by_ref()
                    .take(MAX_HTTP_RESPONSE_BYTES as u64)
                    .read_to_end(&mut b)
                {
                    Ok(_) => String::from_utf8_lossy(&b).to_string(),
                    Err(e) => {
                        self.set_error(format!("Failed to read response body: {}", e));
                        String::new()
                    }
                }
            }
            Err(e) => {
                self.set_error(format!("HTTP POST failed: {}", e));
                String::new()
            }
        }
    }

    fn set_status(&mut self, code: i64) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.last_status_code = code;
            inner.last_error.clear();
        }
    }

    fn set_error(&mut self, err: impl Into<String>) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.last_status_code = 0;
            inner.last_error = err.into();
        }
    }
}
