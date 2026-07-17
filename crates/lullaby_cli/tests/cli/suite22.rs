//! CLI integration tests, part 22 — **packed narrow array elements**: walking a
//! narrow-element buffer (`array<i32>`, `array<u8>`, `array<i16>`, …) through raw
//! pointers (`road_to_1_0_stable.md` C3, "accepted limitations" #1).
//!
//! # The gap these close
//!
//! `addr_of(a[0])` + `ptr_offset` over an `array<i64>` worked on all four tiers,
//! but a NARROW-element array was refused natively (`L0339`): the element cell was
//! 8 bytes while `ptr_offset` strides `size_of(T)` (4 for `i32`, 1 for `u8`), so a
//! walk would desynchronize. Refusing was correct — a silent desync is memory
//! corruption — but it meant **a byte buffer could not be walked**, which is most
//! of what a driver does.
//!
//! Narrow-element arrays are now PACKED at the element's C width, so the stride and
//! the storage agree and the walk is C-compatible.
//!
//! # Why these are parity assertions, not native-only claims
//!
//! The interpreters ALREADY define these programs: `RawPointerMemory` builds an
//! `addr_of` region whose stride is `Value::layout_size` of the element — 4 for
//! `i32`, 1 for `byte` (see `crates/lullaby_runtime/src/raw_pointer.rs`). So the
//! interpreters answer a size law of 4 for an `array<i32>` walk, and native must
//! answer the same number or not compile the program at all. That is what makes
//! packing FORCED rather than chosen: an 8-byte-cell native layout would answer 8,
//! giving a defined program two answers.
//!
//! Every test here therefore runs the fixture on ast/ir/bytecode AND on a real
//! compiled `.exe`, and asserts they agree. A native-only assertion would prove
//! nothing about the property at issue.
//!
//! # What a wrong stride would look like
//!
//! [`narrow_array_write_through_matches_the_interpreters`] is the load-bearing one:
//! it writes THROUGH the pointer walk and reads back BY INDEX. A stride that
//! disagreed with the element layout would write to every other element (and past
//! the array's end) while the index reads saw zeros — so the two paths naming the
//! same storage is exactly what it pins.

use crate::*;

/// Run a valid fixture on `backend` and return the captured output.
fn run_backend(fixture: &str, backend: &str) -> std::process::Output {
    let path = workspace_root().join(fixture);
    lullaby()
        .args([
            "run",
            "--backend",
            backend,
            path.to_str().expect("fixture path"),
        ])
        .output()
        .expect("run cli")
}

/// Compile `fixture` natively (asserting `main` is NOT skipped), run the produced
/// `.exe`, and return its exit code. `None` on a non-Windows host, where the COFF
/// output cannot be executed.
fn native_exit_code(fixture: &str, out_name: &str, extra_args: &[&str]) -> Option<i32> {
    let path = workspace_root().join(fixture);
    let out = std::env::temp_dir().join(out_name);
    let _ = std::fs::remove_file(&out);

    let out_str = out.to_str().expect("out path").to_string();
    let fixture_str = path.to_str().expect("fixture path").to_string();
    let mut args: Vec<&str> = vec!["native", "--verbose", "-o", &out_str];
    args.extend_from_slice(extra_args);
    args.push(&fixture_str);

    let emit = lullaby().args(&args).output().expect("run native");
    assert!(
        emit.status.success(),
        "{fixture} must compile natively (no L0339 skip): {}",
        stderr(&emit)
    );
    // The whole point is that these no longer SKIP. Without this assertion a
    // regression to the old `L0339` refusal would leave the test silently passing
    // on the interpreter results alone.
    assert!(
        stdout(&emit).contains("compiled main"),
        "{fixture}: `main` must be native-eligible, not skipped: {}{}",
        stdout(&emit),
        stderr(&emit)
    );

    if !cfg!(windows) {
        eprintln!("not a Windows host; skipping the run of {out_name}");
        return None;
    }
    assert!(out.is_file(), "{fixture}: native exe was not written");
    let run = std::process::Command::new(&out)
        .output()
        .expect("run native exe");
    Some(run.status.code().expect("native exe exit code"))
}

/// Assert `fixture` produces `expected` on all three interpreters AND from a real
/// compiled `.exe` — the four-tier agreement that is the actual claim.
fn assert_all_tiers_agree(fixture: &str, out_name: &str, extra_args: &[&str], expected: i64) {
    for backend in ["ast", "ir", "bytecode"] {
        let output = run_backend(fixture, backend);
        assert!(
            output.status.success(),
            "[{backend}] {fixture} should run: {}",
            stderr(&output)
        );
        assert_eq!(
            stdout(&output).trim(),
            expected.to_string(),
            "[{backend}] {fixture} should compute {expected}"
        );
    }
    let Some(exit) = native_exit_code(fixture, out_name, extra_args) else {
        return;
    };
    assert_eq!(
        exit, expected as i32,
        "native must agree with all three interpreters on {fixture}"
    );
}

