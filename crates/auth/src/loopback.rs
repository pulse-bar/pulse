use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::timeout;

use crate::error::{AuthError, AuthResult};

// Pulse's OAuth callback ports. Atlassian (and most providers) require
// the redirect URI to be pre-registered, so the developer registers all
// three with their OAuth app — Pulse picks the first that's free at
// runtime.
pub const LOOPBACK_PORTS: &[u16] = &[19_834, 19_835, 19_836];
pub const LOOPBACK_PATH: &str = "/callback";
pub const LOOPBACK_TIMEOUT_SECS: u64 = 180;

pub struct LoopbackServer {
    listener: TcpListener,
    port: u16,
}

impl LoopbackServer {
    pub async fn bind() -> AuthResult<Self> {
        let mut last_err: Option<std::io::Error> = None;
        for &port in LOOPBACK_PORTS {
            let addr: SocketAddr = ([127, 0, 0, 1], port).into();
            match TcpListener::bind(addr).await {
                Ok(listener) => return Ok(Self { listener, port }),
                Err(e) => last_err = Some(e),
            }
        }
        Err(AuthError::OAuth(format!(
            "could not bind any of {:?}: {}",
            LOOPBACK_PORTS,
            last_err.map(|e| e.to_string()).unwrap_or_default()
        )))
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn redirect_uri(&self) -> String {
        format!("http://127.0.0.1:{}{LOOPBACK_PATH}", self.port)
    }

    pub async fn await_callback(self, expected_state: &str) -> AuthResult<String> {
        let accept = self.listener.accept();
        let (mut socket, _) = timeout(Duration::from_secs(LOOPBACK_TIMEOUT_SECS), accept)
            .await
            .map_err(|_| AuthError::OAuth("OAuth callback timed out".into()))?
            .map_err(|e| AuthError::OAuth(format!("accept: {e}")))?;

        let mut buf = vec![0u8; 8192];
        let n = socket
            .read(&mut buf)
            .await
            .map_err(|e| AuthError::OAuth(format!("read: {e}")))?;
        let request = String::from_utf8_lossy(&buf[..n]);

        let result = parse_callback(&request, expected_state);
        let body = render_response(&result);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.as_bytes().len()
        );
        let _ = socket.write_all(response.as_bytes()).await;
        let _ = socket.shutdown().await;
        result
    }
}

fn parse_callback(request: &str, expected_state: &str) -> AuthResult<String> {
    let first_line = request.lines().next().unwrap_or("");
    let path_with_query = first_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| AuthError::OAuth("malformed request".into()))?;

    let (path, query) = match path_with_query.split_once('?') {
        Some((p, q)) => (p, q),
        None => (path_with_query, ""),
    };

    if !path.starts_with(LOOPBACK_PATH) {
        return Err(AuthError::OAuth(format!("unexpected path: {path}")));
    }

    let mut code: Option<String> = None;
    let mut state: Option<String> = None;
    let mut error: Option<String> = None;
    let mut error_description: Option<String> = None;

    for kv in query.split('&') {
        if let Some((k, v)) = kv.split_once('=') {
            let v = url_decode(v);
            match k {
                "code" => code = Some(v),
                "state" => state = Some(v),
                "error" => error = Some(v),
                "error_description" => error_description = Some(v),
                _ => {}
            }
        }
    }

    if let Some(err) = error {
        let desc = error_description.unwrap_or_default();
        return Err(AuthError::OAuth(format!("provider returned: {err} {desc}")));
    }

    let code = code.ok_or_else(|| AuthError::OAuth("missing 'code' in callback".into()))?;
    let state = state.ok_or_else(|| AuthError::OAuth("missing 'state' in callback".into()))?;
    if state != expected_state {
        return Err(AuthError::OAuth(format!(
            "state mismatch — expected {expected_state}, got {state}"
        )));
    }
    Ok(code)
}

fn url_decode(s: &str) -> String {
    let s = s.replace('+', " ");
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_digit(bytes[i + 1]), hex_digit(bytes[i + 2])) {
                out.push((hi << 4) | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn render_response(result: &AuthResult<String>) -> String {
    let ok = result.is_ok();
    let title = if ok { "Pulse is connected" } else { "Pulse couldn't complete the connection" };
    let body_text = match result {
        Ok(_) => "You can close this tab and return to Pulse.".to_string(),
        Err(e) => format!("Reason: {e}. The error has been reported back to Pulse — try again."),
    };
    let accent = if ok { "#3DD68C" } else { "#FF6B6B" };
    let badge = if ok { "✓" } else { "!" };
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8" />
<meta name="viewport" content="width=device-width, initial-scale=1" />
<title>{title}</title>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; margin: 0; min-height: 100vh; display: grid; place-items: center; background: #14141F; color: #E2E2F0; }}
  .card {{ background: rgba(255,255,255,0.04); border: 1px solid rgba(255,255,255,0.08); padding: 36px 44px; border-radius: 14px; text-align: center; max-width: 420px; box-shadow: 0 20px 60px rgba(0,0,0,0.5); }}
  .badge {{ display: inline-flex; width: 56px; height: 56px; border-radius: 50%; background: {accent}22; border: 2px solid {accent}; align-items: center; justify-content: center; margin-bottom: 16px; color: {accent}; font-size: 28px; font-weight: 700; }}
  h1 {{ margin: 0 0 10px; font-size: 18px; font-weight: 600; letter-spacing: -0.01em; }}
  p {{ margin: 0; font-size: 13px; color: rgba(255,255,255,0.65); line-height: 1.5; }}
  .brand {{ margin-top: 22px; font-size: 11px; color: rgba(255,255,255,0.35); letter-spacing: 0.1em; text-transform: uppercase; }}
</style>
</head>
<body>
  <div class="card">
    <div class="badge">{badge}</div>
    <h1>{title}</h1>
    <p>{body_text}</p>
    <div class="brand">Pulse · pulse-bar</div>
  </div>
</body>
</html>"#
    )
}
