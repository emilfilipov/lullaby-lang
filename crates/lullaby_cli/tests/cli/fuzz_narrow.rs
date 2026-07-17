//! Fuzzing for **packed narrow array elements** — walking an `array<i32>` /
//! `array<u8>` / `array<i16>` / … through raw pointers
//! (`native_object_types.rs`'s `NativeType::Narrow`, `native_object_rawptr.rs`'s
//! width-agreement law). A submodule of `fuzz.rs`, reusing its shared `Rng`,
//! `ScratchDir`, and `fuzz_native_exit` harness via `use super::*`.
//!
//! # The bug class this exists to catch
//!
//! A narrow-element array is stored PACKED at the element's C width, so
//! `ptr_offset(p, 1)` must stride `size_of(T)` — 4 for `i32`, 1 for `u8`. If the
//! stride and the element layout ever disagree, a walk DESYNCHRONIZES: it reads
//! and writes the wrong elements and runs off the end of the buffer. That is
//! memory corruption, and it is silent — the program still exits, just with the
//! wrong number. This fuzzer's whole purpose is to make a wrong stride loud.
//!
//! # Two oracles, both real
//!
//! Narrow walks have **full four-tier parity** (the interpreters' `addr_of` region
//! stride is `Value::layout_size` of the element), so both oracles apply:
//!
//! * a **differential** oracle — native's exit code must equal the interpreters',
//!   and the three interpreters must agree with each other; and
//! * a **value** oracle — the generator computes each program's exact result
//!   independently in Rust, so a bug that corrupts *every* tier identically still
//!   fails. A pure differential would miss a stride error replicated across tiers.
//!
//! # Why the generator interleaves pointer walks with index reads
//!
//! Reading `a[i]` and reading `ptr_read(ptr_offset(addr_of(a[0]), i))` go down
//! **different code paths** — the element load path and the raw-pointer path — and
//! the entire correctness claim is that they name the same storage. Generating
//! both over the same array, and summing them together, is what turns a stride
//! disagreement into a wrong total rather than two independently-plausible answers.
//! The same reason drives the write-through arm: a `ptr_write` walk followed by
//! `a[i]` reads crosses the two paths in the other direction.

use super::*;

/// A narrow element type the backend packs, with the facts the generator needs to
/// predict values exactly: its Lullaby type name, its constructor, its byte width
/// (the stride the size law must report), and its value range.
#[derive(Clone, Copy)]
struct NarrowTy {
    name: &'static str,
    ctor: &'static str,
    bytes: i64,
    lo: i64,
    hi: i64,
}

/// The packed narrow element types. This list mirrors
/// `native_object_types.rs::narrow_array_element` exactly — if that set changes,
/// this must too, or the fuzzer stops covering a delivered width.
const NARROW_TYPES: &[NarrowTy] = &[
    NarrowTy {
        name: "i32",
        ctor: "to_i32",
        bytes: 4,
        lo: -1000,
        hi: 1000,
    },
    NarrowTy {
        name: "u32",
        ctor: "to_u32",
        bytes: 4,
        lo: 0,
        hi: 2000,
    },
    NarrowTy {
        name: "i16",
        ctor: "to_i16",
        bytes: 2,
        lo: -300,
        hi: 300,
    },
    NarrowTy {
        name: "u16",
        ctor: "to_u16",
        bytes: 2,
        lo: 0,
        hi: 600,
    },
    NarrowTy {
        name: "i8",
        ctor: "to_i8",
        bytes: 1,
        lo: -50,
        hi: 50,
    },
    NarrowTy {
        name: "u8",
        ctor: "to_u8",
        bytes: 1,
        lo: 0,
        hi: 100,
    },
];

/// One generated narrow-array program plus its independently-computed value.
struct NarrowProgram {
    source: String,
    expected: i64,
}

