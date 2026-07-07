//! JSON-RPC-over-stdio transport: `Content-Length`-framed message framing plus
//! the blocking read/write loop that drives [`crate::handle_message`].
//!
//! The framing is the LSP base protocol: each message is a set of `Header: value`
//! lines terminated by a blank line (`\r\n\r\n`), where `Content-Length` gives
//! the byte length of the JSON body that follows.

use std::io::{self, BufRead, Write};

use serde_json::Value;

use crate::{Message, ServerState, handle_message};

/// Run the server against real stdin/stdout until an `exit` notification is
/// received (or stdin closes).
pub fn run_stdio() -> io::Result<()> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    serve(&mut reader, &mut writer)
}

/// The read/write loop, generic over the streams so it is testable with byte
/// buffers. Reads framed messages, dispatches each to [`handle_message`], and
/// writes every returned message back with framing. Stops when the state's
/// `exit` flag is set or the input ends.
pub fn serve<R: BufRead, W: Write>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    let mut state = ServerState::new();
    // Loop until EOF (the client closed the pipe) or the `exit` flag is set.
    while let Some(message) = read_message(reader)? {
        let method = message
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let id = message.get("id").cloned();
        let params = message
            .get("params")
            .cloned()
            .unwrap_or(Value::Object(Default::default()));

        let outbound = handle_message(&mut state, &method, id, params);
        for out in outbound {
            write_message(writer, out)?;
        }

        if state.should_exit() {
            break;
        }
    }
    Ok(())
}

/// Read one `Content-Length`-framed JSON message, or `None` at end of input.
fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<Value>> {
    let mut content_length: Option<usize> = None;

    // Read headers until a blank line.
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            // EOF before any header.
            return Ok(None);
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some((name, value)) = trimmed.split_once(':')
            && name.trim().eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse().ok();
        }
    }

    let Some(length) = content_length else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "missing or invalid Content-Length header",
        ));
    };

    let mut body = vec![0u8; length];
    reader.read_exact(&mut body)?;
    let value: Value = serde_json::from_slice(&body)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(Some(value))
}

/// Write one message with `Content-Length` framing.
fn write_message<W: Write>(writer: &mut W, message: Message) -> io::Result<()> {
    let body = serde_json::to_vec(&message.into_json())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(&body)?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Frame a JSON value as an LSP message on the wire.
    fn frame(value: Value) -> Vec<u8> {
        let body = serde_json::to_vec(&value).unwrap();
        let mut out = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
        out.extend_from_slice(&body);
        out
    }

    /// Split the raw output stream into decoded JSON messages.
    fn parse_stream(bytes: &[u8]) -> Vec<Value> {
        let mut reader = io::BufReader::new(bytes);
        let mut messages = Vec::new();
        while let Some(value) = read_message(&mut reader).unwrap() {
            messages.push(value);
        }
        messages
    }

    #[test]
    fn drives_initialize_open_and_exit_over_framed_stdio() {
        let mut input = Vec::new();
        input.extend(frame(json!({
            "jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}
        })));
        input.extend(frame(json!({
            "jsonrpc": "2.0", "method": "initialized", "params": {}
        })));
        input.extend(frame(json!({
            "jsonrpc": "2.0", "method": "textDocument/didOpen",
            "params": { "textDocument": {
                "uri": "file:///a.lby",
                "text": "fn main -> i64\n    let value bool = 1\n    return 0\n"
            } }
        })));
        input.extend(frame(json!({
            "jsonrpc": "2.0", "id": 2, "method": "shutdown", "params": {}
        })));
        input.extend(frame(json!({
            "jsonrpc": "2.0", "method": "exit", "params": {}
        })));

        let mut reader = io::BufReader::new(&input[..]);
        let mut output: Vec<u8> = Vec::new();
        serve(&mut reader, &mut output).unwrap();

        let messages = parse_stream(&output);
        // initialize result, publishDiagnostics, shutdown result.
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["id"], json!(1));
        assert!(messages[0]["result"]["capabilities"].is_object());
        assert_eq!(
            messages[1]["method"],
            json!("textDocument/publishDiagnostics")
        );
        assert!(
            !messages[1]["params"]["diagnostics"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(messages[2]["id"], json!(2));
    }
}
