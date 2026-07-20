//! The fixed x86-64 general-purpose register table shared by the semantic
//! validator and the native backend for inline-`asm` operand binding.
//!
//! Every register name a `in`/`out`/`clobber` clause may reference is validated
//! against this table. A register is described by its 4-bit architectural
//! encoding (`0..=15`, identical for a 64-bit register and all of its narrower
//! sub-registers, since they alias the same architectural register) and its
//! access width. The encoding doubles as the *parent* 64-bit register's code, so
//! the callee-saved analysis and the machine-code emitter both key off it.
//!
//! This lives in the parser crate — the lowest crate that both `lullaby_semantics`
//! and `lullaby_ir` depend on — so the register law has exactly one definition.

/// The access width of a named register.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegWidth {
    W8,
    W16,
    W32,
    W64,
}

impl RegWidth {
    /// The width in bits, for diagnostics.
    pub fn bits(self) -> u16 {
        match self {
            RegWidth::W8 => 8,
            RegWidth::W16 => 16,
            RegWidth::W32 => 32,
            RegWidth::W64 => 64,
        }
    }
}

/// A validated architectural register: its encoding and access width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegisterInfo {
    /// The register's 4-bit encoding (`0..=15`). Identical for a 64-bit register
    /// and each of its narrower views, because they name the same architectural
    /// register (`rax`/`eax`/`ax`/`al` all encode `0`).
    pub code: u8,
    /// The access width the spelled name selects.
    pub width: RegWidth,
}

impl RegisterInfo {
    /// Whether the register is the stack or base pointer (`rsp`/`rbp` and their
    /// sub-registers). Binding to or clobbering these would corrupt the frame the
    /// whole function is addressed through, so operand validation rejects them.
    pub fn is_frame_register(self) -> bool {
        self.code == 4 || self.code == 5
    }

    /// Whether the register's parent 64-bit register is callee-saved under
    /// *either* the Win64 or the System V AMD64 ABI. The native marshaller
    /// preserves (saves/restores around the asm) every callee-saved register an
    /// asm touches, so it uses the **union** of the two ABIs' callee-saved sets:
    /// `rbx`, `rsi`, `rdi`, `r12`–`r15`. (`rsi`/`rdi` are callee-saved on Win64
    /// but caller-saved on SysV; preserving a caller-saved register unnecessarily
    /// is harmless, whereas failing to preserve a callee-saved one corrupts the
    /// caller — so the union is the always-safe choice regardless of target.)
    /// `rbp`/`rsp` are excluded here because they are rejected outright as
    /// operands (`is_frame_register`).
    pub fn is_callee_saved_union(self) -> bool {
        matches!(self.code, 3 | 6 | 7 | 12 | 13 | 14 | 15)
    }
}

/// The canonical 64-bit register name for an encoding `0..=15`, for diagnostics
/// (`al` reports its parent as `rax`).
pub fn reg64_name(code: u8) -> &'static str {
    match code {
        0 => "rax",
        1 => "rcx",
        2 => "rdx",
        3 => "rbx",
        4 => "rsp",
        5 => "rbp",
        6 => "rsi",
        7 => "rdi",
        8 => "r8",
        9 => "r9",
        10 => "r10",
        11 => "r11",
        12 => "r12",
        13 => "r13",
        14 => "r14",
        15 => "r15",
        _ => "?",
    }
}

