//! Direct ELF64 executable writer for the **freestanding** native case — the
//! Linux analog of the `pe_image` direct-PE writer.
//!
//! For a freestanding program — one with a `main` and no C-runtime import —
//! Lullaby lays a complete, runnable Linux `ET_EXEC` image around the
//! already-generated `.text` machine code itself, skipping the external linker
//! (`ld.lld`/`rust-lld`) entirely, exactly as the PE path skips it on Windows.
//!
//! # `ET_EXEC` (fixed load base), not static-PIE
//!
//! The emitted image is a fixed-base `ET_EXEC`: a non-relocatable executable
//! that the kernel maps at its declared `p_vaddr`s. This is the exact analog of
//! the direct-PE writer's `RELOCS_STRIPPED`, fixed-`ImageBase` PE — and it is
//! chosen for the same reason. Every intra-image reference the backend emits is
//! **PC-relative** already (`call rel32` between functions, `lea`/`mov`
//! RIP-relative to `.rodata`/`.bss`), so once the segment virtual addresses are
//! fixed here at emit time, every relocation resolves to a final 32-bit
//! displacement and **no load-time fixup remains**. A static-PIE (`ET_DYN`)
//! image would instead need a `PT_DYNAMIC` with `R_X86_64_RELATIVE` self-
//! relocations and a self-relocating entry — machinery that buys nothing when
//! the code is already position-independent and there is no dynamic symbol to
//! resolve. `ET_EXEC` keeps the writer, like the PE writer, a pure byte layout
//! with no dynamic-linking surface: no `PT_INTERP`, no `PT_DYNAMIC`, no dynamic
//! relocation table.
//!
//! # Segment / entry layout
//!
//! One `PT_LOAD` per distinct permission set, page-aligned, laid out so each
//! segment's file offset and virtual address are congruent modulo the page size
//! (`p_vaddr == LOAD_BASE + p_offset`), which is what ELF requires:
//!
//! - **`R+X`** — the `Elf64_Ehdr`, the program-header table, and `.text`
//!   (entry stub + compiled functions + heap/string helpers). Mapping the
//!   headers read-execute is harmless and makes `p_offset == 0` for this
//!   segment, so the program headers are reachable at `LOAD_BASE + e_phoff`.
//!   `e_entry` points at the entry stub, which the object model places at
//!   `.text` offset 0.
//! - **`R`** — `.rodata` (the NUL-terminated string constants), emitted only
//!   when the program interns any string.
//! - **`R+W`** — `.bss` (the bump-heap cells), emitted only when the program
//!   uses the heap. It occupies address space but no file bytes; the kernel
//!   zero-fills it, which is exactly what the bump allocator's zeroed cursor
//!   cells require.
//!
//! A purely-scalar freestanding program (no strings, no heap) needs only the
//! single `R+X` segment — the minimal one-`PT_LOAD` static executable.
//!
//! # Eligibility / skip boundary (mirrors direct-PE)
//!
//! This writer resolves **only** the freestanding-linkable symbol set the
//! backend produces: the compiled functions, the heap/string runtime helpers,
//! the `.rodata` string constants, and the four `.bss` heap cells. There is no
//! external symbol to resolve (the freestanding subset imports no libc and, on
//! Linux, exits through a raw `exit` syscall rather than an imported function —
//! so unlike the PE path there is not even an `ExitProcess` import). If any
//! relocation names a symbol this writer cannot place — an undefined external,
//! anything requiring the C runtime or dynamic linking — it returns `None` and
//! the caller falls back to the relocatable-object + external-linker path,
//! exactly as the PE writer does. It never emits a partially-linked or malformed
//! executable.
//!
//! This module is a descendant of `native_object`, so it reuses the private
//! `.text` layout types and the neutral [`ObjectModel`] builder
//! ([`build_object_model`]) via `use super::*`. It consumes the *same* Linux
//! object model the relocatable-ELF path builds, so the machine code, entry
//! stub, and symbol set are identical between the direct executable and the
//! object — only the container differs.

