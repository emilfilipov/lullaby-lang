//! Run the existing frontend pipeline over a document and convert the results
//! into LSP `Diagnostic` JSON values.
//!
//! The pipeline mirrors what `lullaby check` does: lex, then parse, then
//! semantic validation. It stops at the first stage that produces errors, so
//! diagnostics come from exactly one phase at a time (the same behavior a user
//! sees on the command line). Spans are single points (1-based line/column); we
//! convert them to 0-based LSP ranges and widen the end to cover the identifier
//! or token that starts at that position when there is one.

use lullaby_diagnostics::DiagnosticReport;
use lullaby_lexer::{Span, lex, lex_with_comments};
use lullaby_parser::{Program, format_program_with_comments, parse};
use lullaby_semantics::{SemanticDiagnostic, validate};
use serde_json::{Value, json};

/// LSP diagnostic severity: 1 = Error.
const SEVERITY_ERROR: i64 = 1;

/// Compute the LSP diagnostics for a document's source text.
///
/// Returns the list of LSP `Diagnostic` JSON values (possibly empty). Runs
/// lex -> parse -> semantic validation and reports whichever stage first fails.
/// This is the single-document pipeline; module/project awareness lives in
/// [`crate::project`], which reuses the helpers below.
pub fn compute(text: &str) -> Vec<Value> {
    let program = match lex_parse_lsp(text) {
        Ok(program) => program,
        Err(items) => return items,
    };
    match validate(&program) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .into_iter()
            .map(|diagnostic| semantic_diag_to_lsp(text, diagnostic))
            .collect(),
    }
}

/// Lex and parse `text`, returning the parsed [`Program`] or the lex/parse
/// diagnostics already rendered as LSP `Diagnostic` JSON values. The rendered
/// positions match the single-document pipeline exactly.
pub(crate) fn lex_parse_lsp(text: &str) -> Result<Program, Vec<Value>> {
    let tokens = match lex(text) {
        Ok(tokens) => tokens,
        Err(diagnostics) => {
            return Err(diagnostics
                .into_iter()
                .map(|d| lsp_diagnostic(text, d.code, &d.message, d.span))
                .collect());
        }
    };
    match parse(&tokens) {
        Ok(program) => Ok(program),
        Err(diagnostics) => Err(diagnostics
            .into_iter()
            .map(|d| lsp_diagnostic(text, d.code, &d.message, d.span))
            .collect()),
    }
}

/// Render one semantic diagnostic as an LSP `Diagnostic` JSON value. A semantic
/// diagnostic may lack a span; fall back to the top of the document so the
/// marker is still visible.
pub(crate) fn semantic_diag_to_lsp(text: &str, diagnostic: SemanticDiagnostic) -> Value {
    let span = diagnostic.span.unwrap_or(Span::new(1, 1));
    lsp_diagnostic(text, diagnostic.code, &diagnostic.message, span)
}

/// Render one loader diagnostic report (import resolution / cross-module
/// visibility / no-shadowing) as an LSP `Diagnostic` JSON value, positioned by
/// the report's span against the open document.
pub(crate) fn report_to_lsp(text: &str, report: &DiagnosticReport) -> Value {
    let span = report.span.unwrap_or(Span::new(1, 1));
    lsp_diagnostic(text, &report.code, &report.message, span)
}

/// Canonically format a document's source, or `None` if it does not lex/parse.
/// Comments are preserved through the format round-trip.
pub fn format_source(text: &str) -> Option<String> {
    let (tokens, comments) = lex_with_comments(text).ok()?;
    let program = parse(&tokens).ok()?;
    Some(format_program_with_comments(&program, &comments))
}

/// Build one LSP `Diagnostic` JSON value from a Lullaby code, message, and span.
fn lsp_diagnostic(text: &str, code: &str, message: &str, span: Span) -> Value {
    json!({
        "range": span_to_range(text, span),
        "severity": SEVERITY_ERROR,
        "code": code,
        "source": "lullaby",
        "message": message,
    })
}

/// Convert a 1-based Lullaby `Span` point into a 0-based LSP `Range`.
///
/// Lullaby spans are single points. To give editors something to underline we
/// widen the end to the length of the identifier/number/keyword token that
/// begins at that column; if there is no such token (whitespace, end of line)
/// we fall back to a one-character range.
fn span_to_range(text: &str, span: Span) -> Value {
    let line = span.line.saturating_sub(1);
    let character = span.column.saturating_sub(1);
    let width = token_width_at(text, span).max(1);
    json!({
        "start": { "line": line, "character": character },
        "end": { "line": line, "character": character + width },
    })
}

/// The number of characters in the word-like token starting at the span's
/// position, or `0` if the position is not on a word character.
fn token_width_at(text: &str, span: Span) -> usize {
    let Some(line) = text.lines().nth(span.line.saturating_sub(1)) else {
        return 0;
    };
    let start = span.column.saturating_sub(1);
    let chars: Vec<char> = line.chars().collect();
    if start >= chars.len() {
        return 0;
    }
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    if !is_word(chars[start]) {
        return 0;
    }
    let mut width = 0usize;
    for &c in &chars[start..] {
        if is_word(c) {
            width += 1;
        } else {
            break;
        }
    }
    width
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_program_has_no_diagnostics() {
        let diags = compute("fn main -> i64\n    return 0\n");
        assert!(diags.is_empty());
    }

    #[test]
    fn type_mismatch_reports_a_diagnostic() {
        let diags = compute("fn main -> i64\n    let value bool = 1\n    return 0\n");
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0]["severity"], json!(SEVERITY_ERROR));
        assert_eq!(diags[0]["source"], json!("lullaby"));
        assert!(diags[0]["code"].as_str().unwrap().starts_with('L'));
    }

    #[test]
    fn forbidden_brace_reports_a_diagnostic() {
        // A brace is a forbidden delimiter caught in the frontend.
        let diags = compute("fn main -> i64 {\n    return 0\n}\n");
        assert!(!diags.is_empty());
    }

    #[test]
    fn token_width_covers_identifier() {
        // "value" starts at column 9 (1-based) on line 2.
        let text = "fn main -> i64\n    let value bool = 1\n";
        assert_eq!(token_width_at(text, Span::new(2, 9)), 5);
    }

    #[test]
    fn range_is_zero_based() {
        let text = "fn main -> i64\n    let value bool = 1\n";
        let range = span_to_range(text, Span::new(2, 9));
        assert_eq!(range["start"]["line"], json!(1));
        assert_eq!(range["start"]["character"], json!(8));
        assert_eq!(range["end"]["character"], json!(13));
    }
}
