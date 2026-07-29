//! CLI integration tests, part 27 — inline-`asm` OPERAND BINDING end-to-end.
//!
//! `asm` is native-only, so these compile a real `.exe` and check its exit code
//! (the interpreters reject `asm` with `L0425`, verified here too). Two behaviors
//! are pinned against a linked binary:
//!
//!   * a **pure-register round-trip** — `mov rax, rcx` with `in rcx = x` /
//!     `out rax = y` moves an input register to an output register, so `y == x`;
//!   * **callee-saved clobber preservation** — a caller that keeps a hot local in
//!     a callee-saved register (via register promotion) calls a function whose
//!     `asm` clobbers that register; the caller's value must survive, which it
//!     only does because the marshaller saves/restores the clobbered callee-saved
//!     register around the body.
//!
//! INJECT-THE-BUG TEETH (manually verified): removing steps 1/6 (the callee-saved
//! save/restore) from `native_object_asm.rs::lower_native_asm` makes
//! `native_asm_clobber_preserves_caller_value` return a corrupted value (the
//! caller's promoted `rbx` reads back as the clobber constant), and the codegen
//! byte pins in `crates/lullaby_ir/src/native_object_asm_tests.rs` fail. The
//! register-promotion exclusion's teeth are proven in that same file.

use crate::*;
use std::process::Command;

/// Build `source` to a real `.exe` and return its exit code, or `None` when this
/// host cannot produce/run one.
fn native_exit_for(source: &str, tag: &str) -> Option<i32> {
    if !cfg!(windows) {
        eprintln!("not a Windows host; skipping {tag}");
        return None;
    }
    let dir = ScratchDir::new("asm_native_exit");
    let src = dir.join(format!("{tag}.lby"));
    let exe = dir.join(format!("{tag}.exe"));
    std::fs::write(&src, source).expect("write source");
    let _ = std::fs::remove_file(&exe);

    let emit = lullaby()
        .args([
            "native",
            "-o",
            exe.to_str().expect("exe path"),
            src.to_str().expect("src path"),
        ])
        .output()
        .expect("run native");
    assert!(
        emit.status.success(),
        "native emit failed for {tag}:\n{source}\n{}",
        stderr(&emit)
    );
    assert!(
        exe.is_file(),
        "expected a native exe for {tag} (an operand `asm` must COMPILE, not skip):\n{}",
        stdout(&emit)
    );
    let run = Command::new(&exe).output().expect("run exe");
    Some(run.status.code().expect("exit code"))
}

/// A pure-register round-trip: `mov rax, rcx` (bytes `48 89 C8`) with `in rcx = x`
/// and `out rax = y` copies the input to the output, so the function returns its
/// argument unchanged.
#[test]
fn native_asm_register_roundtrip() {
    let source = concat!(
        "fn identity x i64 -> i64\n",
        "    let y i64 = 0\n",
        "    unsafe\n",
        "        asm 72, 137, 200\n", // mov rax, rcx
        "            in rcx = x\n",
        "            out rax = y\n",
        "    y\n",
        "\n",
        "fn main -> i64\n",
        "    identity(7)\n",
    );
    if let Some(exit) = native_exit_for(source, "asm_roundtrip") {
        assert_eq!(
            exit, 7,
            "`in rcx = x` / `mov rax, rcx` / `out rax = y` must round-trip x to y"
        );
    }
}