use super::*;

use crate::object_model::{
    ObjectMachine, ObjectModel, ObjectRelocationKind, ObjectSectionKind, ObjectSymbolKind,
};

/// Preferred load address of the emitted executable. The traditional non-PIE
/// x86-64 ELF text base; page-aligned, and clear of the NULL page so a null
/// pointer never aliases mapped memory. With every segment placed at
/// `LOAD_BASE + p_offset`, the ELF `p_vaddr ≡ p_offset (mod p_align)` congruence
/// holds for free.
const LOAD_BASE: u64 = 0x40_0000;

/// Page size used for both file and memory segment alignment (`p_align`).
const PAGE: u64 = 0x1000;

/// Size in bytes of an `Elf64_Ehdr`.
const EHDR_SIZE: u64 = 64;
/// Size in bytes of an `Elf64_Phdr`.
const PHDR_SIZE: u64 = 56;

/// `ET_EXEC` — an executable (fixed-address) object.
const ET_EXEC: u16 = 2;
/// `EM_X86_64`.
const EM_X86_64: u16 = 62;
/// `PT_LOAD` — a loadable segment.
const PT_LOAD: u32 = 1;
/// `PF_X` — executable segment.
const PF_X: u32 = 0x1;
/// `PF_W` — writable segment.
const PF_W: u32 = 0x2;
/// `PF_R` — readable segment.
const PF_R: u32 = 0x4;

fn align_up(value: u64, align: u64) -> u64 {
    value.div_ceil(align) * align
}