/// Resolve a spelled register name to its encoding and width, or `None` if the
/// name is not a recognized x86-64 general-purpose register. The high-byte
/// registers (`ah`/`bh`/`ch`/`dh`) are intentionally omitted: they alias a
/// different encoding only in the no-REX form and are never needed by the kernel
/// primitives this surface targets; a clause naming one is an unknown register.
pub fn lookup_register(name: &str) -> Option<RegisterInfo> {
    let info = |code: u8, width: RegWidth| Some(RegisterInfo { code, width });
    match name {
        // 64-bit.
        "rax" => info(0, RegWidth::W64),
        "rcx" => info(1, RegWidth::W64),
        "rdx" => info(2, RegWidth::W64),
        "rbx" => info(3, RegWidth::W64),
        "rsp" => info(4, RegWidth::W64),
        "rbp" => info(5, RegWidth::W64),
        "rsi" => info(6, RegWidth::W64),
        "rdi" => info(7, RegWidth::W64),
        "r8" => info(8, RegWidth::W64),
        "r9" => info(9, RegWidth::W64),
        "r10" => info(10, RegWidth::W64),
        "r11" => info(11, RegWidth::W64),
        "r12" => info(12, RegWidth::W64),
        "r13" => info(13, RegWidth::W64),
        "r14" => info(14, RegWidth::W64),
        "r15" => info(15, RegWidth::W64),
        // 32-bit.
        "eax" => info(0, RegWidth::W32),
        "ecx" => info(1, RegWidth::W32),
        "edx" => info(2, RegWidth::W32),
        "ebx" => info(3, RegWidth::W32),
        "esp" => info(4, RegWidth::W32),
        "ebp" => info(5, RegWidth::W32),
        "esi" => info(6, RegWidth::W32),
        "edi" => info(7, RegWidth::W32),
        "r8d" => info(8, RegWidth::W32),
        "r9d" => info(9, RegWidth::W32),
        "r10d" => info(10, RegWidth::W32),
        "r11d" => info(11, RegWidth::W32),
        "r12d" => info(12, RegWidth::W32),
        "r13d" => info(13, RegWidth::W32),
        "r14d" => info(14, RegWidth::W32),
        "r15d" => info(15, RegWidth::W32),
        // 16-bit.
        "ax" => info(0, RegWidth::W16),
        "cx" => info(1, RegWidth::W16),
        "dx" => info(2, RegWidth::W16),
        "bx" => info(3, RegWidth::W16),
        "sp" => info(4, RegWidth::W16),
        "bp" => info(5, RegWidth::W16),
        "si" => info(6, RegWidth::W16),
        "di" => info(7, RegWidth::W16),
        "r8w" => info(8, RegWidth::W16),
        "r9w" => info(9, RegWidth::W16),
        "r10w" => info(10, RegWidth::W16),
        "r11w" => info(11, RegWidth::W16),
        "r12w" => info(12, RegWidth::W16),
        "r13w" => info(13, RegWidth::W16),
        "r14w" => info(14, RegWidth::W16),
        "r15w" => info(15, RegWidth::W16),
        // 8-bit (low byte; the REX-form spellings for codes 4..=7).
        "al" => info(0, RegWidth::W8),
        "cl" => info(1, RegWidth::W8),
        "dl" => info(2, RegWidth::W8),
        "bl" => info(3, RegWidth::W8),
        "spl" => info(4, RegWidth::W8),
        "bpl" => info(5, RegWidth::W8),
        "sil" => info(6, RegWidth::W8),
        "dil" => info(7, RegWidth::W8),
        "r8b" => info(8, RegWidth::W8),
        "r9b" => info(9, RegWidth::W8),
        "r10b" => info(10, RegWidth::W8),
        "r11b" => info(11, RegWidth::W8),
        "r12b" => info(12, RegWidth::W8),
        "r13b" => info(13, RegWidth::W8),
        "r14b" => info(14, RegWidth::W8),
        "r15b" => info(15, RegWidth::W8),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_share_one_encoding() {
        for name in ["rax", "eax", "ax", "al"] {
            assert_eq!(lookup_register(name).unwrap().code, 0);
        }
        for name in ["r12", "r12d", "r12w", "r12b"] {
            assert_eq!(lookup_register(name).unwrap().code, 12);
        }
    }

    #[test]
    fn widths_are_distinguished() {
        assert_eq!(lookup_register("rax").unwrap().width, RegWidth::W64);
        assert_eq!(lookup_register("eax").unwrap().width, RegWidth::W32);
        assert_eq!(lookup_register("ax").unwrap().width, RegWidth::W16);
        assert_eq!(lookup_register("al").unwrap().width, RegWidth::W8);
    }

    #[test]
    fn callee_saved_union_and_frame_regs() {
        for name in ["rbx", "rsi", "rdi", "r12", "r13", "r14", "r15"] {
            assert!(lookup_register(name).unwrap().is_callee_saved_union());
        }
        for name in ["rax", "rcx", "rdx", "r8", "r9", "r10", "r11"] {
            assert!(!lookup_register(name).unwrap().is_callee_saved_union());
        }
        assert!(lookup_register("rsp").unwrap().is_frame_register());
        assert!(lookup_register("rbp").unwrap().is_frame_register());
        assert!(!lookup_register("rbx").unwrap().is_frame_register());
    }

    #[test]
    fn unknown_names_rejected() {
        for name in ["ah", "bh", "rax1", "xmm0", "", "foo"] {
            assert!(lookup_register(name).is_none());
        }
    }
}