/// Callee-saved clobber preservation — the sharpest test. `compute` runs a hot
/// loop, so its accumulator is register-promoted into the callee-saved `rbx`.
/// Each iteration it calls `poke`, whose `asm` does `mov rbx, 222` and declares
/// `clobber rbx`. Because `rbx` is callee-saved, `poke` must preserve it, so
/// `compute`'s accumulator survives every call and the result is
/// `sum(0..3) = 0 + 1 + 2 = 3`. If `poke` failed to save/restore `rbx`, the
/// accumulator would be smashed to 222 on the first call and the exit code would
/// be far larger than 3 (that is the injected-bug signature).
#[test]
fn native_asm_clobber_preserves_caller_value() {
    let source = concat!(
        "fn poke -> i64\n",
        "    unsafe\n",
        "        asm 72, 199, 195, 222, 0, 0, 0\n", // mov rbx, 222
        "            clobber rbx\n",
        "    0\n",
        "\n",
        "fn compute n i64 -> i64\n",
        "    let acc i64 = 0\n",
        "    let i i64 = 0\n",
        "    while i < n\n",
        "        acc = acc + poke() + i\n",
        "        i = i + 1\n",
        "    acc\n",
        "\n",
        "fn main -> i64\n",
        "    compute(3)\n",
    );
    if let Some(exit) = native_exit_for(source, "asm_clobber_preserve") {
        assert_eq!(
            exit, 3,
            "compute's accumulator (promoted into callee-saved rbx) must survive \
             poke's `clobber rbx`; a corrupted value signals a missing save/restore"
        );
    }
}

/// Whether Docker with a working `linux/amd64` runtime is available.
fn docker_amd64_available() -> bool {
    Command::new("docker")
        .args([
            "run",
            "--rm",
            "--platform",
            "linux/amd64",
            "busybox",
            "true",
        ])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// The real Linux `syscall` proof: a program that drives `write(1, buf, len)` and
/// `exit(code)` entirely through operand-bound inline `asm`, compiled to an
/// x86-64 Linux ELF, linked with `ld.lld`, and run under `linux/amd64` Docker.
/// It writes the byte `'H'` to stdout and exits `42`, so the marshalling — syscall
/// number and arguments into `rax`/`rdi`/`rsi`/`rdx`, `rcx`/`r11` clobbered by the
/// `syscall` instruction — is proven by the container's stdout AND exit code, not
/// just by inspection. Gated on Docker+amd64 and `ld.lld`; skipped gracefully
/// otherwise (reporting what it did or did not run).
#[test]
fn native_asm_linux_syscall_write_and_exit_under_docker() {
    let Some(lld) = ld_lld_path() else {
        eprintln!("ld.lld not found; skipping Linux syscall asm test (ran nothing)");
        return;
    };
    if !docker_amd64_available() {
        eprintln!("Docker linux/amd64 unavailable; skipping Linux syscall asm test (ran nothing)");
        return;
    }
    // write(1, &buf, 1) then exit(42), both via operand `asm` over the SysV
    // syscall convention (nr in rax, args in rdi/rsi/rdx, syscall clobbers rcx/r11).
    let source = concat!(
        "fn main -> i64\n",
        "    let buf array<i64> = [72]\n", // 72 = 'H'
        "    unsafe\n",
        "        let p ptr<i64> = addr_of(buf[0])\n",
        "        asm 15, 5\n",      // syscall
        "            in rax = 1\n", // __NR_write
        "            in rdi = 1\n", // fd = stdout
        "            in rsi = p\n", // buf
        "            in rdx = 1\n", // len = 1 byte
        "            clobber rcx\n",
        "            clobber r11\n",
        "        asm 15, 5\n",       // syscall
        "            in rax = 60\n", // __NR_exit
        "            in rdi = 42\n", // exit code
        "            clobber rcx\n",
        "            clobber r11\n",
        "    0\n",
    );

    let scratch = ScratchDir::new("asm_linux_syscall");
    let dir = scratch.join("work");
    std::fs::create_dir_all(&dir).expect("create work dir");
    let src = dir.join("syscall.lby");
    let obj = dir.join("syscall.o");
    let exe = dir.join("syscall");
    std::fs::write(&src, source).expect("write source");

    // 1. Emit the x86-64 Linux ELF object.
    let emit = lullaby()
        .args([
            "native",
            "--target",
            "x86_64-unknown-linux-gnu",
            "-o",
            obj.to_str().expect("obj path"),
            src.to_str().expect("src path"),
        ])
        .output()
        .expect("emit linux object");
    assert!(
        emit.status.success(),
        "x86-64 Linux emit failed:\n{}",
        stderr(&emit)
    );

    // 2. Link into an x86-64 ELF executable.
    let link = Command::new(&lld)
        .args([
            "-m",
            "elf_x86_64",
            "-o",
            exe.to_str().expect("exe path"),
            obj.to_str().expect("obj path"),
        ])
        .output()
        .expect("run ld.lld");
    assert!(
        link.status.success(),
        "ld.lld failed: {}",
        String::from_utf8_lossy(&link.stderr)
    );

    // 3. Run under linux/amd64 Docker (Windows bind mounts drop the exec bit, so
    //    copy + chmod inside the container).
    let mount = format!("{}:/w", dir.display());
    let run = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--platform",
            "linux/amd64",
            "-v",
            &mount,
            "busybox",
            "sh",
            "-c",
            "cp /w/syscall /s && chmod +x /s && /s",
        ])
        .output()
        .expect("docker run amd64");

    // 4. The syscall's observable effects: stdout is the written byte, exit is 42.
    let code = run.status.code().expect("container exit code");
    let out = String::from_utf8_lossy(&run.stdout);
    eprintln!("ran the Linux syscall asm exe under docker: stdout={out:?} exit={code}");
    assert_eq!(
        out,
        "H",
        "the `write(1, &buf, 1)` operand syscall must print 'H'; docker stderr: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        code, 42,
        "the `exit(42)` operand syscall must set the process exit code to 42"
    );
}

