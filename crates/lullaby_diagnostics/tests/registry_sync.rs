//! Drift guard: keeps `documents/diagnostic_registry.md` and the diagnostic
//! codes actually emitted by the compiler in sync.
//!
//! A diagnostic code is user-facing and tool-facing: `L####` is what a user
//! looks up and what an LLM/editor integration matches on. Three defects have
//! historically crept in here, each of which this test now makes impossible to
//! land:
//!
//! 1. **Duplicates** — one number documented twice with two different meanings,
//!    so neither a user nor a tool can tell which error they have. (`L0210` was
//!    registered for both a malformed region declaration and a for-loop missing
//!    its `to`.)
//! 2. **Missing** — a code emitted by the compiler but absent from the registry,
//!    leaving a user who hits it with nothing to look up. (`L0214`/`L0215`.)
//! 3. **Orphaned** — a code documented but no longer emitted anywhere, so the
//!    registry advertises an error that can never occur.
//!
//! The test reads the two sources of truth directly — the registry Markdown and
//! the crate sources — so it cannot drift out of date the way a hand-maintained
//! list would.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

/// The workspace root, derived from this crate's manifest directory
/// (`<root>/crates/lullaby_diagnostics`).
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crate manifest dir should live at <root>/crates/<crate>")
        .to_path_buf()
}

/// Extract every `L####` code that appears as a *quoted string literal*, e.g.
/// `self.error("L0217", ...)` or `code: "L0210"`.
///
/// Quoting is the discriminator that separates a real emission site from a
/// passing mention: prose and doc comments spell codes bare or in backticks
/// (`L0339`), while every construction site spells them `"L0339"`.
fn quoted_codes(text: &str) -> BTreeSet<String> {
    let bytes = text.as_bytes();
    let mut found = BTreeSet::new();
    // A code literal is exactly seven bytes: `"` `L` d d d d `"`.
    for window in bytes.windows(7) {
        if window[0] == b'"'
            && window[1] == b'L'
            && window[6] == b'"'
            && window[2..6].iter().all(u8::is_ascii_digit)
        {
            found.insert(String::from_utf8_lossy(&window[1..6]).into_owned());
        }
    }
    found
}

/// Recursively collect `.rs` files under `dir`, skipping test modules: those
/// assert on codes rather than emitting them, so counting them would let an
/// orphaned code masquerade as live.
fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, out);
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.ends_with(".rs") && !name.to_ascii_lowercase().contains("test") {
            out.push(path);
        }
    }
}

/// Every code emitted from non-test crate sources.
fn emitted_codes(root: &Path) -> BTreeSet<String> {
    let mut files = Vec::new();
    let crates_dir = root.join("crates");
    let entries = fs::read_dir(&crates_dir).expect("crates/ directory should exist");
    for entry in entries.flatten() {
        let src = entry.path().join("src");
        if src.is_dir() {
            rust_sources(&src, &mut files);
        }
    }
    assert!(!files.is_empty(), "expected to find crate sources to scan");

    let mut codes = BTreeSet::new();
    for file in files {
        let text = fs::read_to_string(&file).unwrap_or_default();
        codes.extend(quoted_codes(&text));
    }
    codes
}

/// Every code documented as a row of the registry's `## Codes` table, mapped to
/// the line numbers it was declared on (so a duplicate can name its offenders).
fn registered_codes(root: &Path) -> BTreeMap<String, Vec<usize>> {
    let path = root.join("documents").join("diagnostic_registry.md");
    let text = fs::read_to_string(&path).expect("diagnostic_registry.md should be readable");

    let mut rows: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (index, line) in text.lines().enumerate() {
        // A code row looks like: | `L0217` | parser | ... |
        let Some(rest) = line.strip_prefix("| `") else {
            continue;
        };
        let Some(code) = rest.split('`').next() else {
            continue;
        };
        let is_code = code.len() == 5
            && code.starts_with('L')
            && code[1..].bytes().all(|b| b.is_ascii_digit());
        if is_code {
            rows.entry(code.to_string()).or_default().push(index + 1);
        }
    }
    assert!(!rows.is_empty(), "expected registry rows to parse");
    rows
}

/// A diagnostic number must mean exactly one thing. Two rows for one code means
/// the registry describes two different errors under one number.
#[test]
fn registry_has_no_duplicate_codes() {
    let registry = registered_codes(&workspace_root());
    let duplicates: Vec<String> = registry
        .iter()
        .filter(|(_, lines)| lines.len() > 1)
        .map(|(code, lines)| format!("{code} declared on lines {lines:?}"))
        .collect();

    assert!(
        duplicates.is_empty(),
        "diagnostic_registry.md declares a code more than once; each `L####` must \
         have exactly one meaning. Give the newer meaning the next free number \
         and update every reference to it:\n  {}",
        duplicates.join("\n  ")
    );
}

/// A user who hits a code must be able to look it up.
#[test]
fn every_emitted_code_is_registered() {
    let root = workspace_root();
    let registry = registered_codes(&root);
    let missing: Vec<String> = emitted_codes(&root)
        .into_iter()
        .filter(|code| !registry.contains_key(code))
        .collect();

    assert!(
        missing.is_empty(),
        "these diagnostic codes are emitted by the compiler but are missing from \
         documents/diagnostic_registry.md, so a user who hits one has nothing to \
         look up. Add a row for each: {missing:?}"
    );
}

/// The registry must not advertise an error that can never occur.
#[test]
fn every_registered_code_is_emitted() {
    let root = workspace_root();
    let emitted = emitted_codes(&root);
    let orphaned: Vec<String> = registered_codes(&root)
        .into_keys()
        .filter(|code| !emitted.contains(code))
        .collect();

    assert!(
        orphaned.is_empty(),
        "these diagnostic codes are documented in documents/diagnostic_registry.md \
         but are no longer emitted anywhere, so the registry describes an error \
         that cannot occur. Remove each dead row, or mark it reserved if the \
         number is being held on purpose: {orphaned:?}"
    );
}

#[test]
fn quoted_codes_matches_only_quoted_literals() {
    // Emission sites are quoted; prose mentions are not.
    let codes = quoted_codes(r#"self.error("L0217", "expected `to`"); // see L0206 and `L0210`"#);
    assert_eq!(codes, BTreeSet::from(["L0217".to_string()]));
    // Guards against matching a longer run of digits (`"L02170"` is not a code).
    assert!(quoted_codes(r#""L02170""#).is_empty());
}
