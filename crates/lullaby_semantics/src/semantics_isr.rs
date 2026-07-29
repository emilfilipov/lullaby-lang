//! Freestanding-tier **interrupt / naked function** enforcement — stage 8 of the
//! freestanding / kernel tier (`documents/freestanding_tier_design.md` §6).
//!
//! Two prefix keywords declare a function the *hardware* enters rather than a
//! caller:
//!
//! * **`interrupt fn NAME frame ptr<T>`** (optionally `, error_code u64`) — an
//!   interrupt-service routine. The native backend gives it the ISR calling
//!   convention: save the full register set, realign the stack, run the body,
//!   restore, and terminate with `iretq` instead of `ret`.
//! * **`naked fn NAME`** — a function for which the compiler emits *no* prologue,
//!   epilogue, or implicit `ret`. Its body is inline `asm` only; the author writes
//!   every instruction, including the return.
//!
//! This pass owns both gates:
//!
//! * **Tier gate (`L0441`).** Both kinds are `no-runtime`-tier constructs; one in
//!   a module without the directive is rejected, exactly as §6 specifies. This
//!   check lives here rather than in [`crate::semantics_no_runtime`] because that
//!   pass is a no-op for a module *without* the directive, which is precisely the
//!   case that must be caught.
//! * **Shape gate (`L0446`).** Every signature and body constraint below.
//!
//! # The constraints, and why each exists
//!
//! An ISR's contract is dictated by the CPU, not by Lullaby, so the checks are
//! not stylistic — each one guards a shape the backend physically cannot honor.
//!
//! | Constraint | Why |
//! | :-- | :-- |
//! | `interrupt fn` takes 1 or 2 parameters | The CPU passes nothing in registers; the emitter materializes exactly the interrupt frame pointer and (on an error-code vector) the CPU-pushed error code from the stack. |
//! | parameter 1 is a raw `ptr<T>` | It addresses the CPU-pushed frame in place. |
//! | parameter 2, if present, is `u64` | The x86-64 error code is a 64-bit stack word. |
//! | `interrupt fn` returns `void` | The return is `iretq`, which pops the interrupted context; there is no register left carrying a result to a caller that does not exist. |
//! | `naked fn` takes no parameters and returns `void` | There is no frame, so no parameter can be seated and no result convention applies. |
//! | a `naked fn` body is `asm` only | The definition of naked: whatever the compiler emitted around the author's instructions would corrupt a hand-written entry sequence. |
//! | a `naked fn`'s `asm` binds no operands or clobbers | The operand marshaller stages inputs and saves clobbered callee-saved registers in **`rbp`-relative frame slots** (`crates/lullaby_ir/src/native_object_asm.rs`), and a naked function has no frame to hold them. This is §6's own rule — "referencing Lullaby locals is unavailable because there is no frame" — applied to the delivered operand form. |
//! | neither kind may be **called** from Lullaby | Their return instruction is not `ret`: a `call` to an `interrupt fn` would execute `iretq` and pop three words of the caller's stack as `rip`/`cs`/`rflags`. There is no diagnosable failure at runtime, only a wild jump, so the call is refused at compile time. |
//! | neither kind is generic | A generic function is monomorphized per instantiation; a hardware entry point is named once by an IDT entry or a bootloader. |
//!
//! Modifier exclusivity (`interrupt`/`naked` with each other or with
//! `pub`/`async`/`extern`/`export`) is rejected earlier, by the parser, as
//! `L0201` — the same place the delivered `export`/`extern`/`async` pairwise
//! rules live.

use lullaby_parser::{Function, Program, Stmt, TypeRef};

use crate::SemanticDiagnostic;

/// The kind of hardware-entry function, used only to name it in a diagnostic.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryKind {
    Interrupt,
    Naked,
}

impl EntryKind {
    /// Classify a resolved signature. `None` for an ordinary function, which is
    /// every function in a program that uses neither keyword.
    pub(crate) fn of_signature(signature: &crate::Signature) -> Option<Self> {
        match (signature.is_interrupt, signature.is_naked) {
            (true, _) => Some(EntryKind::Interrupt),
            (_, true) => Some(EntryKind::Naked),
            _ => None,
        }
    }