/// The interpreters cannot execute machine code, so an operand `asm` is refused
/// with `L0425` on every interpreter backend — exactly as the raw-byte form is.
#[test]
fn asm_operands_rejected_by_interpreters_with_l0425() {
    let source = concat!(
        "fn identity x i64 -> i64\n",
        "    let y i64 = 0\n",
        "    unsafe\n",
        "        asm 72, 137, 200\n",
        "            in rcx = x\n",
        "            out rax = y\n",
        "    y\n",
        "\n",
        "fn main -> i64\n",
        "    identity(7)\n",
    );
    let dir = ScratchDir::new("asm_interp_refuse");
    let src = dir.join("asm_interp.lby");
    std::fs::write(&src, source).expect("write source");
    for backend in ["ast", "ir", "bytecode"] {
        let out = lullaby()
            .args(["run", "--backend", backend, src.to_str().expect("src path")])
            .output()
            .expect("run interpreter");
        assert!(
            !out.status.success(),
            "the {backend} interpreter must refuse an operand `asm`, not run it"
        );
        assert!(
            stderr(&out).contains("L0425"),
            "the {backend} interpreter must refuse an operand `asm` with L0425:\n{}",
            stderr(&out)
        );
    }
}

/// OPERAND VISIBILITY, end-to-end: the demonstrated smuggle. A `no-runtime`
/// module built a real heap `string` inside an `asm` operand; `check` exited 0
/// and `native` emitted a runnable no-C-runtime direct PE, breaking hard rule #1
/// of the freestanding tier (no hidden allocation). Both the diagnostic and the
/// absence of an artifact are pinned — a rejected program must produce no exe.
#[test]
fn no_runtime_heap_smuggle_through_an_asm_operand_is_rejected_end_to_end() {
    let source = concat!(
        "no-runtime\n",
        "\n",
        "fn main -> i64\n",
        "    let y i64 = 0\n",
        "    unsafe\n",
        "        asm 72, 137, 200\n",
        "            in rcx = len(to_string(12345))\n",
        "            out rax = y\n",
        "    y\n",
    );
    let dir = ScratchDir::new("asm_no_runtime_smuggle");
    let src = dir.join("smuggle.lby");
    let exe = dir.join("smuggle.exe");
    std::fs::write(&src, source).expect("write source");

    let checked = lullaby()
        .args(["check", src.to_str().expect("src path")])
        .output()
        .expect("run check");
    assert!(
        !checked.status.success(),
        "a heap allocation inside an asm operand must not pass `check` in a \
         `no-runtime` module:\n{}{}",
        stdout(&checked),
        stderr(&checked)
    );
    assert!(
        stderr(&checked).contains("L0441"),
        "the freestanding-tier gate must fire (L0441):\n{}",
        stderr(&checked)
    );

    let emitted = lullaby()
        .args([
            "native",
            "-o",
            exe.to_str().expect("exe path"),
            src.to_str().expect("src path"),
        ])
        .output()
        .expect("run native");
    assert!(
        !emitted.status.success(),
        "`native` must refuse the rejected program:\n{}{}",
        stdout(&emitted),
        stderr(&emitted)
    );
    assert!(
        !exe.is_file(),
        "no freestanding binary may be produced for an L0441-rejected program"
    );
}

