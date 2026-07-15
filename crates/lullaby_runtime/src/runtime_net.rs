//! Networking and external-process runtime support shared by every interpreter
//! backend: the socket/process resource handles, the blocking HTTP exchange, and
//! the process-exit-code helper. Split out of `lib.rs` as a behavior-preserving
//! code move; `Value` and `result_value` (in `lib.rs`) are reached through
//! `crate::` paths.

use std::net::{TcpListener, TcpStream, UdpSocket};
use std::process::Child;

use crate::{Value, result_value};

/// Build an `err(string)` result whose payload is the display form of an I/O or
/// network error. Network builtins report failures as runtime `result` values
/// rather than propagating a `RuntimeError`.
pub fn net_err(error: &std::io::Error) -> Value {
    result_value(Err(Value::String((error.to_string()).into())))
}

/// Perform one HTTP/1.1 exchange over a fresh `TcpStream` and return the
/// response body as a `result<string, string>` runtime value.
///
/// `method` is `"GET"` or `"POST"`; `body` is `None` for GET and `Some(text)`
/// for POST (sent as `Content-Type: text/plain`). Only the `http` scheme is
/// supported — an `https://` URL yields `err("https not supported")`. Chunked
/// transfer decoding is not implemented; the response body is read to EOF via
/// the `Connection: close` header. A read timeout keeps a hung server from
/// stalling the caller. A 2xx/3xx status yields `ok(body)`; any 4xx/5xx status
/// yields `err("http {code}: {first-body-line}")`. All connection/parse/HTTP
/// failures are `err(...)` results, never a propagated `RuntimeError`.
pub fn http_exchange(method: &str, url: &str, body: Option<&str>) -> Value {
    use std::io::{Read, Write};
    use std::time::Duration;

    let (scheme, rest) = match url.split_once("://") {
        Some(parts) => parts,
        None => return result_value(Err(Value::String(("invalid url".to_string()).into()))),
    };
    if scheme.eq_ignore_ascii_case("https") {
        return result_value(Err(Value::String(
            ("https not supported".to_string()).into(),
        )));
    }
    if !scheme.eq_ignore_ascii_case("http") {
        return result_value(Err(Value::String(
            format!("unsupported scheme `{scheme}`").into(),
        )));
    }

    // Split `host[:port]` from the path (default `/`).
    let (authority, path) = match rest.find('/') {
        Some(index) => (&rest[..index], &rest[index..]),
        None => (rest, "/"),
    };
    if authority.is_empty() {
        return result_value(Err(Value::String(("missing host".to_string()).into())));
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port_text)) => match port_text.parse::<u16>() {
            Ok(port) => (host, port),
            Err(_) => {
                return result_value(Err(Value::String(
                    format!("invalid port `{port_text}`").into(),
                )));
            }
        },
        None => (authority, 80u16),
    };
    let path = if path.is_empty() { "/" } else { path };

    let mut stream = match TcpStream::connect((host, port)) {
        Ok(stream) => stream,
        Err(error) => return net_err(&error),
    };
    if let Err(error) = stream.set_read_timeout(Some(Duration::from_secs(10))) {
        return net_err(&error);
    }

    let request = match body {
        None => format!(
            "GET {path} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: lullaby\r\nConnection: close\r\n\r\n"
        ),
        Some(body) => format!(
            "{method} {path} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: lullaby\r\nConnection: close\r\nContent-Type: text/plain\r\nContent-Length: {len}\r\n\r\n{body}",
            len = body.len()
        ),
    };
    if let Err(error) = stream.write_all(request.as_bytes()) {
        return net_err(&error);
    }
    if let Err(error) = stream.flush() {
        return net_err(&error);
    }

    let mut response = Vec::new();
    if let Err(error) = stream.read_to_end(&mut response) {
        return net_err(&error);
    }

    let split = response.windows(4).position(|window| window == b"\r\n\r\n");
    let (head, resp_body) = match split {
        Some(index) => (&response[..index], &response[index + 4..]),
        None => {
            return result_value(Err(Value::String(
                ("malformed response: no header terminator".to_string()).into(),
            )));
        }
    };
    let head = String::from_utf8_lossy(head);
    let status_line = head.lines().next().unwrap_or("");
    let code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|token| token.parse::<u16>().ok());
    let body_text = String::from_utf8_lossy(resp_body).into_owned();
    match code {
        Some(code) if (200..400).contains(&code) => {
            result_value(Ok(Value::String((body_text).into())))
        }
        Some(code) => {
            let first_line = body_text.lines().next().unwrap_or("");
            result_value(Err(Value::String(
                format!("http {code}: {first_line}").into(),
            )))
        }
        None => result_value(Err(Value::String(
            format!("malformed status line `{status_line}`").into(),
        ))),
    }
}

/// An open network resource held behind a socket handle. Not `Clone`, which is
/// why sockets are surfaced to Lullaby as opaque integer handles. Shared by the
/// AST interpreter and the IR interpreter so both keep identical socket
/// semantics.
pub enum SocketResource {
    Listener(TcpListener),
    Stream(TcpStream),
    Udp(UdpSocket),
}

/// A live external process held behind a `Value::Process` handle. Not `Clone`
/// (a `std::process::Child` owns OS resources), which is why processes are
/// surfaced to Lullaby as opaque integer handles, exactly like `SocketResource`.
/// Shared by the AST interpreter and the IR interpreter / bytecode VM so every
/// backend keeps identical process semantics. `stdout`/`stderr` are taken out of
/// the `Child` on the first `proc_stdout`/`proc_stderr` read (a `ChildStdout`
/// cannot be read twice), leaving `None` behind so a second read returns EOF.
pub struct ProcessResource {
    pub child: Child,
}

/// Which captured pipe a `proc_stdout`/`proc_stderr` read should drain.
#[derive(Clone, Copy)]
pub(crate) enum PipeKind {
    Stdout,
    Stderr,
}

/// Convert a finished child's exit status into the `i64` a `proc_wait`/`proc_kill`
/// success returns. On every platform a normal exit yields its exit code. On Unix
/// a process killed by a signal has no exit code; by convention that is reported
/// as `128 + signal` (the shell convention), so callers still get a total,
/// deterministic `i64`. Shared by both interpreters so the value is identical
/// across backends.
pub fn process_exit_code(status: &std::process::ExitStatus) -> i64 {
    if let Some(code) = status.code() {
        return i64::from(code);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return 128 + i64::from(signal);
        }
    }
    // No exit code and (on non-Unix) no signal information available.
    -1
}
