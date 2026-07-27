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

// ---------------------------------------------------------------------------
// OPERAND VISIBILITY — every semantic pass that walks expressions must walk an
// `asm` operand block's expressions too.
//
// `Stmt::Asm` was matched as `Stmt::Asm { .. } => {}` alongside genuinely
// childless statements (`Break`/`Continue`/`Return(None)`) in six passes, so the
// ordinary expressions inside `in <reg> = <expr>` / `out <reg> = <lvalue>` were
// invisible to all of them. The consequences were real, not theoretical: a
// `no-runtime` module smuggled a heap allocation past `L0441` into a freestanding
// binary, a use-after-free walked past `L0350`, and a use-after-send past
// `L0357`. Each case below is the operand-block form of a violation the pass
// already rejects when it is written OUTSIDE an `asm`; the paired "outside"
// assertion pins that equivalence so neither half can rot alone.
//
// INJECT-THE-BUG TEETH (verified): restoring any one pass's
// `Stmt::Asm { .. } => {}` arm makes exactly that pass's test below fail while
// the rest stay green.
// ---------------------------------------------------------------------------

/// `L0441` (freestanding tier): a heap-typed value hidden in an operand.
/// `to_string(...)` builds a real `string` on the host allocator — hard rule #1
/// of the `no-runtime` tier is no hidden allocation.
#[test]
fn no_runtime_rejects_heap_value_in_asm_operand() {
    let smuggled = concat!(
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
    let plain = concat!(
        "no-runtime\n",
        "\n",
        "fn main -> i64\n",
        "    len(to_string(12345))\n",
    );
    assert!(
        has(plain, "L0441"),
        "baseline: `to_string` outside an asm operand is already L0441"
    );
    assert!(
        has(smuggled, "L0441"),
        "a heap allocation inside an asm operand must not evade L0441: {:?}",
        diags(smuggled)
    );
}

/// `L0441`: the by-name host-allocator builtin check (`alloc`/`dealloc`/
/// `share`/`shared_get`) reaches into an operand too. This is a different code
/// path from the value-type check above — it fires on the call's NAME.
#[test]
fn no_runtime_rejects_alloc_builtin_in_asm_operand() {
    let source = concat!(
        "no-runtime\n",
        "\n",
        "fn main -> i64\n",
        "    let y i64 = 0\n",
        "    unsafe\n",
        "        asm 72, 137, 200\n",
        "            in rcx = ptr_to_int(alloc(8))\n",
        "            out rax = y\n",
        "    y\n",
    );
    assert!(
        has(source, "L0441"),
        "`alloc` inside an asm operand must be L0441: {:?}",
        diags(source)
    );
}

/// `L0350` (resource lifetime): reading a freed box through an operand.
#[test]
fn use_after_free_in_asm_operand_is_l0350() {
    let smuggled = concat!(
        "fn main -> i64\n",
        "    let y i64 = 0\n",
        "    unsafe\n",
        "        let p = alloc(8)\n",
        "        dealloc(p)\n",
        "        asm 72, 137, 200\n",
        "            in rcx = ptr_read(p)\n",
        "            out rax = y\n",
        "    y\n",
    );
    let plain = concat!(
        "fn main -> i64\n",
        "    unsafe\n",
        "        let p = alloc(8)\n",
        "        dealloc(p)\n",
        "        ptr_read(p)\n",
    );
    assert!(
        has(plain, "L0350"),
        "baseline: the same read outside an asm operand is already L0350"
    );
    assert!(
        has(smuggled, "L0350"),
        "a use-after-free inside an asm operand must not evade L0350: {:?}",
        diags(smuggled)
    );
}

/// `L0357` (actor message ownership): reading a moved-away value through an
/// operand.
#[test]
fn use_after_send_in_asm_operand_is_l0357() {
    let source = concat!(
        "actor Sink\n",
        "    state\n",
        "        log string\n",
        "\n",
        "    on take msg string\n",
        "        log = msg\n",
        "\n",
        "fn main -> i64\n",
        "    let s Actor<Sink> = spawn Sink()\n",
        "    let m string = \"hello\"\n",
        "    let y i64 = 0\n",
        "    tell s.take(m)\n",
        "    unsafe\n",
        "        asm 72, 137, 200\n",
        "            in rcx = len(m)\n",
        "            out rax = y\n",
        "    y\n",
    );
    assert!(
        has(source, "L0357"),
        "a use-after-send inside an asm operand must not evade L0357: {:?}",
        diags(source)
    );
}

/// Constant folding reaches into an operand. Folding runs before the type
/// checker and IR lowering, and nothing downstream is `const`-aware, so an
/// unfolded reference used to survive `check` and then die in IR lowering as
/// `L0501 unknown variable`. Assert on the folded AST, which is the pass's
/// actual product.
#[test]
fn const_reference_in_asm_operand_is_folded() {
    let source = concat!(
        "const SEED i64 = 41\n",
        "\n",
        "fn main -> i64\n",
        "    let y i64 = 0\n",
        "    unsafe\n",
        "        asm 72, 137, 200\n",
        "            in rcx = SEED\n",
        "            out rax = y\n",
        "    y\n",
    );
    assert!(
        matches!(sole_asm_in_operand(source).kind, ExprKind::Integer(41)),
        "a `const` named in an asm operand must fold to its literal, not survive \
         as a variable reference: {:?}",
        sole_asm_in_operand(source).kind
    );
}

/// Const-sized-array erasure reaches into an operand: a fill literal `[v; k]` is
/// expanded to an ordinary array literal, because no stage after this pass knows
/// what an `ExprKind::ArrayFill` is.
#[test]
fn array_fill_in_asm_operand_is_expanded() {
    let source = concat!(
        "fn main -> i64\n",
        "    let y i64 = 0\n",
        "    unsafe\n",
        "        asm 72, 137, 200\n",
        "            in rcx = len([7; 3])\n",
        "            out rax = y\n",
        "    y\n",
    );
    let ExprKind::Call { args, .. } = &sole_asm_in_operand(source).kind else {
        panic!("expected the operand to be a call");
    };
    let items = match &args[0].kind {
        ExprKind::Array(items) => items,
        other => panic!("a fill literal in an asm operand must be expanded, got {other:?}"),
    };
    assert_eq!(items.len(), 3, "`[7; 3]` expands to three elements");
}

/// The single `in` operand expression of the single `asm` statement in `main`,
/// taken from the program AFTER `validate` has run its rewriting passes.
fn sole_asm_in_operand(source: &str) -> Expr {
    let tokens = lex(source).expect("lex");
    let program = parse(&tokens).expect("parse");
    let checked = validate(&program).expect("validate");
    let main = checked
        .program
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("a `main` function");
    for stmt in &main.body {
        let Stmt::Unsafe { body, .. } = stmt else {
            continue;
        };
        for inner in body {
            let Stmt::Asm { operands, .. } = inner else {
                continue;
            };
            for operand in operands {
                if let AsmOperand::In { value, .. } = operand {
                    return value.clone();
                }
            }
        }
    }
    panic!("expected an `asm` statement with an `in` operand inside `unsafe`");
}
