//! Per-process, per-test scratch directories for the CLI test harness.
//!
//! Tests that emit a real `.exe` and then run it MUST NOT share a path with any
//! other process. A fixed `std::env::temp_dir().join("some_fixed_name")` is
//! machine-global: two `cargo test` runs in two git worktrees on the same
//! machine (this project's normal mode — several agents build concurrently)
//! land on byte-identical `.exe` paths. One process then rewrites or deletes the
//! image the other process is currently executing, which surfaces as
//! `os error 32` ("being used by another process") on the write and
//! `os error 2` ("cannot find the file") on the spawn — non-deterministically,
//! and worse under load.
//!
//! [`ScratchDir`] is the harness's only sanctioned answer. It is unique per
//! process AND per construction, is created on demand, and removes itself on
//! `Drop` (so it cleans up on panic and early return, not just the happy path).
//!
//! Reintroducing the bug is structurally hard rather than merely discouraged:
//! the fuzz helpers take a `&ScratchDir`, not a `&Path`, and `ScratchDir` has no
//! constructor that accepts a caller-chosen path. A new fuzzer physically cannot
//! hand them a fixed directory — the only way to obtain one is `ScratchDir::new`,
//! which always mints a fresh unique path.
//!
//! # A fixed path also masks dead assertions
//!
//! Isolation is the headline reason, but not the only one. `%TEMP%` is never
//! cleaned, so a *fixed* path accumulates artifacts from every past run — and an
//! assertion that reads one cannot tell this run's output from one left months
//! ago. Converting the harness to `ScratchDir` caught four tests doing exactly
//! that: three read a `.obj` the direct-PE fast path had long since stopped
//! emitting (they were scanning a three-day-old file), and `ffi_calls_c_abs_when_linkable`
//! ran a stale exe when its link step had silently skipped. All four passed
//! green while proving nothing. A scratch dir starts empty, so a missing artifact
//! fails loudly the first time.
//!
//! # Every writer goes through here
//!
//! The whole CLI harness now mints its paths this way; `std::env::temp_dir()`
//! should not appear in these tests outside this module. Helpers that hand a path
//! to a caller (`fs_temp_dir`, `new_project_work_dir`, `fmt_comments::write_temp`)
//! return the `ScratchDir` *with* the path so the guard outlives the borrow —
//! returning the path alone deletes the directory at the helper's `}`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Monotonic per-process counter. Combined with the pid this is already unique;
/// the wall-clock nanos below only guard against a pid being recycled while a
/// previous run's directory somehow survived.
static SEQ: AtomicU64 = AtomicU64::new(0);

/// A uniquely-named temporary directory that deletes itself when dropped.
///
/// Hold it for as long as you need the files inside it: the directory and
/// everything under it are removed when the value goes out of scope. Bind it to
/// a named local (`let dir = ScratchDir::new("x");`), never to `_`, or it drops
/// immediately.
#[must_use = "a ScratchDir bound to `_` is deleted immediately; bind it to a name"]
pub(crate) struct ScratchDir {
    path: PathBuf,
}

impl ScratchDir {
    /// Create a fresh scratch directory. `label` only aids debugging when a run
    /// is interrupted; uniqueness comes from the pid, the sequence number, and
    /// the wall-clock nanos, never from the label.
    pub(crate) fn new(label: &str) -> Self {
        let seq = SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.subsec_nanos());
        let pid = std::process::id();
        let path =
            std::env::temp_dir().join(format!("lullaby_scratch_{label}_{pid}_{seq}_{nanos}"));
        // Cannot already exist (pid+seq is unique within this process), but a
        // recycled pid plus a leaked directory from a killed run is conceivable;
        // start from a known-empty directory either way.
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|e| panic!("create scratch dir {}: {e}", path.display()));
        Self { path }
    }

    /// A path to `name` inside this directory. This is deliberately the only
    /// accessor: handing out the bare directory would let a caller stash it past
    /// the guard's lifetime and use it after the directory is gone.
    pub(crate) fn join(&self, name: impl AsRef<Path>) -> PathBuf {
        self.path.join(name)
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        // Cleanup only — never panics, and never fails a test. The path is
        // already unique, so a directory that survives is untidy, not a
        // correctness problem, and panicking here would mask the real failure
        // when we are dropping during an assertion unwind.
        //
        // The bounded retry is for cleanup alone: every child spawned in these
        // tests is fully waited on via `Command::output()` before we get here,
        // so the exe's own handle is gone, but a Windows virus scanner can hold
        // a transient handle on a just-executed image for a few milliseconds.
        // This is NOT the fix for the write/run race (unique paths are) and is
        // not on any assertion's path.
        for attempt in 0..10u32 {
            match std::fs::remove_dir_all(&self.path) {
                Ok(()) => return,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return,
                Err(_) if attempt < 9 => {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                Err(e) => eprintln!(
                    "warning: could not remove scratch dir {}: {e}",
                    self.path.display()
                ),
            }
        }
    }
}
