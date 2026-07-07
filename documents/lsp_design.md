# Lullaby Language Server (LSP) Design

The `lullaby_lsp` crate implements a minimal [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) server for `.lby` source files. It is additive editor tooling: it reuses the existing frontend (lexer, parser, semantic analyzer) and the canonical formatter without touching the interpreters, IR, native code generation, or the WebAssembly backend. Cross-backend parity is unaffected.

The server is exposed through the `lullaby lsp` CLI subcommand, which runs the stdio read/write loop. It uses the Rust standard library plus `serde_json` only; no third-party LSP framework, async runtime, or protocol crate is added.

## Transport And Framing

The server speaks JSON-RPC 2.0 over stdin/stdout using the LSP base-protocol framing:

```
Content-Length: <N>\r\n
\r\n
<N bytes of UTF-8 JSON>
```

`crates/lullaby_lsp/src/transport.rs` reads header lines until a blank line, parses the `Content-Length`, reads exactly that many body bytes, and decodes the JSON with `serde_json`. Outbound messages are written with the same framing and the stream is flushed after each write. The loop terminates when the client sends `exit` or closes stdin.

## Request-Handling Core

All protocol behavior lives in the pure function

```rust
pub fn handle_message(
    state: &mut ServerState,
    method: &str,
    id: Option<Value>,
    params: Value,
) -> Vec<Message>
```

It mutates the in-memory `ServerState` and returns the outbound messages (responses and notifications) instead of writing to any stream. This makes the whole protocol testable without real stdio: tests build `params` as `serde_json` values, call `handle_message`, and assert on the returned `Message`s. The stdio loop (`serve`/`run_stdio`) is a thin wrapper that decodes a request, calls `handle_message`, and writes each returned message back.

`id` is `Some` for a request (which must receive a response) and `None` for a notification (which must not).

## Capabilities

The `initialize` response advertises:

- `textDocumentSync = 1` (full): the client sends the entire document text on every change.
- `documentFormattingProvider = true`.

Hover, completion, and other providers are intentionally not advertised.

## Lifecycle

| Method | Behavior |
| --- | --- |
| `initialize` | Returns capabilities and `serverInfo`. |
| `initialized` | Acknowledged (no-op notification). |
| `shutdown` | Marks the server as shutdown-requested and returns a null result. |
| `exit` | Sets the exit flag so the stdio loop stops. |

## Document Sync

Open documents are held in a `HashMap<String, String>` keyed by URI:

- `textDocument/didOpen` stores the text and publishes diagnostics.
- `textDocument/didChange` replaces the text with the last content change's full `text` (full sync) and republishes diagnostics.
- `textDocument/didClose` drops the document and publishes an empty diagnostics set to clear any markers.

## Diagnostics

On open and change the server runs the same pipeline as `lullaby check`: lex, then parse, then `lullaby_semantics::validate`. It reports whichever stage first produces errors, which matches the command-line behavior (a single failing phase at a time). Each Lullaby diagnostic carries a stable code (for example `L0307`), a message, and a source span.

Lullaby spans are single 1-based `line`/`column` points. They are converted to 0-based LSP ranges. Because a span has no length, the end is widened to cover the identifier/number/keyword token that starts at that column (scanning the document line for word characters); when the position is not on a word character the range falls back to a single character. Each diagnostic is published as an LSP `Diagnostic` with `severity = 1` (Error), `source = "lullaby"`, the Lullaby `code`, and the message, via a `textDocument/publishDiagnostics` notification.

## Formatting

`textDocument/formatting` looks up the stored document text and runs the canonical formatter (`lullaby_parser::format_program`) after a successful lex+parse. It returns a single full-document `TextEdit` whose range spans the entire current document. If the document does not parse, or is already canonical, it returns no edits.

## Testing

`crates/lullaby_lsp` carries unit tests that drive `handle_message` directly (no stdio):

- `initialize` advertises the expected capabilities.
- `didOpen` with an invalid program publishes a `publishDiagnostics` notification with at least one diagnostic whose code is `L`-prefixed and whose range is 0-based and well-formed.
- `didOpen` with a valid `fn main -> i64` returning a literal publishes zero diagnostics.
- `didChange` updates the stored text and republishes.
- `didClose` drops the document and clears diagnostics.
- `formatting` returns exactly one full-document `TextEdit` for a parseable-but-unformatted document and no edits for an unparseable one.

The transport module additionally tests the framed read/write loop end to end over in-memory byte buffers (initialize -> didOpen -> shutdown -> exit).

## Deferred Features

The following are intentionally out of scope for this increment and can be layered on later without changing the transport or the `handle_message` shape:

- Hover, completion, signature help.
- Go-to-definition, references, document symbols, workspace symbols.
- Incremental (range) document sync.
- Code actions / quick fixes (for example applying the formatter or diagnostic-directed edits).
- Multi-file / project-aware analysis (imports and `lullaby.json` search directories); the server currently analyzes each open document in isolation.
- Reporting more than the first failing phase's diagnostics at once.
