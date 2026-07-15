//! The runtime error type and the scalar argument extractors shared by every
//! interpreter backend. Split out of `lib.rs` as a behavior-preserving code
//! move; `Value` (in `lib.rs`) is reached through a `crate::` path.

use std::fmt;

use lullaby_diagnostics::{Span, TraceFrame};

use crate::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeError {
    pub code: &'static str,
    pub category: ErrorCategory,
    pub message: String,
    pub span: Option<Span>,
    pub function: Option<String>,
    pub traceback: Vec<TraceFrame>,
}

impl RuntimeError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self::categorized(code, ErrorCategory::Runtime, message)
    }

    pub fn resource(code: &'static str, message: impl Into<String>) -> Self {
        Self::categorized(code, ErrorCategory::Resource, message)
    }

    pub fn categorized(
        code: &'static str,
        category: ErrorCategory,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code,
            category,
            message: message.into(),
            span: None,
            function: None,
            traceback: Vec::new(),
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        if self.span.is_none() {
            self.span = Some(span);
        }
        self
    }

    pub fn with_function(mut self, function: impl Into<String>) -> Self {
        if self.function.is_none() {
            self.function = Some(function.into());
        }
        self
    }

    pub fn with_traceback(mut self, traceback: Vec<TraceFrame>) -> Self {
        if self.traceback.is_empty() {
            self.traceback = traceback;
        }
        self
    }
}

/// The error raised when an `extern fn` (C-ABI) function is called on any
/// interpreter (AST, IR, or bytecode). The interpreters cannot execute real C
/// FFI — an extern function only has meaning after native codegen + linking —
/// so a call to one is `L0423` rather than a panic or a silent no-op. `check`
/// still validates the extern declaration and its call sites.
pub fn extern_call_error(name: &str) -> RuntimeError {
    RuntimeError::new(
        "L0423",
        format!(
            "cannot call extern (C-ABI) function `{name}` on an interpreter; compile with `lullaby native` to link and run it"
        ),
    )
}

/// The error raised when an interpreter encounters an `asm` inline-assembly
/// statement. Raw machine code can only run after native codegen + linking, so
/// every interpreter (AST, IR, bytecode) rejects it with `L0425`. `check` still
/// validates the byte range and the enclosing `unsafe` block.
pub fn asm_interpreter_error() -> RuntimeError {
    RuntimeError::new(
        "L0425",
        "cannot execute an `asm` (inline assembly) statement on an interpreter; compile with `lullaby native` to emit and link the machine code",
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    Runtime,
    Resource,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime => write!(formatter, "runtime"),
            Self::Resource => write!(formatter, "resource"),
        }
    }
}

/// Unwrap a runtime `Value` expected to be a string, reporting `L0417` otherwise.
pub fn expect_string(name: &str, value: Value) -> Result<String, RuntimeError> {
    match value {
        Value::String(text) => Ok((text).into()),
        other => Err(RuntimeError::new(
            "L0417",
            format!("{name} expects a string but got `{other}`"),
        )),
    }
}

pub fn expect_i64(name: &str, value: Value) -> Result<i64, RuntimeError> {
    match value {
        Value::I64(number) => Ok(number),
        other => Err(RuntimeError::new(
            "L0417",
            format!("{name} expects an i64 but got `{other}`"),
        )),
    }
}

/// Unwrap a runtime `Value` expected to be a `bool`, reporting `L0417`
/// otherwise. Shared by the AST interpreter and the IR interpreter so every
/// backend extracts boolean builtin arguments identically.
pub fn expect_bool(name: &str, value: Value) -> Result<bool, RuntimeError> {
    match value {
        Value::Bool(flag) => Ok(flag),
        other => Err(RuntimeError::new(
            "L0417",
            format!("{name} expects a bool but got `{other}`"),
        )),
    }
}

/// Unwrap a runtime `Value` expected to be a list (an array), reporting `L0417`
/// otherwise. A `list<T>` is represented at runtime as a `Value::Array`.
pub fn expect_list(name: &str, value: Value) -> Result<Vec<Value>, RuntimeError> {
    match value {
        Value::Array(values) => Ok((values).into()),
        other => Err(RuntimeError::new(
            "L0417",
            format!("{name} expects a list but got `{other}`"),
        )),
    }
}
