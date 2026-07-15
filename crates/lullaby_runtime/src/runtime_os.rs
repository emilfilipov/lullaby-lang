//! Operating-system services shared by every interpreter backend: the monotonic
//! and wall clocks, thread sleep, the OS CSPRNG, and the stdin line/all readers.
//! Split out of `lib.rs` as a behavior-preserving code move; `Value`,
//! `RuntimeError`, and `option_value` (in sibling modules) are reached through
//! `crate::` paths.

use crate::{RuntimeError, Value, option_value};

/// Unwrap a runtime `Value` expected to be an `i64`, reporting `L0417` otherwise.
/// The process-global monotonic baseline for `mono_now`. It is initialized on
/// the first call to [`monotonic_now_nanos`] and never re-initialized, so the
/// clock is non-decreasing for the whole process. Both interpreters and the
/// bytecode VM route through this single function, so they observe one shared
/// baseline regardless of which backend is active.
static MONOTONIC_BASELINE: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// Nanoseconds elapsed since the process-global monotonic baseline. The first
/// call establishes the baseline (returning `0` or a tiny value); every later
/// call returns a value `>=` all previous ones. Backs the `mono_now` builtin on
/// every interpreter backend.
pub fn monotonic_now_nanos() -> i64 {
    let baseline = MONOTONIC_BASELINE.get_or_init(std::time::Instant::now);
    baseline.elapsed().as_nanos() as i64
}

/// Milliseconds since the Unix epoch (wall-clock time). Backs the `wall_now`
/// builtin. A pre-epoch system clock (rare, misconfigured host) yields the
/// negated pre-epoch offset so the value stays total and never panics.
pub fn wall_now_millis() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(delta) => delta.as_millis() as i64,
        Err(err) => -(err.duration().as_millis() as i64),
    }
}

/// Sleep the current thread for `ms` milliseconds. A negative `ms` is treated as
/// `0` (no sleep, no error), keeping the builtin total. Backs `sleep_millis`.
pub fn sleep_millis(ms: i64) {
    if ms > 0 {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    }
}

/// Fill a fresh buffer of `len` bytes with cryptographically-secure randomness
/// straight from the operating-system CSPRNG (`getrandom`/`getentropy` on
/// Unix-likes, `BCryptGenRandom` on Windows, `/dev/urandom` as a fallback).
/// This is a real OS randomness source — never a seeded or deterministic PRNG,
/// so callers may use it for keys, nonces, and tokens.
///
/// Backs the `os_random` builtin on every interpreter backend (AST runtime, IR
/// interpreter, bytecode VM) so all three exhibit identical behavior:
///
/// - `len < 0` returns `Err("os_random length must be non-negative")` and never
///   panics.
/// - `len == 0` returns `Ok(Vec::new())` (no syscall, an empty buffer).
/// - a genuine OS RNG failure is surfaced as `Err(message)` rather than a panic.
pub fn os_random_bytes(len: i64) -> Result<Vec<u8>, String> {
    if len < 0 {
        return Err("os_random length must be non-negative".to_string());
    }
    let len = len as usize;
    if len == 0 {
        return Ok(Vec::new());
    }
    let mut buffer = vec![0u8; len];
    match getrandom::fill(&mut buffer) {
        Ok(()) => Ok(buffer),
        Err(error) => Err(format!("os_random failed: {error}")),
    }
}

/// Backing implementation of the `read_line() -> option<string>` stdin builtin,
/// shared verbatim by the AST and IR/bytecode interpreters so line-reading is
/// byte-for-byte identical across backends. Reads one line from the process's
/// standard input through the shared, buffered global `Stdin` handle (so
/// consecutive calls consume consecutive lines without dropping buffered bytes),
/// strips the trailing line terminator, and distinguishes end-of-input from a
/// blank line:
///
/// - End of input (no bytes read) yields `none`.
/// - A line yields `some(text)` with the trailing `\n` removed, and a preceding
///   `\r` also removed so Windows CRLF input round-trips like LF input.
/// - A blank input line yields `some("")`, keeping EOF and an empty line
///   distinct.
///
/// A genuine read failure (for example, non-UTF-8 bytes on stdin) is the
/// resource error `L0419`, the standard-stream I/O family.
pub fn read_stdin_line() -> Result<Value, RuntimeError> {
    use std::io::BufRead;
    let mut buffer = String::new();
    let read = std::io::stdin()
        .lock()
        .read_line(&mut buffer)
        .map_err(|error| {
            RuntimeError::resource(
                "L0419",
                format!("failed to read a line from stdin: {error}"),
            )
        })?;
    if read == 0 {
        return Ok(option_value(None));
    }
    if buffer.ends_with('\n') {
        buffer.pop();
        if buffer.ends_with('\r') {
            buffer.pop();
        }
    }
    Ok(option_value(Some(Value::String(buffer.into()))))
}

/// Backing implementation of the `read_all() -> string` stdin builtin, shared by
/// both interpreters. Reads the whole of standard input to EOF into one `string`
/// (empty when stdin is empty or already closed). A read failure (for example,
/// non-UTF-8 bytes on stdin) is the resource error `L0419`.
pub fn read_stdin_all() -> Result<Value, RuntimeError> {
    use std::io::Read;
    let mut buffer = String::new();
    std::io::stdin()
        .lock()
        .read_to_string(&mut buffer)
        .map_err(|error| {
            RuntimeError::resource("L0419", format!("failed to read stdin: {error}"))
        })?;
    Ok(Value::String(buffer.into()))
}
