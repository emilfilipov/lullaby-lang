//! Semantic validation tests for inline-`asm` operand binding: the `L0443`
//! operand-shape law and the `L0461` register/value width law. The raw-byte
//! `asm` cases (`L0330` unsafe-gating, `L0425` byte range) live in
//! `semantics_tests.rs`; these cover only the operand/clobber clauses.

use lullaby_lexer::lex;
use lullaby_parser::parse;

use super::*;

fn diags(source: &str) -> Vec<SemanticDiagnostic> {
    let tokens = lex(source).expect("lex");
    let program = parse(&tokens).expect("parse");
    validate(&program).err().unwrap_or_default()
}

fn has(source: &str, code: &str) -> bool {
    diags(source).iter().any(|d| d.code == code)
}

/// A well-formed operand block (Linux `write` syscall shape) type-checks: 64-bit
/// registers, i64 values, distinct bindings, caller-saved clobbers.
#[test]
fn valid_operand_block_type_checks() {
    let source = concat!(
        "fn sys_write fd i64 buf ptr<i64> len i64 -> i64\n",
        "    let ret i64 = 0\n",
        "    unsafe\n",
        "        asm 15, 5\n",
        "            in rax = 1\n",
        "            in rdi = fd\n",
        "            in rsi = buf\n",
        "            in rdx = len\n",
        "            out rax = ret\n",
        "            clobber rcx\n",
        "            clobber r11\n",
        "    ret\n",
    );
    let d = diags(source);
    assert!(d.is_empty(), "expected no diagnostics, got {d:?}");
}

/// An unknown register name in an operand is `L0443`.
#[test]
fn unknown_register_is_l0443() {
    let source = concat!(
        "fn main -> i64\n",
        "    let r i64 = 0\n",
        "    unsafe\n",
        "        asm 144\n",
        "            out zzz = r\n",
        "    r\n",
    );
    assert!(has(source, "L0443"), "unknown register must be L0443");
}

/// An `out` target that is not an lvalue (a literal) is `L0443`.
#[test]
fn out_to_non_lvalue_is_l0443() {
    let source = concat!(
        "fn main -> i64\n",
        "    unsafe\n",
        "        asm 144\n",
        "            out rax = 5\n",
        "    0\n",
    );
    assert!(has(source, "L0443"), "`out` to a literal must be L0443");
}

/// Two `out`s to the same architectural register (even via different widths) is
/// `L0443` — the two writes would race for the register's value.
#[test]
fn duplicate_same_direction_binding_is_l0443() {
    let source = concat!(
        "fn main -> i64\n",
        "    let r i64 = 0\n",
        "    let s i64 = 0\n",
        "    unsafe\n",
        "        asm 144\n",
        "            out rax = r\n",
        "            out rax = s\n",
        "    r\n",
    );
    assert!(
        has(source, "L0443"),
        "two `out`s to the same register must be L0443"
    );
}

/// A register carrying BOTH an `in` and an `out` (the syscall-style read+write
/// pattern: `in rax` = number, `out rax` = result) is allowed, not a duplicate.
#[test]
fn in_and_out_on_same_register_is_allowed() {
    let source = concat!(
        "fn syscall0 nr i64 -> i64\n",
        "    let ret i64 = 0\n",
        "    unsafe\n",
        "        asm 15, 5\n",
        "            in rax = nr\n",
        "            out rax = ret\n",
        "            clobber rcx\n",
        "            clobber r11\n",
        "    ret\n",
    );
    assert!(
        !has(source, "L0443"),
        "`in rax` + `out rax` is the read+write pattern, not a duplicate: {:?}",
        diags(source)
    );
}

/// A register bound as an operand may not also be declared clobbered (`L0443`).
#[test]
fn operand_register_also_clobbered_is_l0443() {
    let source = concat!(
        "fn main x i64 -> i64\n",
        "    unsafe\n",
        "        asm 144\n",
        "            in rax = x\n",
        "            clobber rax\n",
        "    0\n",
    );
    assert!(
        has(source, "L0443"),
        "an operand register also clobbered must be L0443"
    );
}

/// Clobbering the stack or base pointer is `L0443` (the frame depends on them).
#[test]
fn clobbering_frame_register_is_l0443() {
    let source = concat!(
        "fn main -> i64\n",
        "    unsafe\n",
        "        asm 144\n",
        "            clobber rsp\n",
        "    0\n",
    );
    assert!(has(source, "L0443"), "clobbering rsp must be L0443");
}

/// A sub-width register in an `out` binding is a width mismatch (`L0461`): the
/// operand register must be 64-bit.
#[test]
fn out_to_sub_width_register_is_l0461() {
    let source = concat!(
        "fn main -> i64\n",
        "    let r i64 = 0\n",
        "    unsafe\n",
        "        asm 144\n",
        "            out al = r\n",
        "    r\n",
    );
    assert!(
        has(source, "L0461"),
        "an `out al = <i64>` binding (8-bit register) must be L0461"
    );
}

/// A narrow-typed value bound to a 64-bit register is a width mismatch (`L0461`).
#[test]
fn narrow_value_to_64bit_register_is_l0461() {
    let source = concat!(
        "fn main -> i64\n",
        "    let b u8 = to_u8(1)\n",
        "    unsafe\n",
        "        asm 144\n",
        "            in rax = b\n",
        "    0\n",
    );
    assert!(
        has(source, "L0461"),
        "binding a `u8` (8-bit value) to `rax` (64-bit) must be L0461"
    );
}

/// A pointer and the 64-bit unsigned/size integers are all valid 64-bit operand
/// values (no `L0461`).
#[test]
fn pointer_and_wide_integers_are_valid_operand_values() {
    let source = concat!(
        "fn main p ptr<i64> a u64 b isize c usize -> i64\n",
        "    unsafe\n",
        "        asm 144\n",
        "            in rax = p\n",
        "            in rbx = a\n",
        "            in rcx = b\n",
        "            in rdx = c\n",
        "    0\n",
    );
    assert!(
        !has(source, "L0461"),
        "ptr/u64/isize/usize are 64-bit operand values: {:?}",
        diags(source)
    );
}
