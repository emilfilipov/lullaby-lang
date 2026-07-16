//! Real failure isolation for `lullaby test`: run the suite in a child process
//! and survive a test that kills it.
//!
//! # Why a subprocess
//!
//! The interpreter returns every *runtime* error as an ordinary `RuntimeError`,
//! so the runner already contains those in-process. Two shapes are not runtime
//! errors and escape that `Result` entirely:
//!
//! * a **stack overflow** faults on the guard page and the process dies — Rust
//!   cannot unwind or catch it (this is exactly why libtest can `catch_unwind` a
//!   panic and we cannot); and
//! * a **non-terminating** test never returns at all, so there is nothing to
//!   catch.
//!
//! Neither can be contained inside the process running the test. The only
//! mechanism that contains both is an OS process boundary plus a deadline, so
//! that is what this module implements.
//!
//! # Batch-with-resume, not process-per-test
//!
//! A process per test would be the obvious shape, but it pays a process spawn
//! (~13 ms on Windows) *and* a full recompile of the program for every test —
//! ~16 ms each, so ~1.6 s on a 100-test suite against ~20 ms in-process. That is
//! a tax on the common path, where every test passes and nothing needs isolating.
//!
//! So the child runs the *whole remaining suite* sequentially in one process, and
//! the parent spawns again only when a test actually takes the child down: on a
//! crash or a timeout the parent records that test's failure and resumes the
//! batch at the next index. An all-passing suite therefore costs exactly one
//! spawn and one compile no matter how many tests it has — the same work the old
//! in-process runner did — and it keeps that runner's exact semantics, since all
//! tests still run sequentially in one process. Only a suite that actually kills
//! the runner pays per incident, which is the right place to pay.
//!
//! # The protocol
//!
//! The child reports progress on **stderr**, one line per event, so the test's
//! own stdout stays untouched and is inherited straight through to the terminal.
//! A Lullaby program can itself write to stderr, so every protocol line carries a
//! per-run **nonce** the child is given at spawn: a test cannot forge a line whose
//! nonce it was never told, and any stderr line that is not a protocol line is
//! forwarded to the parent's stderr as the test's own output.
//!
//! Results are printed as they stream in, never collected and dumped at the end:
//! a batch only ever resumes *past* the test that killed it, so results arrive in
//! index order and printing them on arrival preserves both the live progress of
//! the old runner and its ordering. The child flushes stdout before each result
//! line and the parent flushes its own stdout before each spawn, so a test's
//! output always lands before the `PASS`/`FAIL` line reporting it.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use lullaby_runtime::run_named_function;

use crate::args::OutputMode;
use crate::compile::{SourceMode, compile};
use crate::diagnostics::format_reports;

use super::test::discover_tests;

/// The hidden internal subcommand the parent re-invokes itself with. Positional:
/// `<path> <start-index> <verbose:0|1> <nonce> [filter]`.
pub(crate) const RUN_BATCH_COMMAND: &str = "__run-test-batch";

/// Protocol verbs, emitted by the child as `##<nonce> <verb> <index> [payload]`.
const START: &str = "start";
const PASS: &str = "pass";
const FAIL: &str = "fail";
/// The child finished the batch under its own power. This is what completes a
/// batch — NOT stderr EOF. A test may leave a grandchild holding the inherited
/// stderr pipe, in which case EOF never arrives; waiting for it would stall a
/// wholly passing suite until the deadline. See `kill_process_tree`.
const DONE: &str = "done";

/// Separates the failure message from each traceback frame inside one `fail`
/// payload. A control character, so it cannot occur in a rendered diagnostic.
const FIELD_SEPARATOR: char = '\u{1f}';