    /// The keyword with its indefinite article and opening backtick, so messages
    /// read "an `interrupt fn`" and "a `naked fn`" rather than "a `interrupt fn`".
    fn article_keyword(self) -> &'static str {
        match self {
            EntryKind::Interrupt => "an `interrupt",
            EntryKind::Naked => "a `naked",
        }
    }

    /// Why this kind cannot be reached through the ordinary call ABI.
    fn not_callable_reason(self) -> &'static str {
        match self {
            EntryKind::Interrupt => {
                "it returns with `iretq`, not `ret`, so a `call` would pop the caller's \
                 stack as an interrupt frame — the CPU enters it from an IDT entry instead"
            }
            EntryKind::Naked => {
                "the compiler emits no prologue, epilogue, or `ret` for it — its control \
                 flow is whatever its `asm` writes, so only a bootloader or hand-written \
                 `asm` may enter it"
            }
        }
    }
}

/// How a hardware entry point was reached where an ordinary function was expected.
/// The two routes are diagnosed separately because they need different advice, but
/// they are the *same* rule and share one code (`L0446`) and one reason string.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum EntryMisuse {
    /// A direct named call, `timer_isr(p)`.
    Called,
    /// The bare name used as a first-class `fn(...)` **value** — bound to a local,
    /// passed to a higher-order function, or returned.
    UsedAsValue,
}

/// Build the `L0446` for a hardware entry point used as an ordinary function, or
/// `None` when `signature` is an ordinary function (the overwhelmingly common
/// case, so this costs two `bool` reads).
///
/// Both misuse routes funnel through here so the two sites cannot drift — in
/// wording, in grammar (`an `interrupt fn`` / `a `naked fn``), or, far more
/// importantly, in *coverage*. Guarding only the direct-call route left the value
/// route open: `let f fn(ptr<i64>) -> void = timer_isr` then `f(p)` type-checked
/// and **ran the ISR body** on all three interpreters, while native happened to
/// decline for an unrelated reason (a fn-typed local that is not a closure literal
/// or factory call is outside the native subset). That is a silent tier divergence
/// today and would become a real `call`-into-`iretq` the moment native gains
/// first-class function values.
pub(crate) fn entry_misuse_diagnostic(
    signature: &crate::Signature,
    name: &str,
    misuse: EntryMisuse,
    in_function: &str,
    span: lullaby_diagnostics::Span,
) -> Option<SemanticDiagnostic> {
    let kind = EntryKind::of_signature(signature)?;
    let keyword = kind.article_keyword();
    let reason = kind.not_callable_reason();
    let message = match misuse {
        EntryMisuse::Called => {
            format!("`{name}` is {keyword} fn` and cannot be called: {reason}")
        }
        EntryMisuse::UsedAsValue => format!(
            "`{name}` is {keyword} fn` and cannot be used as a function value: a \
             `fn(...)` value exists to be invoked, and every consumer of one — a local, \
             a higher-order parameter, a returned callable — reaches it through the \
             ordinary call ABI, which does not apply here ({reason})"
        ),
    };
    Some(SemanticDiagnostic::at(
        "L0446",
        message,
        Some(in_function.to_string()),
        span,
    ))
}

/// The type of a bare top-level function name used as a first-class value:
/// `fn(params) -> ret`, or the `L0446` that refuses a hardware entry point.
///
/// This is the **value** half of the not-callable rule, and it is the half that a
/// first implementation missed. Guarding only the direct named call in
/// `check_call` left an alias route wide open: `let f fn(ptr<i64>) -> void =
/// timer_isr` followed by `f(p)` type-checked cleanly and **ran the ISR body** on
/// all three interpreters, and passing the name to a higher-order function
/// (`apply(timer_isr, p)`) did the same. Native happened to decline, but only
/// because a fn-typed local that is not a closure literal or a factory call is
/// outside its subset — an accident, and one that would stop protecting anything
/// the moment native gains first-class function values, at which point the
/// program would emit a real `call` into a body that ends in `iretq`.
///
/// Refusing the value rather than enumerating call shapes is what makes the rule
/// complete: every sink for a `fn(...)` value — a local, a higher-order argument,
/// a returned callable, a stored one — reaches it through the ordinary call ABI,
/// so refusing to produce the value closes all of them at once.
pub(crate) fn function_value_type(
    signature: &crate::Signature,
    name: &str,
    in_function: &str,
    span: lullaby_diagnostics::Span,
) -> Result<TypeRef, SemanticDiagnostic> {
    match entry_misuse_diagnostic(signature, name, EntryMisuse::UsedAsValue, in_function, span) {
        Some(diagnostic) => Err(diagnostic),
        None => Ok(crate::function_type(
            &signature.params,
            &signature.return_type,
        )),
    }
}