fn push_u16(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

/// One planned `PT_LOAD` segment.
struct ElfSegment {
    flags: u32,
    file_offset: u64,
    vaddr: u64,
    file_size: u64,
    mem_size: u64,
}

/// Write a runnable `ET_EXEC` x86-64 ELF executable for a freestanding native
/// program, given its lowered functions and interned string constants. Returns
/// `None` (so the caller falls back to the `ld.lld` object path) if the program
/// is not directly emittable — any relocation naming a symbol this writer cannot
/// place, a non-x86-64 model, a non-REL32 relocation kind, or no entry point.
///
/// The caller guarantees the program is freestanding-eligible (a `main` is
/// present and no `extern fn` C import is required); this writer reuses the
/// Linux [`ObjectModel`] the relocatable-ELF path builds — same `.text`, same
/// entry stub (`_start`, which exits through the `exit` syscall), same symbol
/// set — and lays the executable image around it.
pub(crate) fn write_elf_executable(
    functions: &[LoweredNativeFunction],
    strings: &StringPool,
    entry_stub: EntryStub,
) -> Option<Vec<u8>> {
    let model = build_object_model(functions, strings, entry_stub, PlatformAbi::Linux);
    write_elf_executable_from_model(&model)
}

/// Lay a fixed-base `ET_EXEC` image around a Linux [`ObjectModel`]. Split from
/// [`write_elf_executable`] so the layout/relocation logic can be unit-tested
/// against a hand-built model without going through the whole backend.
fn write_elf_executable_from_model(model: &ObjectModel) -> Option<Vec<u8>> {
    // Only the shared x86-64 machine code has a REL32 relocation model this
    // writer resolves; the AArch64 ELF path has its own object builder and is
    // never a direct-executable candidate here.
    if !matches!(model.machine, ObjectMachine::X86_64) {
        return None;
    }
    // A runnable image needs an entry point; a library object (no `main`) is not
    // a direct-executable candidate and keeps the object path.
    let entry_name = model.entry_symbol.as_ref()?;

    // Locate the sections. `.text` is always index 0. `.rodata`/`.bss` follow
    // when the heap is used. A DWARF/`--debug` section is never a direct-exec
    // candidate (the debug build keeps the object + linker path), so refuse if
    // one is present rather than silently dropping it.
    let mut text_data: Vec<u8> = Vec::new();
    let mut rodata_data: &[u8] = &[];
    let mut bss_size: u64 = 0;
    let mut rodata_index: Option<usize> = None;
    let mut bss_index: Option<usize> = None;
    for (index, section) in model.sections.iter().enumerate() {
        match section.kind {
            ObjectSectionKind::Text => text_data = section.data.clone(),
            ObjectSectionKind::ReadOnlyData => {
                rodata_data = &section.data;
                rodata_index = Some(index);
            }
            ObjectSectionKind::Bss => {
                bss_size = section.size;
                bss_index = Some(index);
            }
            ObjectSectionKind::Debug(_) => return None,
        }
    }
    let text_len = text_data.len() as u64;

    // -- Decide which segments exist, then the header size follows -----------
    // `.rodata` gets a segment only when non-empty (an empty read-only segment
    // is pointless and no relocation can reference it — string symbols exist
    // only when strings do). `.bss` gets a segment whenever the heap is used.
    let has_rodata = !rodata_data.is_empty();
    let has_bss = bss_size > 0;
    let segment_count = 1 + u64::from(has_rodata) + u64::from(has_bss);
    let header_size = EHDR_SIZE + PHDR_SIZE * segment_count;

    // -- Assign file offsets and virtual addresses (p_vaddr = BASE + offset) -
    // The R+X segment leads with the headers, so `.text` starts at `header_size`
    // and the entry stub (at `.text` offset 0) sits at `LOAD_BASE + header_size`.
    let text_file_offset = header_size;
    let text_vaddr = LOAD_BASE + text_file_offset;
    let mut file_cursor = text_file_offset + text_len;

    let mut segments: Vec<ElfSegment> = vec![ElfSegment {
        flags: PF_R | PF_X,
        file_offset: 0,
        vaddr: LOAD_BASE,
        file_size: header_size + text_len,
        mem_size: header_size + text_len,
    }];

    let mut rodata_vaddr = 0u64;
    let mut rodata_file_offset = 0u64;
    if has_rodata {
        rodata_file_offset = align_up(file_cursor, PAGE);
        rodata_vaddr = LOAD_BASE + rodata_file_offset;
        let rodata_len = rodata_data.len() as u64;
        segments.push(ElfSegment {
            flags: PF_R,
            file_offset: rodata_file_offset,
            vaddr: rodata_vaddr,
            file_size: rodata_len,
            mem_size: rodata_len,
        });
        file_cursor = rodata_file_offset + rodata_len;
    }

    let mut bss_vaddr = 0u64;
    if has_bss {
        let bss_file_offset = align_up(file_cursor, PAGE);
        bss_vaddr = LOAD_BASE + bss_file_offset;
        segments.push(ElfSegment {
            flags: PF_R | PF_W,
            file_offset: bss_file_offset,
            vaddr: bss_vaddr,
            // `.bss` occupies no file bytes; the kernel zero-fills `mem_size`.
            file_size: 0,
            mem_size: bss_size,
        });
    }
    debug_assert_eq!(segments.len() as u64, segment_count);

    // The virtual address of a section (by its model index), used to resolve a
    // symbol's address. A section referenced but not laid out (impossible for
    // the freestanding subset, but guarded) makes the program non-emittable.
    let section_vaddr = |index: usize| -> Option<u64> {
        if index == 0 {
            Some(text_vaddr)
        } else if Some(index) == rodata_index && has_rodata {
            Some(rodata_vaddr)
        } else if Some(index) == bss_index && has_bss {
            Some(bss_vaddr)
        } else {
            None
        }
    };

    // -- Resolve every `.text` relocation to a final displacement ------------
    // Both freestanding REL32 kinds resolve to `S - (P + 4)`: the displacement
    // from the end of the 4-byte field to the target symbol. Any other kind, or
    // a symbol with no defining section (an undefined external), is unresolvable
    // here — refuse and let the caller fall back to the linker path.
    for reloc in &model.sections[0].relocations {
        if !matches!(
            reloc.kind,
            ObjectRelocationKind::Branch | ObjectRelocationKind::PcRel32
        ) {
            return None;
        }
        let symbol = &model.symbols[reloc.symbol];
        // A section symbol is DWARF-only and never referenced from `.text`; an
        // undefined external cannot be placed. Either means "not directly
        // emittable".
        if matches!(symbol.kind, ObjectSymbolKind::Section) {
            return None;
        }
        let target_vaddr = section_vaddr(symbol.section?)? + symbol.value;
        let field_vaddr = text_vaddr + reloc.offset;
        let displacement =
            i64::try_from(target_vaddr).ok()? - (i64::try_from(field_vaddr).ok()? + 4);
        let displacement = i32::try_from(displacement).ok()?;
        let field = usize::try_from(reloc.offset).ok()?;
        text_data
            .get_mut(field..field + 4)?
            .copy_from_slice(&displacement.to_le_bytes());
    }

    // The entry point is the entry symbol's virtual address (the stub at
    // `.text` offset 0). Resolved through the symbol table rather than assumed,
    // so it stays correct if the stub is ever placed at a non-zero offset.
    let entry_symbol = model.symbols.iter().find(|s| &s.name == entry_name)?;
    let entry_vaddr = section_vaddr(entry_symbol.section?)? + entry_symbol.value;

    // -- Emit the image ------------------------------------------------------
    let mut bytes: Vec<u8> = Vec::new();

    // Elf64_Ehdr.
    bytes.extend_from_slice(&[0x7F, b'E', b'L', b'F']); // EI_MAG
    bytes.push(2); // EI_CLASS = ELFCLASS64
    bytes.push(1); // EI_DATA  = ELFDATA2LSB
    bytes.push(1); // EI_VERSION = EV_CURRENT
    bytes.push(0); // EI_OSABI = ELFOSABI_SYSV
    bytes.push(0); // EI_ABIVERSION
    bytes.resize(16, 0); // EI_PAD (7 zero bytes)
    push_u16(&mut bytes, ET_EXEC); // e_type
    push_u16(&mut bytes, EM_X86_64); // e_machine
    push_u32(&mut bytes, 1); // e_version = EV_CURRENT
    push_u64(&mut bytes, entry_vaddr); // e_entry
    push_u64(&mut bytes, EHDR_SIZE); // e_phoff (phdrs follow the ehdr)
    push_u64(&mut bytes, 0); // e_shoff (no section headers: not needed to run)
    push_u32(&mut bytes, 0); // e_flags
    push_u16(&mut bytes, EHDR_SIZE as u16); // e_ehsize
    push_u16(&mut bytes, PHDR_SIZE as u16); // e_phentsize
    push_u16(&mut bytes, segment_count as u16); // e_phnum
    push_u16(&mut bytes, 0); // e_shentsize
    push_u16(&mut bytes, 0); // e_shnum
    push_u16(&mut bytes, 0); // e_shstrndx

    // Program header table: one Elf64_Phdr per segment.
    for segment in &segments {
        push_u32(&mut bytes, PT_LOAD); // p_type
        push_u32(&mut bytes, segment.flags); // p_flags
        push_u64(&mut bytes, segment.file_offset); // p_offset
        push_u64(&mut bytes, segment.vaddr); // p_vaddr
        push_u64(&mut bytes, segment.vaddr); // p_paddr (= vaddr)
        push_u64(&mut bytes, segment.file_size); // p_filesz
        push_u64(&mut bytes, segment.mem_size); // p_memsz
        push_u64(&mut bytes, PAGE); // p_align
    }
    debug_assert_eq!(bytes.len() as u64, header_size);

    // `.text` bytes (resolved) directly follow the header block.
    bytes.extend_from_slice(&text_data);

    // `.rodata`, page-aligned in the file to match its page-aligned vaddr.
    if has_rodata {
        bytes.resize(rodata_file_offset as usize, 0);
        bytes.extend_from_slice(rodata_data);
    }
    // `.bss` contributes no file bytes (the kernel zero-fills it).

    Some(bytes)
}

#[cfg(test)]
#[path = "elf_image_tests.rs"]
mod tests;