/// Kill the child AND every process it spawned.
///
/// Killing only the child is not enough: `sys_status`/`sys_output`/`proc_spawn`
/// let a `test_*` function spawn arbitrary processes, and a grandchild both
/// outlives a killed child and inherits the child's stderr pipe handle — so the
/// pipe never reaches EOF and the runaway process keeps running. A runner whose
/// `--timeout N` does not actually bound the run within ~N seconds fails the very
/// guarantee it exists to provide.
///
/// Best-effort by design: this runs only on the timeout path, where we are
/// forcibly stopping a runaway test. A suite that *passes* keeps whatever it
/// spawned — `proc_spawn` is a documented builtin for background processes, and
/// reaping a passing test's children would break legitimate use.
#[cfg(windows)]
fn kill_process_tree(child: &mut Child) {
    // `/T` kills the whole tree rooted at the child; it must run BEFORE the child
    // itself dies, or the parent/child links the tree walk needs are gone.
    let _ = Command::new("taskkill")
        .args(["/F", "/T", "/PID"])
        .arg(child.id().to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// The POSIX counterpart: the child leads its own process group (see
/// `process_group(0)` at spawn), so signalling the negated group id reaches every
/// descendant that has not deliberately left the group.
#[cfg(unix)]
fn kill_process_tree(child: &mut Child) {
    let _ = Command::new("kill")
        .arg("--")
        .arg(format!("-{}", child.id()))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(not(any(windows, unix)))]
fn kill_process_tree(_child: &mut Child) {}

/// Windows `STATUS_STACK_OVERFLOW`, the one abnormal exit we can name exactly.
#[cfg(windows)]
const WINDOWS_STACK_OVERFLOW: i32 = 0xC000_00FDu32 as i32;

/// Describe why a child died, without guessing. A stack overflow is the common
/// cause but not the only one — an external `taskkill`/`SIGTERM` lands here too,
/// and reporting that as "most likely a stack overflow" would be a fabrication.
fn describe_abnormal_exit(status: Option<ExitStatus>) -> String {
    #[cfg(windows)]
    if let Some(status) = status
        && status.code() == Some(WINDOWS_STACK_OVERFLOW)
    {
        return "the test process terminated abnormally: stack overflow                 (STATUS_STACK_OVERFLOW), i.e. unbounded recursion"
            .to_string();
    }
    match status {
        Some(status) => format!("the test process terminated abnormally ({status})"),
        None => "the test process terminated abnormally (cause unknown)".to_string(),
    }
}

/// How many tests passed and failed so far.
#[derive(Default)]
pub(crate) struct Tally {
    pub(crate) passed: usize,
    pub(crate) failed: usize,
}

/// A per-run token the test being run cannot know, so it cannot forge a protocol
/// line on the stderr it shares with the child.
fn make_nonce() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.subsec_nanos())
        .unwrap_or(0);
    format!("{:x}-{:x}", std::process::id(), nanos)
}

/// Escape a message so it survives as one protocol line.
fn escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('\n', "\\n")
}

fn unescape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('n') => out.push('\n'),
            Some('\\') => out.push('\\'),
            Some(other) => out.push(other),
            None => out.push('\\'),
        }
    }
    out
}

/// Print one test's result, in the same shape the in-process runner used.
fn report_pass(name: &str, tally: &mut Tally) {
    tally.passed += 1;
    println!("PASS {name}");
}

fn report_fail(name: &str, message: &str, frames: &[String], tally: &mut Tally) {
    tally.failed += 1;
    println!("FAIL {name}: {message}");
    for frame in frames {
        println!("{frame}");
    }
}

/// Run every discovered test under isolation, printing `PASS`/`FAIL` per test as
/// results arrive, and return the tally.
///
/// `timeout` of `None` disables the per-test deadline.
pub(crate) fn run_isolated(
    path: &Path,
    names: &[String],
    filter: Option<&str>,
    verbose: bool,
    timeout: Option<Duration>,
) -> Result<Tally, String> {
    let exe = std::env::current_exe()
        .map_err(|error| format!("could not locate the lullaby executable: {error}"))?;
    let nonce = make_nonce();
    let mut tally = Tally::default();
    let mut next = 0usize;

    while next < names.len() {
        let resume = run_batch(
            &exe, path, next, filter, verbose, &nonce, timeout, names, &mut tally,
        )?;
        // A batch must always make progress, or this would spin forever on a child
        // that dies before reporting anything. If it made none, attribute the
        // failure to the test we tried to resume at and step past it.
        if resume > next {
            next = resume;
        } else {
            report_fail(
                &names[next],
                "the isolated test process exited before reporting this test",
                &[],
                &mut tally,
            );
            next += 1;
        }
    }

    Ok(tally)
}

