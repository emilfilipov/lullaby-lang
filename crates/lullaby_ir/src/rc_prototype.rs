//! Scope-based reference-counting — PROTOTYPE.
//!
//! This module de-risks the *drop-insertion mechanics* of the native memory model
//! chosen in `documents/memory_model_decision.md` (reference counting + arenas). It
//! is deliberately **not wired into the backend yet**: it emits and verifies the
//! three primitive refcount operations the real drop-insertion pass (stage 2 of the
//! plan) will call, so that when that pass lands, the runtime ops it targets are
//! already proven-correct machine code.
//!
//! ## Record layout
//!
//! An rc-managed heap block is `[refcount: i64][payload…]`. The pointer names the
//! block (refcount at offset 0, payload at +8). A fresh block is initialised with a
//! refcount of 1 by the allocator (or [`emit_rc_init`]).
//!
//! ## The three primitives (Win64: pointer in `rcx`; leaf, preserve nothing)
//!
//! * [`emit_rc_init`] — `mov qword [rcx], 1`: seed a fresh block's count.
//! * [`emit_rc_inc`]  — `inc qword [rcx]`: a value was copied/aliased (a *bind*).
//! * [`emit_rc_dec`]  — `dec qword [rcx]; jnz keep; call __lullaby_rc_free; keep:`:
//!   a value left its scope (a *drop*). Frees exactly when the last reference dies.
//!
//! ## Why this shape de-risks drop insertion
//!
//! The hard part of the real pass is emitting a `dec` on **every** scope-exit edge
//! (fallthrough, `return`, `?`/`throw`, `match` arms) exactly once. Each such edge
//! becomes a single `call __lullaby_rc_dec` with the local's pointer in `rcx`; the
//! branch/free logic lives *inside* `rc_dec`, so the codegen pass only has to place
//! calls — it never re-implements the free decision. This module proves that call
//! target is correct; the placement is stage 2.
//!
//! The allocator with a free-list (stage 1 runtime) and the escape-analysis-driven
//! arena fast path (stage 4) are separate increments; see the decision record.

/// Symbol the prototype `rc_dec` calls when a refcount reaches zero. Stage 1 will
/// emit its body (free-list push); here it is only the call target under test.
pub const RC_FREE_SYMBOL: &str = "__lullaby_rc_free";

/// `mov qword ptr [rcx], 1` — initialise a fresh block's refcount to 1.
pub fn emit_rc_init(code: &mut Vec<u8>) {
    // REX.W C7 /0 id : mov r/m64, imm32 (sign-extended). ModRM 0x01 = [rcx], reg 0.
    code.extend_from_slice(&[0x48, 0xC7, 0x01, 0x01, 0x00, 0x00, 0x00]);
}

/// `inc qword ptr [rcx]` — a new reference to the block (bind/copy of an rc value).
pub fn emit_rc_inc(code: &mut Vec<u8>) {
    // REX.W FF /0 : inc r/m64. ModRM 0x01 = [rcx].
    code.extend_from_slice(&[0x48, 0xFF, 0x01]);
}

/// `dec qword ptr [rcx]; jnz keep; call __lullaby_rc_free; keep: ret` — a drop.
/// Decrements the refcount and frees the block iff it reached zero, then returns.
/// The `call` records a relocation against [`RC_FREE_SYMBOL`] (pushed onto
/// `relocations`, matching the backend's other `.text` helper calls). `rcx` (the
/// block pointer) is the free helper's argument and is preserved by the `dec`.
pub fn emit_rc_dec(code: &mut Vec<u8>, relocations: &mut Vec<(u32, String)>) {
    // REX.W FF /1 : dec r/m64. ModRM 0x09 = [rcx], reg 1.
    code.extend_from_slice(&[0x48, 0xFF, 0x09]); // dec qword [rcx]
    code.extend_from_slice(&[0x0F, 0x85]); // jnz keep (rel32; count != 0 -> live)
    let keep_site = code.len();
    code.extend_from_slice(&[0, 0, 0, 0]);
    // count == 0: free the block (pointer already in rcx).
    code.push(0xE8); // call rel32
    relocations.push((code.len() as u32, RC_FREE_SYMBOL.to_string()));
    code.extend_from_slice(&[0, 0, 0, 0]);
    // keep:
    let rel = (code.len() as i32) - (keep_site as i32 + 4);
    code[keep_site..keep_site + 4].copy_from_slice(&rel.to_le_bytes());
    code.push(0xC3); // ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rc_init_inc_dec_emit_the_expected_refcount_ops() {
        // init: mov qword [rcx], 1
        let mut c = Vec::new();
        emit_rc_init(&mut c);
        assert_eq!(c, [0x48, 0xC7, 0x01, 0x01, 0x00, 0x00, 0x00]);

        // inc: inc qword [rcx]
        let mut c = Vec::new();
        emit_rc_inc(&mut c);
        assert_eq!(c, [0x48, 0xFF, 0x01]);

        // dec: dec qword [rcx] ; jnz keep ; call rc_free ; keep: ret
        let mut c = Vec::new();
        let mut relocs = Vec::new();
        emit_rc_dec(&mut c, &mut relocs);
        // starts with `dec qword [rcx]` (48 FF 09) then a `jnz` (0F 85).
        assert_eq!(&c[0..3], &[0x48, 0xFF, 0x09]);
        assert_eq!(&c[3..5], &[0x0F, 0x85]);
        // one relocation to the free symbol, and the code ends in `ret`.
        assert_eq!(relocs.len(), 1);
        assert_eq!(relocs[0].1, RC_FREE_SYMBOL);
        assert_eq!(*c.last().unwrap(), 0xC3, "rc_dec must end in ret");
        // the `jnz` skips exactly the 5-byte `call rel32`, landing on `ret`.
        let jnz_rel = i32::from_le_bytes([c[5], c[6], c[7], c[8]]);
        assert_eq!(jnz_rel, 5, "jnz must skip the 5-byte call to reach `ret`");
    }
}