/// Whether `ty` spells a raw pointer. Accepts the canonical generic spelling
/// `ptr<T>` and the legacy flat `ptr_T` spelling that `alloc` produces, matching
/// the native backend's `is_raw_pointer_type_name`.
fn is_raw_pointer(ty: &TypeRef) -> bool {
    ty.name == "ptr" || ty.name.starts_with("ptr<") || ty.name.starts_with("ptr_")
}

/// Enforce §6's interrupt/naked rules over `program`, appending diagnostics.
///
/// `resolved_returns` is the checker's inferred-return table, consulted for a
/// function declared without an explicit `-> T` so that an omitted return clause
/// is judged by the type the body actually yields rather than by the
/// `INFERRED_RETURN` sentinel.
///
/// A no-op for a program with no `interrupt fn` / `naked fn`, so ordinary
/// programs are completely unaffected.
pub(crate) fn enforce(
    program: &Program,
    resolved_returns: &std::collections::HashMap<String, TypeRef>,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) {
    for function in &program.functions {
        let kind = match (function.is_interrupt, function.is_naked) {
            (true, _) => EntryKind::Interrupt,
            (_, true) => EntryKind::Naked,
            _ => continue,
        };

        // Tier gate first: outside the freestanding tier the construct does not
        // exist at all, so reporting its shape on top would be noise.
        //
        // The tier is a **module** property, not a program property. A multi-file
        // build is merged into one flat `Program` before this pass runs, so
        // `is_no_runtime` alone (the whole-unit flag) would mis-gate a mixed-tier
        // build in both directions: it would accept an `interrupt fn` written in a
        // *hosted* module of a program whose other module is freestanding, and
        // reject a legitimate one in a freestanding module of a program whose
        // top-level file is hosted. `Program::origins` carries the loader's
        // per-declaration attribution; consulting both is exactly what
        // `semantics_no_runtime::TierScope` does for `L0441`'s other half.
        let in_freestanding_module = program.is_no_runtime
            || program
                .origins
                .is_freestanding(&lullaby_parser::decl_origin_key(&function.name));
        if !in_freestanding_module {
            diagnostics.push(SemanticDiagnostic::at(
                "L0441",
                format!(
                    "{} fn` is a freestanding-tier construct and is unavailable in a \
                     module without the `no-runtime` directive (it has a hardware calling \
                     convention, not the safe tier's); add `no-runtime` as the module's \
                     first line, or declare an ordinary `fn`",
                    kind.article_keyword()
                ),
                Some(function.name.clone()),
                function.span,
            ));
            continue;
        }

        check_not_generic(function, kind, diagnostics);
        check_returns_void(function, kind, resolved_returns, diagnostics);
        match kind {
            EntryKind::Interrupt => check_interrupt_params(function, diagnostics),
            EntryKind::Naked => {
                check_naked_params(function, diagnostics);
                check_naked_body(function, diagnostics);
            }
        }
    }
}

fn report(
    function: &Function,
    span: lullaby_diagnostics::Span,
    message: String,
) -> SemanticDiagnostic {
    SemanticDiagnostic::at("L0446", message, Some(function.name.clone()), span)
}

fn check_not_generic(
    function: &Function,
    kind: EntryKind,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) {
    if function.type_params.is_empty() {
        return;
    }
    diagnostics.push(report(
        function,
        function.span,
        format!(
            "{} fn` may not be generic: it is a single hardware entry point named \
             once by an IDT entry or a bootloader, and there is no call site to infer \
             a type argument from",
            kind.article_keyword()
        ),
    ));
}