/// Spawn one child covering `names[start..]` and consume its protocol stream
/// until it finishes, crashes, or trips the per-test deadline. Returns the index
/// to resume at (`names.len()` when the batch completed the suite).
#[allow(clippy::too_many_arguments)]
fn run_batch(
    exe: &Path,
    path: &Path,
    start: usize,
    filter: Option<&str>,
    verbose: bool,
    nonce: &str,
    timeout: Option<Duration>,
    names: &[String],
    tally: &mut Tally,
) -> Result<usize, String> {
    let mut command = Command::new(exe);
    command
        .arg(RUN_BATCH_COMMAND)
        .arg(path)
        .arg(start.to_string())
        .arg(if verbose { "1" } else { "0" })
        .arg(nonce);
    if let Some(filter) = filter {
        command.arg(filter);
    }
    // POSIX: give the child its own process group so `kill_process_tree` can
    // signal every descendant by group id.
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    // Order our own prints against the child's inherited stdout.
    let _ = std::io::stdout().flush();
    // stdout is inherited so a test's own output reaches the terminal exactly as
    // it did in-process; stderr is piped because it carries the protocol.
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("could not start the isolated test process: {error}"))?;

    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "isolated test process has no stderr pipe".to_string())?;
    let (tx, rx) = mpsc::channel::<String>();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });

    let prefix = format!("##{nonce} ");
    let mut in_flight: Option<usize> = None;
    // The highest index this batch actually reported. This is what makes "the
    // child finished" distinguishable from "the child died having reported
    // nothing" -- see the `Disconnected` arm.
    let mut last_reported: Option<usize> = None;
    let mut deadline = timeout.map(|limit| Instant::now() + limit);

    let resume = loop {
        let event = match deadline {
            Some(at) => rx.recv_timeout(at.saturating_duration_since(Instant::now())),
            None => rx.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };

        match event {
            Ok(line) => {
                let Some(rest) = line.strip_prefix(&prefix) else {
                    // Not ours: the test's own stderr output. Pass it through.
                    eprintln!("{line}");
                    continue;
                };
                let (verb, payload) = rest.split_once(' ').unwrap_or((rest, ""));
                if verb == DONE {
                    // The child ran the batch to completion. Break on this rather
                    // than on EOF: a grandchild holding the stderr pipe can defer
                    // EOF indefinitely, and a passing suite must not wait for it.
                    break names.len();
                }
                let (index_text, detail) = payload.split_once(' ').unwrap_or((payload, ""));
                let index: usize = match index_text.parse() {
                    Ok(index) if index < names.len() => index,
                    _ => continue,
                };
                match verb {
                    START => {
                        in_flight = Some(index);
                        deadline = timeout.map(|limit| Instant::now() + limit);
                    }
                    PASS => {
                        report_pass(&names[index], tally);
                        in_flight = None;
                        last_reported = Some(index);
                        deadline = timeout.map(|limit| Instant::now() + limit);
                    }
                    FAIL => {
                        // One line carries the whole failure: message, then a
                        // traceback frame per field. Collecting it in a single
                        // event is what lets us print on arrival.
                        let mut fields = detail.split(FIELD_SEPARATOR);
                        let message = unescape(fields.next().unwrap_or_default());
                        let frames: Vec<String> = fields.map(unescape).collect();
                        report_fail(&names[index], &message, &frames, tally);
                        in_flight = None;
                        last_reported = Some(index);
                        deadline = timeout.map(|limit| Instant::now() + limit);
                    }
                    _ => {}
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                // The in-flight test outlived the deadline: it is not going to
                // finish. Kill the whole TREE -- not just the child -- bank the
                // timeout as that test's failure, and resume the batch after it.
                kill_process_tree(&mut child);
                let _ = child.kill();
                let _ = child.wait();
                let limit = timeout.unwrap_or_default().as_secs();
                break match in_flight {
                    Some(index) => {
                        report_fail(
                            &names[index],
                            &format!(
                                "timed out after {limit}s (did not terminate); the test process was \
                                 killed — raise the limit with `--timeout <seconds>` if the test is \
                                 merely slow"
                            ),
                            &[],
                            tally,
                        );
                        index + 1
                    }
                    // No test in flight: the child stalled between tests. Resume
                    // after the last test it did report; if it reported none,
                    // `run_isolated` turns the no-progress batch into a failure.
                    None => last_reported.map_or(start, |index| index + 1),
                };
            }
            Err(RecvTimeoutError::Disconnected) => {
                // stderr closed: the child is done, one way or another.
                let status = child.wait().ok();
                break match in_flight {
                    // A test was running and the process died under it: a stack
                    // overflow or another abnormal termination. The exit code is
                    // platform-specific (Windows STATUS_STACK_OVERFLOW 0xC00000FD,
                    // POSIX 127 or a signal), so report the *observable* — abnormal
                    // termination — and never pin a value.
                    Some(index) => {
                        report_fail(&names[index], &describe_abnormal_exit(status), &[], tally);
                        index + 1
                    }
                    // No test in flight and no `done`: the child exited between
                    // tests without finishing the batch. Resume after the last test
                    // it reported -- NEVER claim the batch is complete here.
                    // Treating this as "complete" would make a child that died
                    // having reported nothing indistinguishable from one that ran
                    // everything, silently yielding `0 passed, 0 failed` + exit 0:
                    // a green run that executed no tests. `run_isolated`'s progress
                    // guard turns the report-nothing case into a loud failure.
                    None => last_reported.map_or(start, |index| index + 1),
                };
            }
        }
    };

    // Deliberately NOT joined. The reader blocks until stderr reaches EOF, and a
    // grandchild that inherited the pipe handle defers EOF for as long as it
    // lives -- joining here is what let a test's spawned process outlast the
    // deadline and unbound the whole run (`--timeout 3` taking 14s). Tree-killing
    // above normally EOFs the pipe promptly; detaching guarantees we proceed
    // within the deadline even if it does not. The thread is `'static`, owns its
    // pipe, and exits on EOF or when its send fails after `rx` drops -- so it is
    // bounded by the grandchild's lifetime, never by ours.
    drop(reader);
    let _ = child.wait();
    Ok(resume)
}

