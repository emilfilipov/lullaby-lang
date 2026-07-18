//! Structural tests for the direct ELF64 (`ET_EXEC`) executable writer. They
//! emit a native program for the Linux target through the full frontend, then
//! parse the produced ELF bytes back and assert the `Elf64_Ehdr`, the program
//! header table, the segment permissions/alignment, and the resolved entry
//! point are all well-formed and mutually consistent — mirroring the parse-back
//! structural tests in `pe_image_tests.rs`. They are the Windows-host guard;
//! actual load-and-run parity is proven by the CLI test
//! `native_freestanding_direct_elf_runs_under_docker` (no linker involved).
//!
//! The eligibility/skip boundary and the writer's refusal to emit a
//! partially-linked image are additionally proven at the model level, against
//! hand-built [`ObjectModel`]s with an undefined external symbol, a non-x86-64
//! machine, and a non-REL32 relocation — the three ways a program leaves the
//! directly-emittable subset.

use super::*;
use crate::native_contract::native_target_for_triple;
use crate::object_model::{
    ObjectModel, ObjectRelocation, ObjectRelocationKind, ObjectSection, ObjectSectionKind,
    ObjectSymbol, ObjectSymbolKind,
};
use crate::{lower, lower_to_bytecode};
use lullaby_lexer::lex;
use lullaby_parser::parse;
use lullaby_semantics::validate_executable;

fn module_for(source: &str) -> BytecodeModule {
    let tokens = lex(source).expect("lex");
    let program = parse(&tokens).expect("parse");
    let checked = validate_executable(&program).expect("semantic");
    let ir = lower(&checked).expect("lower");
    lower_to_bytecode(&ir)
}