fn check_returns_void(
    function: &Function,
    kind: EntryKind,
    resolved_returns: &std::collections::HashMap<String, TypeRef>,
    diagnostics: &mut Vec<SemanticDiagnostic>,
) {
    // A function declared without `-> T` carries the inference sentinel here;
    // consult the type the checker inferred for its body instead.
    let declared = &function.return_type;
    let effective = if declared.name == lullaby_parser::INFERRED_RETURN {
        resolved_returns
            .get(&function.name)
            .cloned()
            .unwrap_or_else(|| TypeRef::new("void"))
    } else {
        declared.clone()
    };
    // The sentinel can survive the lookup: a body whose last statement is an
    // inline `asm` satisfies the final-value requirement by echoing the declared
    // return type, which for an un-annotated function is the sentinel itself. That
    // body yielded no value type, so it is `void` — the shape every well-formed
    // `naked fn` has. `validate` performs the same collapse when it writes the
    // inferred types back into the program.
    if effective.name == "void" || effective.name == lullaby_parser::INFERRED_RETURN {
        return;
    }
    let why = match kind {
        EntryKind::Interrupt => {
            "an interrupt handler returns with `iretq`, which pops the interrupted \
             context — there is no caller to receive a result"
        }
        EntryKind::Naked => {
            "a naked function has no frame and no compiler-generated return, so no \
             result convention applies"
        }
    };
    diagnostics.push(report(
        function,
        function.span,
        format!(
            "{} fn` must return `void`, but `{}` returns `{}`: {why}",
            kind.article_keyword(),
            function.name,
            effective.name
        ),
    ));
}

fn check_interrupt_params(function: &Function, diagnostics: &mut Vec<SemanticDiagnostic>) {
    if function.params.is_empty() || function.params.len() > 2 {
        diagnostics.push(report(
            function,
            function.span,
            format!(
                "an `interrupt fn` takes the CPU-pushed interrupt frame as its only \
                 parameter, plus an optional second `error_code u64` for the vectors \
                 that push one — but `{}` declares {} parameters",
                function.name,
                function.params.len()
            ),
        ));
        return;
    }
    let frame = &function.params[0];
    if !is_raw_pointer(&frame.ty) {
        diagnostics.push(report(
            function,
            function.span,
            format!(
                "an `interrupt fn`'s first parameter addresses the CPU-pushed \
                 interrupt frame in place, so it must be a raw pointer `ptr<T>`, but \
                 `{}` is declared `{}`",
                frame.name, frame.ty.name
            ),
        ));
    }
    if let Some(error_code) = function.params.get(1)
        && error_code.ty.name != "u64"
    {
        diagnostics.push(report(
            function,
            function.span,
            format!(
                "an `interrupt fn`'s second parameter is the CPU-pushed error code, a \
                 64-bit unsigned stack word, so it must be declared `u64`, but `{}` is \
                 declared `{}`",
                error_code.name, error_code.ty.name
            ),
        ));
    }
}

fn check_naked_params(function: &Function, diagnostics: &mut Vec<SemanticDiagnostic>) {
    if function.params.is_empty() {
        return;
    }
    diagnostics.push(report(
        function,
        function.span,
        format!(
            "a `naked fn` takes no parameters: the compiler emits no prologue, so \
             there is no frame to seat one in — but `{}` declares {}",
            function.name,
            function.params.len()
        ),
    ));
}

/// A `naked fn` body must be inline `asm` and nothing else. `unsafe` blocks are
/// walked through rather than rejected because `asm` *requires* one (`L0330`), so
/// every well-formed naked body is at least one `unsafe` wrapper deep.
///
/// There is deliberately no separate "the body must contain at least one `asm`"
/// rule here. The grammar has no empty block — a `fn`/`unsafe`/`region` header
/// must be followed by an indented statement, and a comment-only body is a parser
/// error (`L0205`) — so a naked body containing no `asm` necessarily contains some
/// other statement, which the walk below already reports. A rule that can never
/// fire would be dead. The corresponding *backend* guard in `lower_naked_function`
/// is kept, because it is a default-deny skip rather than a user diagnostic and is
/// what would catch a body emptied by a future IR-level transformation.
fn check_naked_body(function: &Function, diagnostics: &mut Vec<SemanticDiagnostic>) {
    check_naked_block(function, &function.body, diagnostics);
}