// ---------------------------------------------------------------------------
// Child side
// ---------------------------------------------------------------------------

/// The `__run-test-batch` entry point: run `names[start..]` in THIS process,
/// reporting each test's progress and result on stderr. Discovery is re-derived
/// here rather than passed in, because it is deterministic — the parent and the
/// child agree on the list and therefore on every index.
pub(crate) fn run_batch_child(
    path: PathBuf,
    start: usize,
    verbose: bool,
    nonce: &str,
    filter: Option<String>,
) -> Result<(), String> {
    let compiled = match compile(&path, SourceMode::Library) {
        Ok(compiled) => compiled,
        Err(failure) => {
            return Err(format_reports(
                &failure.reports,
                OutputMode::Concise,
                failure.source.as_deref(),
            ));
        }
    };
    // `false`: the parent already printed this suite's `skip` lines.
    let (names, _filtered_out) =
        discover_tests(&compiled.checked.program, filter.as_deref(), false);

    for (index, name) in names.iter().enumerate().skip(start) {
        eprintln!("##{nonce} {START} {index}");
        let result = run_named_function(&compiled.checked.program, name);
        // Flush the test's own output before reporting its result, so the parent
        // prints `PASS`/`FAIL` strictly after it.
        let _ = std::io::stdout().flush();
        match result {
            Ok(_) => eprintln!("##{nonce} {PASS} {index}"),
            Err(error) => {
                let mut payload = escape(&error.message);
                if verbose {
                    for frame in &error.traceback {
                        let text = match frame.span {
                            Some(span) => {
                                format!("    at {} ({}:{})", frame.function, span.line, span.column)
                            }
                            None => format!("    at {}", frame.function),
                        };
                        payload.push(FIELD_SEPARATOR);
                        payload.push_str(&escape(&text));
                    }
                }
                eprintln!("##{nonce} {FAIL} {index} {payload}");
            }
        }
    }
    // Report completion explicitly: the parent must not have to infer it from
    // stderr EOF, which a grandchild holding the inherited pipe can defer.
    eprintln!("##{nonce} {DONE} 0");
    Ok(())
}