/// Generate a program that builds a narrow-element array and reads it back
/// through a mix of: a `ptr_offset` walk from `addr_of(a[0])`, direct `a[i]`
/// index reads, size-law terms, a negative offset stepping back, whole-array
/// decay, and (sometimes) a write-through walk read back by index.
///
/// `expected` is computed in Rust from the same element values, so it does not
/// depend on any Lullaby tier being correct.
fn gen_narrow_array_program(seed: u64) -> NarrowProgram {
    let mut rng = Rng(seed | 1);
    let ty = NARROW_TYPES[rng.below(NARROW_TYPES.len() as u64) as usize];
    let len = rng.range(2, 6);
    let values: Vec<i64> = (0..len).map(|_| rng.range(ty.lo, ty.hi)).collect();

    let mut src = String::from("fn main -> i64\n");
    let elements: Vec<String> = values.iter().map(|v| format!("{}({v})", ty.ctor)).collect();
    src.push_str(&format!(
        "    let a array<{}> = [{}]\n",
        ty.name,
        elements.join(", ")
    ));
    src.push_str("    let out i64 = 0\n");
    src.push_str("    unsafe\n");
    src.push_str(&format!("        let p ptr<{}> = addr_of(a[0])\n", ty.name));

    let mut expected: i64 = 0;

    // The SIZE LAW: `ptr_to_int(ptr_offset(p, 1)) - ptr_to_int(p) == size_of(T)`.
    // This is the stride, observed directly as a number.
    src.push_str("        out = out + (ptr_to_int(ptr_offset(p, 1)) - ptr_to_int(p))\n");
    expected += ty.bytes;

    // A full WALK through the pointer, element by element.
    src.push_str("        let i i64 = 0\n");
    src.push_str(&format!("        while i < {len}\n"));
    src.push_str("            out = out + to_i64(ptr_read(ptr_offset(p, i)))\n");
    src.push_str("            i = i + 1\n");
    expected += values.iter().sum::<i64>();

    // The SAME elements read by CONSTANT INDEX. Crossing the element-load path
    // against the raw-pointer path over one array is what makes a stride
    // disagreement visible.
    //
    // A constant index folds into a STATIC byte offset at compile time, so this
    // arm exercises `resolve_place_steps_typed`'s constant path only.
    for (index, value) in values.iter().enumerate() {
        src.push_str(&format!("        out = out + to_i64(a[{index}])\n"));
        expected += value;
    }

    // The same elements read by a RUNTIME INDEX. This is a DIFFERENT code path
    // from every arm above and must be generated explicitly:
    //
    // * a constant index folds into a static offset (no scaling emitted at all);
    // * `ptr_offset(p, i)` scales in `native_object_rawptr.rs`
    //   (`emit_scaled_add_rcx_rax`, an x86 SIB scale);
    // * `a[i]` for a runtime `i` scales in `native_object_place.rs`
    //   (`emit_scale_rax_by_stride`, a `shl`).
    //
    // Those are three separate stride implementations. An earlier version of this
    // generator emitted only the first two, and a deliberately injected 4-byte
    // stride bug in `emit_scale_rax_by_stride` passed the fuzzer untouched — the
    // teeth proof caught the gap. Do not remove this arm.
    src.push_str("        let j i64 = 0\n");
    src.push_str(&format!("        while j < {len}\n"));
    src.push_str("            out = out + to_i64(a[j])\n");
    src.push_str("            j = j + 1\n");
    expected += values.iter().sum::<i64>();

    // A NEGATIVE offset: address the last element, then step back to element 0.
    let last = len - 1;
    src.push_str(&format!(
        "        let plast ptr<{}> = addr_of(a[{last}])\n",
        ty.name
    ));
    src.push_str(&format!(
        "        out = out + to_i64(ptr_read(ptr_offset(plast, 0 - {last})))\n"
    ));
    expected += values[0];

    // WHOLE-ARRAY DECAY: `addr_of(a)` is a `ptr<T>` at element 0, so it must both
    // read element 0 and compare equal to `addr_of(a[0])`.
    if rng.chance(2) {
        src.push_str(&format!("        let d ptr<{}> = addr_of(a)\n", ty.name));
        src.push_str("        out = out + to_i64(ptr_read(d))\n");
        expected += values[0];
        src.push_str("        if ptr_to_int(d) == ptr_to_int(p)\n");
        src.push_str("            out = out + 1\n");
        expected += 1;
    }

    // WRITE-THROUGH: overwrite one element via the pointer walk, then read it back
    // BY INDEX. A stride error sends the write to the wrong element (or past the
    // end) while the index read still looks at the right one.
    if rng.chance(2) {
        let target = rng.range(0, len - 1);
        let fresh = rng.range(ty.lo, ty.hi);
        src.push_str(&format!(
            "        ptr_write(ptr_offset(p, {target}), {}({fresh}))\n",
            ty.ctor
        ));
        src.push_str(&format!("        out = out + to_i64(a[{target}])\n"));
        expected += fresh;
    }

    src.push_str("    out\n");
    NarrowProgram {
        source: src,
        expected,
    }
}

