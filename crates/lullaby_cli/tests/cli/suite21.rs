//! CLI integration tests, part 21 — three semantics fixes that close frontend
//! holes: model-preserving `ptr_cast`, the `L0350` simple-alias use-after-free
//! case, and void `export fn`.
//!
//! # `ptr_cast` and the two pointer models
//!
//! Lullaby has two non-convertible pointer models — the legacy `ptr_T` heap box
//! that only `alloc` produces (a heap-SLOT INDEX over a one-cell store on the
//! interpreters) and the modern `ptr<T>` address from `addr_of`/`int_to_ptr`.
//! `let`/parameter binding enforced that (`L0303`/`L0313`); `ptr_cast` did not,
//! because it derived its result type purely from the caller's annotation and never
//! from the operand. Both laundering directions are pinned below, plus the
//! negative control that a legitimate `ptr<T>` retarget still works.

use crate::*;

/// Run `source` and return `(exit code, stdout+stderr)`.
fn run_backend(source: &str, backend: &str, tag: &str) -> (i32, String) {
    let dir = std::env::temp_dir();
    let src = dir.join(format!("{tag}_{backend}.lby"));
    std::fs::write(&src, source).expect("write source");
    let out = lullaby()
        .args(["run", "--backend", backend, src.to_str().expect("src path")])
        .output()
        .expect("run backend");
    (
        out.status.code().expect("exit code"),
        format!("{}{}", stdout(&out), stderr(&out)),
    )
}

/// Assert `source` is REJECTED by every interpreter frontend with `code`. A
/// frontend diagnostic is tier-independent, so all three must agree exactly.
fn assert_all_interpreters_reject(source: &str, tag: &str, code: &str) {
    for backend in ["ast", "ir", "bytecode"] {
        let (exit, output) = run_backend(source, backend, tag);
        assert_ne!(
            exit, 0,
            "{backend} must REJECT this program for {tag}:\n{source}\n{output}"
        );
        assert!(
            output.contains(code),
            "{backend} must reject {tag} with {code}:\n{source}\n{output}"
        );
    }
}

/// Assert every interpreter accepts `source` and prints `expected`.
fn assert_all_interpreters_yield(source: &str, tag: &str, expected: i64) {
    for backend in ["ast", "ir", "bytecode"] {
        let (exit, output) = run_backend(source, backend, tag);
        assert_eq!(
            exit, 0,
            "{backend} must ACCEPT this program for {tag}:\n{source}\n{output}"
        );
        assert_eq!(
            output.trim(),
            expected.to_string(),
            "{backend} must print {expected} for {tag}:\n{source}"
        );
    }
}

/// THE laundering repro: `ptr_cast` used to rewrite an `alloc` box (`ptr_i64`) into
/// a raw address (`ptr<i64>`) purely because the annotation said so. That defeated
/// the `L0303`/`L0313` walls and let `ptr_offset` (below) type-check over a
/// one-cell box. The operand's model must win, so this is now `L0303`.
#[test]
fn ptr_cast_cannot_launder_an_alloc_box_into_a_raw_pointer() {
    assert_all_interpreters_reject(
        concat!(
            "fn main -> i64\n",
            "    unsafe\n",
            "        let p = alloc(8)\n",
            "        let q ptr<i64> = ptr_cast(p)\n",
            "        ptr_read(q)\n",
        ),
        "lullaby_ptr_cast_launder_box",
        "L0303",
    );
}

/// The memory-corruption shape the laundering enabled: once the box is spelled
/// `ptr<i64>`, `ptr_offset(q, 1)` type-checks. The interpreters refuse it at RUN
/// time (`L0406`) and the native gate refuses it, but the frontend accepted the
/// program — natively this strides 8 bytes off a one-cell payload into the next
/// heap block's `[size]` header, the word the allocator's free-list scan reads.
/// Now it never gets past the checker.
#[test]
fn laundered_box_pointer_arithmetic_is_rejected_at_the_frontend() {
    assert_all_interpreters_reject(
        concat!(
            "fn main -> i64\n",
            "    unsafe\n",
            "        let p = alloc(8)\n",
            "        let q ptr<i64> = ptr_cast(p)\n",
            "        let r = ptr_offset(q, 1)\n",
            "        ptr_to_int(r)\n",
        ),
        "lullaby_ptr_cast_launder_offset",
        "L0303",
    );
}

/// The REVERSE direction, which the original report did not cover: a legacy `ptr_U`
/// annotation used to capture an `addr_of` address, relabelling a real machine
/// address as an `alloc` box. That falsifies the invariant the native backend's
/// `is_legacy_box_pointer` spelling test rests on — that a `ptr_T`-typed expression
/// is always `alloc`-derived. The model is taken from the operand, so this is
/// `L0303` too.
#[test]
fn ptr_cast_cannot_relabel_a_raw_pointer_as_an_alloc_box() {
    assert_all_interpreters_reject(
        concat!(
            "fn main -> i64\n",
            "    unsafe\n",
            "        let buf array<i64> = [10, 20, 30]\n",
            "        let base = addr_of(buf[0])\n",
            "        let fake ptr_i64 = ptr_cast(base)\n",
            "        ptr_read(fake)\n",
        ),
        "lullaby_ptr_cast_relabel_address",
        "L0303",
    );
}

/// NEGATIVE CONTROL: model-preservation must not break `ptr_cast` on legitimate
/// `ptr<T>` operands. Retargeting the pointee within the modern model — the
/// `addr_of` -> `ptr_cast<u8>` -> back -> read idiom — still works on every tier.
/// If this fails, the fix over-reached.
#[test]
fn ptr_cast_still_retargets_a_genuine_raw_pointer_pointee() {
    assert_all_interpreters_yield(
        concat!(
            "fn main -> i64\n",
            "    unsafe\n",
            "        let buf array<i64> = [10, 20, 30]\n",
            "        let base = addr_of(buf[0])\n",
            "        let bp ptr<u8> = ptr_cast(base)\n",
            "        let back ptr<i64> = ptr_cast(bp)\n",
            "        ptr_read(back)\n",
        ),
        "lullaby_ptr_cast_roundtrip",
        10,
    );
}

/// An identity cast of a box stays legal and stays a box: `ptr_cast` of a `ptr_T`
/// yields exactly `ptr_T`, so inference binds it and the box still reads back. This
/// pins that the fix preserves rather than rejects — existing box-cast source that
/// did not launder keeps compiling.
#[test]
fn ptr_cast_of_an_alloc_box_is_an_identity_that_stays_a_box() {
    assert_all_interpreters_yield(
        concat!(
            "fn main -> i64\n",
            "    unsafe\n",
            "        let p = alloc(42)\n",
            "        let q = ptr_cast(p)\n",
            "        ptr_read(q)\n",
        ),
        "lullaby_ptr_cast_box_identity",
        42,
    );
}