/// OPERAND VISIBILITY, loader half: the module reference collector skipped
/// `Stmt::Asm`, so a call to a *private* function in another package hid inside
/// `in <reg> = f(x)` and evaded the `L0392` visibility check — the same code the
/// identical call raises when written as an ordinary statement
/// (`rejects_cross_package_private_use_with_l0392` in `cli.rs`). The semantic
/// half of this class is pinned in
/// `crates/lullaby_semantics/src/semantics_asm_tests.rs`.
#[test]
fn rejects_cross_package_private_use_from_an_asm_operand_with_l0392() {
    let project = workspace_root().join("tests/fixtures/invalid/project_private_cross_asm/app");
    let output = lullaby()
        .args(["check", project.to_str().expect("project path")])
        .output()
        .expect("run cli");
    assert!(
        !output.status.success(),
        "a private cross-package call inside an asm operand must be REJECTED: {output:?}"
    );
    let stderr = stderr(&output);
    assert!(stderr.contains("L0392 [loader error]"), "{stderr}");
    assert!(stderr.contains("hidden_helper"), "{stderr}");
}

/// CALLEE SHADOWING, native half: `lullaby native` runs
/// `OptimizationConfig::inlining()` — the inliner is the *whole* native
/// pipeline — so its unscoped module-function lookup did not just make
/// `--optimize full` answer 400, it baked 400 into a shipped binary while
/// `--optimize none` on every interpreter answered 42.
///
/// `lullaby wasm` runs the same config but was NOT exposed: the WASM emitter
/// skips any function carrying a `fn`-typed local, so this program's `main` is
/// never exported (measured — `wasmi` reports `ExportedFuncNotFound`, not a wrong
/// value). Native is the only compiled tier that shipped the bug.
///
/// The native *lowering* already resolved shadowing correctly
/// (`native_object_expr.rs`); only the pass ahead of it did not, which is why
/// this needs its own pin rather than riding on the interpreter parity test
/// (`inlining_honors_a_local_binding_that_shadows_a_module_function` in
/// `crates/lullaby_ir/src/ir_lib_tests.rs`).
///
/// INJECT-THE-BUG TEETH (verified): removing the `!self.shadowed.contains(name)`
/// guard from `Inliner::inline_expr` makes this exe exit 400 instead of 42.
#[test]
fn native_honors_a_shadowed_callee_name() {
    let source = concat!(
        "fn inner v i64 -> i64\n",
        "    v * 10\n",
        "\n",
        "fn main -> i64\n",
        "    let n i64 = 2\n",
        "    let inner fn(i64) -> i64 = fn x i64 -> x + n\n",
        "    inner(40)\n",
    );
    let Some(code) = native_exit_for(source, "shadowed_callee") else {
        return;
    };
    assert_eq!(
        code, 42,
        "the local `inner` closure is the callee; 400 means the module `inner` \
         body was inlined over it"
    );
}