/// The generator must produce programs the interpreters actually run — if it
/// emitted something they reject, the differential oracle below would be vacuous
/// (every program skipping, nothing compared). Pinned separately so that failure
/// mode is loud rather than a silently-passing fuzz run.
#[test]
fn narrow_array_generator_produces_runnable_programs() {
    let base_seed = 0x51D3_9C40_2B7E_A16Fu64;
    for i in 0..12u64 {
        let seed = base_seed.wrapping_add(i.wrapping_mul(0xA076_1D64_78BD_642F));
        let program = gen_narrow_array_program(seed);
        let (ast, ir, bc) = run_interpreters(&program.source);
        assert!(
            ast == ir && ir == bc,
            "interpreter divergence on narrow-array seed {seed:#x}:\n{}",
            program.source
        );
        assert_eq!(
            ast,
            Outcome::Value(program.expected),
            "the generator's own value oracle disagrees with the interpreters on seed \
             {seed:#x}:\n{}",
            program.source
        );
    }
}

/// The differential + value oracle for packed narrow array walks: every generated
/// program must compile natively (these shapes are delivered, so a skip is a
/// REGRESSION, not an acceptable default-deny) and exit with exactly the value the
/// generator computed in Rust — which the three interpreters must also agree on.
///
/// `fuzz_native_exit` panics on an emit failure, so a change that made these
/// programs skip again fails here rather than passing quietly.
#[test]
fn fuzz_narrow_array_walks_match_the_interpreters_and_the_value_oracle() {
    if !cfg!(windows) {
        eprintln!("not a Windows host; skipping the narrow-array native differential fuzz");
        return;
    }
    const PROGRAMS: u64 = 80;
    let base_seed = 0x7C1E_B85A_36F4_D209u64;
    let dir = ScratchDir::new("narrow");
    let mut executed = 0u64;

    for i in 0..PROGRAMS {
        let seed = base_seed.wrapping_add(i.wrapping_mul(0xA076_1D64_78BD_642F));
        let program = gen_narrow_array_program(seed);

        let (ast, ir, bc) = run_interpreters(&program.source);
        assert!(
            ast == ir && ir == bc,
            "interpreter divergence on narrow-array fuzz #{i} (seed {seed:#x}):\n{}",
            program.source
        );
        assert_eq!(
            ast,
            Outcome::Value(program.expected),
            "value-oracle divergence on narrow-array fuzz #{i} (seed {seed:#x}):\n{}",
            program.source
        );

        let Some(exit) = fuzz_native_exit(&program.source, &dir, &format!("narrow_{i}")) else {
            continue;
        };
        assert_eq!(
            exit, program.expected as i32,
            "native/interpreter divergence on narrow-array fuzz #{i} (seed {seed:#x}):\n{}\n\
             expected={}, native exit={exit}",
            program.source, program.expected
        );
        executed += 1;
    }

    // A fuzzer that ran nothing is worse than no fuzzer: report what executed, so a
    // harness that silently stopped producing exes cannot pass as green.
    assert!(
        executed > 0,
        "the narrow-array fuzzer executed no native programs"
    );
    eprintln!("narrow-array fuzz: {executed}/{PROGRAMS} programs executed natively");
}