fn check_naked_block(
    function: &Function,
    body: &[Stmt],
    diagnostics: &mut Vec<SemanticDiagnostic>,
) {
    for stmt in body {
        match stmt {
            Stmt::Unsafe { body, .. } => check_naked_block(function, body, diagnostics),
            Stmt::Asm {
                operands,
                clobbers,
                span,
                ..
            } => {
                // The operand marshaller stages inputs and preserves clobbered
                // callee-saved registers in `rbp`-relative frame slots. A naked
                // function has no frame, so binding either would write through a
                // base pointer that still belongs to the interrupted/calling
                // context. Refused rather than silently emitted.
                if !operands.is_empty() {
                    diagnostics.push(report(
                        function,
                        *span,
                        "an `asm` inside a `naked fn` may not bind `in`/`out` operands: \
                         operand marshalling stages values in frame slots addressed \
                         through `rbp`, and a naked function has no frame — write the \
                         value into the register with an explicit instruction instead"
                            .to_string(),
                    ));
                }
                if !clobbers.is_empty() {
                    diagnostics.push(report(
                        function,
                        *span,
                        "an `asm` inside a `naked fn` may not declare `clobber`s: \
                         preserving a clobbered callee-saved register needs a frame \
                         slot, and a naked function has none — the author owns the \
                         whole register state here, so save and restore explicitly"
                            .to_string(),
                    ));
                }
            }
            other => {
                diagnostics.push(report(
                    function,
                    naked_stmt_span(other).unwrap_or(function.span),
                    format!(
                        "a `naked fn` body may contain only `asm` statements (inside \
                         `unsafe`), but `{}` contains a {}: the compiler emits no \
                         prologue for a naked function, so a Lullaby statement would \
                         address locals through a frame that does not exist",
                        function.name,
                        naked_stmt_label(other)
                    ),
                ));
            }
        }
    }
}

/// A short, user-facing name for the statement kind found in a naked body.
fn naked_stmt_label(stmt: &Stmt) -> &'static str {
    match stmt {
        Stmt::Let { .. } => "`let` binding",
        Stmt::Assign { .. } => "assignment",
        Stmt::Return(_) => "`return`",
        Stmt::Break(_) => "`break`",
        Stmt::Continue(_) => "`continue`",
        Stmt::Expr(_) => "expression statement",
        Stmt::If { .. } => "`if`",
        Stmt::While { .. } => "`while` loop",
        Stmt::For { .. } | Stmt::ForEach { .. } => "`for` loop",
        Stmt::Loop { .. } => "`loop`",
        Stmt::Region(_) | Stmt::RegionBlock { .. } => "`region`",
        Stmt::Throw { .. } => "`throw`",
        Stmt::Try { .. } => "`try`",
        // Reached only through the outer match's `other` arm, which never
        // forwards these two.
        Stmt::Unsafe { .. } => "`unsafe` block",
        Stmt::Asm { .. } => "`asm` statement",
    }
}

/// The span to blame for a statement found in a naked body. `Stmt` has no
/// uniform accessor, so the spans are read per variant; the variants that carry
/// only a sub-expression report that expression's span. A bare `return` carries
/// neither, so it yields `None` and the caller blames the declaration.
fn naked_stmt_span(stmt: &Stmt) -> Option<lullaby_diagnostics::Span> {
    Some(match stmt {
        Stmt::Let { span, .. }
        | Stmt::Assign { span, .. }
        | Stmt::If { span, .. }
        | Stmt::While { span, .. }
        | Stmt::For { span, .. }
        | Stmt::ForEach { span, .. }
        | Stmt::Loop { span, .. }
        | Stmt::Unsafe { span, .. }
        | Stmt::Asm { span, .. }
        | Stmt::RegionBlock { span, .. }
        | Stmt::Throw { span, .. }
        | Stmt::Try { span, .. } => *span,
        Stmt::Break(span) | Stmt::Continue(span) => *span,
        Stmt::Region(decl) => decl.span,
        Stmt::Return(Some(expr)) | Stmt::Expr(expr) => expr.span,
        Stmt::Return(None) => return None,
    })
}