/// THE HEADLINE: an `array<i32>` walk via `addr_of(a[0])` + `ptr_offset`, the size
/// law answering `size_of(i32) == 4`, and a negative offset stepping back.
///
/// This fixture is the one that pinned the gap: it exited 18 on all three
/// interpreters while native refused it (`L0339`) because the element cell was 8
/// bytes. Native striding by the CELL size instead would answer 22 — the same
/// defined program with two answers, which is why the cell-stride design was
/// rejected.
#[test]
fn narrow_i32_walk_matches_the_interpreters() {
    assert_all_tiers_agree(
        "tests/fixtures/valid/raw_ptr_addressing.lby",
        "lullaby_narrow_i32_walk.exe",
        &[],
        18,
    );
}

/// A BYTE-BUFFER walk — the driver idiom. `array<u8>` packs one byte per element,
/// so the walk strides 1 and the size law answers `size_of(u8) == 1`.
#[test]
fn narrow_u8_byte_buffer_walk_matches_the_interpreters() {
    assert_all_tiers_agree(
        "tests/fixtures/valid/narrow_array_u8_walk.lby",
        "lullaby_narrow_u8_walk.exe",
        &[],
        101,
    );
}

/// WRITE through the walk, read back BY INDEX — the shape a wrong stride cannot
/// survive (see the module docs). If `ptr_offset` strided 8 over 4-byte elements,
/// the writes would land on every other element and past the array's end while the
/// index reads saw zeros.
#[test]
fn narrow_array_write_through_matches_the_interpreters() {
    assert_all_tiers_agree(
        "tests/fixtures/valid/narrow_array_write_through.lby",
        "lullaby_narrow_write_through.exe",
        &[],
        42,
    );
}

/// The size law at the remaining packed widths (`i16`/`i8`/`u16`) plus WHOLE-ARRAY
/// DECAY: `addr_of(a)` on an `array<i16>` must yield the same address as
/// `addr_of(a[0])` and stride the same way.
#[test]
fn narrow_array_widths_and_decay_match_the_interpreters() {
    assert_all_tiers_agree(
        "tests/fixtures/valid/narrow_array_widths.lby",
        "lullaby_narrow_widths.exe",
        &[],
        83,
    );
}

/// THE POINT OF THE WHOLE FEATURE: a `no-runtime` module walking a BYTE buffer
/// under `--freestanding` — a driver's idiom, with no host allocator and no
/// runtime. Before packing, `array<u8>` was 8-byte cells and `addr_of(buf[0])` was
/// refused, so a byte buffer could not be walked natively at all.
#[test]
fn freestanding_byte_buffer_walk_matches_the_interpreters() {
    assert_all_tiers_agree(
        "tests/fixtures/valid/no_runtime/freestanding_byte_buffer_walk.lby",
        "lullaby_freestanding_byte_walk.exe",
        &["--freestanding"],
        120,
    );
}

/// The NEGATIVE case, and the boundary of the feature: a narrow SCALAR is still a
/// normalized 8-byte cell, so `addr_of` of one is still refused (`L0339`) — a
/// 4-byte store through its address would leave the cell's upper half stale.
///
/// Pinned here next to the positive cases because the two spell the same type name
/// (`i32`): only the resolved LAYOUT separates them (the width-agreement law in
/// `native_object_rawptr.rs`). If a change ever admitted the scalar, this fails
/// rather than the backend silently handing out a pointer whose width disagrees
/// with its storage.
#[test]
fn addr_of_a_narrow_scalar_is_still_refused_natively() {
    let path = workspace_root().join("tests/fixtures/native_only/addr_of_narrow_scalar.lby");
    let out = std::env::temp_dir().join("lullaby_narrow_scalar_skip.exe");
    let _ = std::fs::remove_file(&out);
    let output = lullaby()
        .args([
            "native",
            "--verbose",
            "-o",
            out.to_str().expect("out path"),
            path.to_str().expect("fixture path"),
        ])
        .output()
        .expect("run native");
    let errors = stderr(&output);
    assert!(
        !output.status.success(),
        "addr_of of a narrow SCALAR must skip, not compile: {}",
        stdout(&output)
    );
    assert!(
        errors.contains("L0339"),
        "the narrow-scalar refusal must be the clean L0339 skip: {errors}"
    );
    assert!(
        errors.contains("storage is 8 bytes wide but the pointee `i32` is 4"),
        "the skip must name the width disagreement that causes it: {errors}"
    );
}