/// Emit the Linux x86-64 direct ELF image for `source`.
fn elf_for(source: &str) -> Vec<u8> {
    let target = native_target_for_triple("x86_64-unknown-linux-gnu").expect("linux target");
    crate::emit_native_program_for_target(&module_for(source), &target, None, false)
        .expect("native program emits")
        .elf_image
        .expect("freestanding program emits a direct ELF")
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn u32_at(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

fn u64_at(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

/// The parsed program header table: one `(p_type, p_flags, p_offset, p_vaddr,
/// p_filesz, p_memsz, p_align)` tuple per `PT_LOAD` segment.
fn segments(bytes: &[u8]) -> Vec<(u32, u32, u64, u64, u64, u64, u64)> {
    let phoff = u64_at(bytes, 32) as usize;
    let phentsize = u16_at(bytes, 54) as usize;
    let phnum = u16_at(bytes, 56) as usize;
    assert_eq!(phentsize, 56, "Elf64_Phdr size");
    (0..phnum)
        .map(|i| {
            let ph = phoff + i * phentsize;
            (
                u32_at(bytes, ph),
                u32_at(bytes, ph + 4),
                u64_at(bytes, ph + 8),
                u64_at(bytes, ph + 16),
                u64_at(bytes, ph + 32),
                u64_at(bytes, ph + 40),
                u64_at(bytes, ph + 48),
            )
        })
        .collect()
}

/// The file offset of a virtual address, via the loaded segment that contains
/// it (`p_vaddr == LOAD_BASE + p_offset`, so `offset = vaddr - p_vaddr + p_offset`).
fn vaddr_to_offset(bytes: &[u8], vaddr: u64) -> usize {
    for (_, _, p_offset, p_vaddr, p_filesz, _, _) in segments(bytes) {
        if vaddr >= p_vaddr && vaddr < p_vaddr + p_filesz {
            return (vaddr - p_vaddr + p_offset) as usize;
        }
    }
    panic!("vaddr {vaddr:#x} is not inside any loaded file segment");
}

#[test]
fn scalar_program_emits_valid_elf64() {
    let elf = elf_for("fn main -> i64\n    42\n");

    // Elf64_Ehdr identification.
    assert_eq!(&elf[0..4], &[0x7F, b'E', b'L', b'F'], "ELF magic");
    assert_eq!(elf[4], 2, "EI_CLASS = ELFCLASS64");
    assert_eq!(elf[5], 1, "EI_DATA = ELFDATA2LSB");
    assert_eq!(elf[6], 1, "EI_VERSION = EV_CURRENT");
    assert_eq!(u16_at(&elf, 16), 2, "e_type = ET_EXEC");
    assert_eq!(u16_at(&elf, 18), 62, "e_machine = EM_X86_64");
    assert_eq!(u32_at(&elf, 20), 1, "e_version = EV_CURRENT");
    assert_eq!(u64_at(&elf, 32), 64, "e_phoff (phdrs follow the ehdr)");
    assert_eq!(u16_at(&elf, 52), 64, "e_ehsize");
    assert_eq!(u16_at(&elf, 54), 56, "e_phentsize");

    // The entry point resolves inside a loaded segment and begins with the
    // freestanding entry stub prologue: `sub rsp, 32` then a `call rel32`.
    let entry = u64_at(&elf, 24);
    let entry_off = vaddr_to_offset(&elf, entry);
    assert_eq!(
        &elf[entry_off..entry_off + 4],
        &[0x48, 0x83, 0xEC, 0x20],
        "sub rsp, 32"
    );
    assert_eq!(elf[entry_off + 4], 0xE8, "call main (rel32)");
    // The Linux stub exits via the `exit` syscall: after `call main` it moves the
    // result to edi (`mov edi, eax` = 89 C7) then `mov eax, 60` (B8 3C ..).
    assert_eq!(
        &elf[entry_off + 9..entry_off + 11],
        &[0x89, 0xC7],
        "mov edi, eax"
    );
    assert_eq!(elf[entry_off + 11], 0xB8, "mov eax, imm32 (SYS_exit)");
    assert_eq!(u32_at(&elf, entry_off + 12), 60, "SYS_exit = 60");
    assert_eq!(
        &elf[entry_off + 16..entry_off + 18],
        &[0x0F, 0x05],
        "syscall"
    );
}

#[test]
fn scalar_program_is_a_single_rx_segment() {
    // No strings, no heap: exactly one PT_LOAD, read+execute, mapping the headers
    // at file offset 0 / virtual address LOAD_BASE.
    let elf = elf_for("fn main -> i64\n    let x i64 = 3\n    x + 4\n");
    let segs = segments(&elf);
    assert_eq!(segs.len(), 1, "one PT_LOAD segment");
    let (p_type, p_flags, p_offset, p_vaddr, p_filesz, p_memsz, p_align) = segs[0];
    assert_eq!(p_type, 1, "PT_LOAD");
    assert_eq!(p_flags, 0x4 | 0x1, "PF_R | PF_X");
    assert_eq!(p_offset, 0, "maps the headers at file offset 0");
    assert_eq!(p_vaddr, 0x40_0000, "LOAD_BASE");
    assert_eq!(p_filesz, p_memsz, "no bss in this segment");
    assert_eq!(p_align, 0x1000, "page-aligned");
    // Every loaded segment satisfies the ELF congruence p_vaddr ≡ p_offset (mod align).
    assert_eq!(
        (p_vaddr % p_align),
        (p_offset % p_align),
        "vaddr/offset congruent"
    );
}

#[test]
fn entry_stub_call_main_resolves_inside_text() {
    // The `call main` displacement immediately after the entry stub must resolve,
    // with no linker, to a virtual address inside the first (text) segment — i.e.
    // the relocation was fully internally resolved, not left zero.
    let elf = elf_for("fn helper a i64 -> i64\n    a * 2\n\nfn main -> i64\n    helper(21)\n");
    let entry = u64_at(&elf, 24);
    let entry_off = vaddr_to_offset(&elf, entry);
    // `call rel32` operand is the 4 bytes at entry_off+5; the next instruction is
    // at entry+9, so target = entry + 9 + disp.
    let disp = i32::from_le_bytes(elf[entry_off + 5..entry_off + 9].try_into().unwrap());
    let call_target = (entry as i64 + 9 + i64::from(disp)) as u64;
    let (_, _, _, p_vaddr, p_filesz, _, _) = segments(&elf)[0];
    assert!(
        call_target >= p_vaddr && call_target < p_vaddr + p_filesz,
        "call main target {call_target:#x} resolves inside the text segment [{p_vaddr:#x}, {:#x})",
        p_vaddr + p_filesz
    );
    // A zero displacement would mean the relocation was never resolved.
    assert_ne!(disp, 0, "call main displacement was resolved");
}

#[test]
fn heap_program_emits_rodata_and_bss_segments() {
    // `len("hello")` interns a `.rodata` string constant and uses the `.bss` bump
    // heap, so the image carries three PT_LOAD segments with distinct permissions.
    let elf = elf_for("fn main -> i64\n    len(\"hello\")\n");
    let segs = segments(&elf);
    assert_eq!(segs.len(), 3, "text + rodata + bss segments");

    assert_eq!(segs[0].1, 0x4 | 0x1, "text = PF_R | PF_X");
    assert_eq!(segs[1].1, 0x4, "rodata = PF_R");
    assert_eq!(segs[2].1, 0x4 | 0x2, "bss = PF_R | PF_W");

    // rodata carries the NUL-terminated literal.
    let (_, _, ro_off, _, ro_filesz, _, _) = segs[1];
    let ro = &elf[ro_off as usize..(ro_off + ro_filesz) as usize];
    assert!(
        ro.windows(6).any(|w| w == b"hello\0"),
        "string constant interned in the rodata segment"
    );

    // bss reserves address space (>= the 1 MiB heap) but no file bytes, and the
    // kernel zero-fills memsz-filesz.
    let (_, _, _, _, bss_filesz, bss_memsz, _) = segs[2];
    assert_eq!(bss_filesz, 0, "bss has no file bytes");
    assert!(bss_memsz >= 1024 * 1024, "bss reserves the heap region");

    // Segments are sorted by ascending virtual address and no two share a page.
    assert!(
        segs[0].3 < segs[1].3 && segs[1].3 < segs[2].3,
        "vaddrs ascending"
    );
    for (_, _, p_offset, p_vaddr, _, _, p_align) in &segs {
        assert_eq!(
            p_vaddr % p_align,
            p_offset % p_align,
            "each segment congruent"
        );
    }
}

#[test]
fn direct_elf_emission_is_deterministic() {
    let source = "fn fib n i64 -> i64\n    if n < 2\n        return n\n    return fib(n - 1) + fib(n - 2)\n\nfn main -> i64\n    fib(6)\n";
    assert_eq!(
        elf_for(source),
        elf_for(source),
        "byte-for-byte deterministic"
    );
}

#[test]
fn library_object_has_no_direct_elf() {
    // An export-only program (no `main`) has no entry point, so no direct ELF.
    let tokens = lex("export fn twice x i64 -> i64\n    x * 2\n").expect("lex");
    let program = parse(&tokens).expect("parse");
    let checked = lullaby_semantics::validate(&program).expect("semantic");
    let ir = lower(&checked).expect("lower");
    let module = lower_to_bytecode(&ir);
    let target = native_target_for_triple("x86_64-unknown-linux-gnu").expect("linux target");
    let emitted = crate::emit_native_program_for_target(&module, &target, None, false)
        .expect("library emits");
    assert!(
        emitted.elf_image.is_none(),
        "a library object (no main) has no direct ELF"
    );
}

// -- Model-level skip-boundary tests -----------------------------------------
// These prove the writer refuses (returns None → caller falls back to the
// object + linker path) rather than emitting a malformed/partially-linked image
// when a program leaves the directly-emittable freestanding subset.

/// A minimal runnable text-only model: an entry symbol at `.text` offset 0 and a
/// single `.text` section. `extra_symbol`/`extra_reloc` inject a boundary case.
fn text_only_model(
    machine: ObjectMachine,
    symbols: Vec<ObjectSymbol>,
    relocations: Vec<ObjectRelocation>,
) -> ObjectModel {
    ObjectModel {
        sections: vec![ObjectSection {
            kind: ObjectSectionKind::Text,
            // 16 bytes of `nop` padding — enough to hold any reloc field we test.
            data: vec![0x90; 16],
            size: 16,
            relocations,
        }],
        symbols,
        entry_symbol: Some("_start".to_string()),
        machine,
    }
}

#[test]
fn refuses_undefined_external_symbol() {
    // A relocation to an undefined external (an `extern fn` bound by the linker)
    // cannot be placed: the writer must refuse, not emit a bogus displacement.
    let symbols = vec![
        ObjectSymbol {
            name: "_start".to_string(),
            section: Some(0),
            value: 0,
            kind: ObjectSymbolKind::Function,
        },
        ObjectSymbol {
            name: "external_c_fn".to_string(),
            section: None, // undefined external
            value: 0,
            kind: ObjectSymbolKind::Function,
        },
    ];
    let relocations = vec![ObjectRelocation {
        offset: 4,
        symbol: 1,
        kind: ObjectRelocationKind::Branch,
    }];
    let model = text_only_model(ObjectMachine::X86_64, symbols, relocations);
    assert!(
        super::write_elf_executable_from_model(&model).is_none(),
        "an undefined external must force a fallback, not a malformed ELF"
    );
}

#[test]
fn refuses_non_x86_64_machine() {
    let symbols = vec![ObjectSymbol {
        name: "_start".to_string(),
        section: Some(0),
        value: 0,
        kind: ObjectSymbolKind::Function,
    }];
    let model = text_only_model(ObjectMachine::Aarch64, symbols, Vec::new());
    assert!(
        super::write_elf_executable_from_model(&model).is_none(),
        "the AArch64 model has its own object builder and is never a direct-exec candidate"
    );
}

#[test]
fn refuses_non_rel32_relocation() {
    // A DWARF-only Absolute64 relocation is not part of the freestanding REL32
    // subset this writer resolves; refuse rather than mis-patch an 8-byte field.
    let symbols = vec![ObjectSymbol {
        name: "_start".to_string(),
        section: Some(0),
        value: 0,
        kind: ObjectSymbolKind::Function,
    }];
    let relocations = vec![ObjectRelocation {
        offset: 4,
        symbol: 0,
        kind: ObjectRelocationKind::Absolute64,
    }];
    let model = text_only_model(ObjectMachine::X86_64, symbols, relocations);
    assert!(
        super::write_elf_executable_from_model(&model).is_none(),
        "a non-REL32 relocation must force a fallback"
    );
}

#[test]
fn accepts_minimal_text_only_model() {
    // The positive control: a text-only model with a resolvable self-branch does
    // emit, so the refusals above are discriminating (not a writer that always
    // returns None).
    let symbols = vec![ObjectSymbol {
        name: "_start".to_string(),
        section: Some(0),
        value: 0,
        kind: ObjectSymbolKind::Function,
    }];
    let relocations = vec![ObjectRelocation {
        offset: 4,
        symbol: 0,
        kind: ObjectRelocationKind::Branch,
    }];
    let model = text_only_model(ObjectMachine::X86_64, symbols, relocations);
    let elf = super::write_elf_executable_from_model(&model).expect("minimal model emits");
    assert_eq!(&elf[0..4], &[0x7F, b'E', b'L', b'F'], "valid ELF magic");
    assert_eq!(u16_at(&elf, 16), 2, "ET_EXEC");
}
