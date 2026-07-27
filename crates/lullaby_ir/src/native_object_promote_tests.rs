//! Unit tests for the **arena stage-4b promotion gate** (`native_object_promote.rs`):
//! the capture-class admission ([`promoted_capture_words`]) and the summed-word survivor
//! size ([`promoted_survivor_words`]) that the admission side
//! ([`returns_promotable_closure`]) and the emit side
//! (`NativeCtx::promoted_survivor_bytes`) share.
//!
//! These assert the gate DIRECTLY against synthetic [`ClosureLayout`]s, which is the
//! only way to test it with teeth today: the gate's whole purpose is to refuse capture
//! classes the current [`native_closure_scalar`] cannot even produce, so no `.lby`
//! source can reach it through the normal pipeline. A source-level test would be green
//! for the wrong reason — it would prove the frontend refuses heap captures, not that
//! the promotion gate does. Building the layout by hand is exactly the widening a
//! future contributor would perform inside `native_closure_scalar`, simulated.
//!
//! The end-to-end "promotion fires and reclaims" proofs live in
//! `crates/lullaby_cli/tests/cli/returned_closure.rs`.

use super::*;

/// A layout returning `i64` with no parameters and the given capture classes.
fn layout_with(captures: &[NativeType]) -> ClosureLayout {
    ClosureLayout {
        captures: captures
            .iter()
            .enumerate()
            .map(|(i, c)| (format!("c{i}"), c.clone()))
            .collect(),
        params: Vec::new(),
        ret: NativeType::I64,
    }
}

/// The three immediate 8-byte scalars are the admitted set, each exactly one word.
#[test]
fn promoted_capture_words_admits_only_immediate_scalars() {
    for class in [NativeType::I64, NativeType::F64, NativeType::F32] {
        assert_eq!(
            promoted_capture_words(&class),
            Some(1),
            "{class:?} is an immediate 8-byte scalar and must be promotable as one word"
        );
    }
}

/// Every class that is NOT an immediate 8-byte scalar is refused. The pointer-carrying
/// ones are the dangerous half: one word wide, so a count-based gate would admit them,
/// but the word is an address the flat memmove cannot relocate — promoting such a block
/// and then setting `heap_next = markF + size` frees the pointee the copied word still
/// addresses.
#[test]
fn promoted_capture_words_refuses_pointer_and_multiword_classes() {
    let refused = [
        // Pointer-carrying, single word — the use-after-free shapes.
        NativeType::String,
        NativeType::HeapStruct {
            name: "S".to_string(),
            fields: vec![("a".to_string(), NativeType::I64)],
        },
        NativeType::List {
            elem: Box::new(NativeType::I64),
        },
        NativeType::Map {
            key: Box::new(NativeType::I64),
            value: Box::new(NativeType::I64),
        },
        NativeType::FatArray {
            elem: Box::new(NativeType::I64),
        },
        // Not a single 8-byte value word.
        NativeType::Void,
        NativeType::Narrow {
            bytes: 4,
            signed: true,
        },
        NativeType::Struct {
            name: "S".to_string(),
            fields: vec![
                ("a".to_string(), NativeType::I64),
                ("b".to_string(), NativeType::I64),
            ],
        },
        NativeType::Array {
            elem: Box::new(NativeType::I64),
            len: 4,
        },
        NativeType::Enum {
            name: "E".to_string(),
            variants: Vec::new(),
            payload_words: 0,
        },
    ];
    for class in refused {
        assert_eq!(
            promoted_capture_words(&class),
            None,
            "{class:?} must be refused by the promotion gate — the stage-4b reset \
             relocates the block with a flat word memmove and then reclaims past it, \
             which is unsound for a pointer word or a non-8-byte capture"
        );
    }
}

/// The survivor size is `1` (code pointer) plus the SUMMED capture words, not a capture
/// count, so the gate and `promoted_survivor_bytes` can never disagree about how many
/// bytes the reset moves.
#[test]
fn promoted_survivor_words_sums_code_pointer_plus_captures() {
    assert_eq!(promoted_survivor_words(&layout_with(&[])), Some(1));
    assert_eq!(
        promoted_survivor_words(&layout_with(&[NativeType::I64])),
        Some(2)
    );
    assert_eq!(
        promoted_survivor_words(&layout_with(&[
            NativeType::I64,
            NativeType::F64,
            NativeType::F32
        ])),
        Some(4)
    );
}

/// The `MAX_PROMOTED_CLOSURE_WORDS` cap is expressed against the summed word count and
/// is inclusive: `1 + 7` captures is exactly the cap and promotes; one more is refused
/// (the factory stays off-arena, still sound). This pins the shipped boundary — the
/// pre-existing `captures.len() < MAX` test admitted precisely the same set.
#[test]
fn promoted_survivor_words_enforces_the_cap_on_summed_words() {
    let at_cap = vec![NativeType::I64; MAX_PROMOTED_CLOSURE_WORDS - 1];
    assert_eq!(
        promoted_survivor_words(&layout_with(&at_cap)),
        Some(MAX_PROMOTED_CLOSURE_WORDS),
        "a block of exactly MAX_PROMOTED_CLOSURE_WORDS words must still promote"
    );
    let over_cap = vec![NativeType::I64; MAX_PROMOTED_CLOSURE_WORDS];
    assert_eq!(
        promoted_survivor_words(&layout_with(&over_cap)),
        None,
        "a block one word past the cap must be refused"
    );
}

/// **The regression this module exists for.** A layout that is small (well under the
/// cap) and has few captures — so every count-based check passes — but carries ONE
/// pointer word must not be promotable. Before the explicit gate, widening
/// `native_closure_scalar` to classify `string` would have made exactly this layout
/// reachable and silently promoted, producing a use-after-free with no diagnostic.
#[test]
fn a_single_pointer_capture_defeats_promotion() {
    let layout = layout_with(&[NativeType::I64, NativeType::String, NativeType::F64]);
    assert!(
        layout.captures.len() < MAX_PROMOTED_CLOSURE_WORDS,
        "the fixture must be small enough that a count-based gate would admit it, \
         otherwise this test proves nothing"
    );
    assert_eq!(
        promoted_survivor_words(&layout),
        None,
        "a capture block containing a pointer word must never be promoted: the flat \
         memmove copies the pointer without relocating its pointee, and the following \
         `heap_next = markF + size` reclaims that pointee"
    );
}
