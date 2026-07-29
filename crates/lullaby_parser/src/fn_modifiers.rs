//! The prefix keywords that may precede a top-level declaration, and the
//! exclusivity rules between them.
//!
//! Lullaby spells declaration modifiers as prefix keywords rather than
//! attributes or sigils (`pub fn`, `export fn`, `interrupt fn`), so the whole set
//! is gathered in declaration order before the parser knows which kind of
//! declaration follows. This module owns that gathering and every pairwise
//! `L0201` rejection, split out of `lib.rs` to keep the top-level parse loop
//! readable and that file from growing further past the size cap — the rules are
//! a self-contained matrix that grows with each new keyword, not parsing logic.
//!
//! # Two categories, and why the distinction decides the matrix
//!
//! * **Visibility / ABI adjectives** — `pub`, `async`, `extern`, `export`. These
//!   describe how an *ordinary* function is reached: from another Lullaby module,
//!   on a spawned thread, through the C ABI. Several combine freely (`pub async
//!   fn`, `pub extern fn`); the delivered rejections between them are the ones
//!   whose meanings genuinely conflict (`extern` imports a body-less symbol,
//!   `export` defines one, `async` needs a runtime neither has).
//! * **Function kinds** — `interrupt`, `naked`
//!   (`documents/freestanding_tier_design.md` §6). These replace the calling
//!   convention wholesale, so they are exclusive with *each other and with every
//!   adjective above*. The reason is uniform and worth stating once: `pub` and
//!   `export` advertise a function as **callable**, but an `interrupt fn` returns
//!   with `iretq` and a `naked fn` returns however its `asm` says — neither
//!   honors the ordinary call ABI, and a `call` reaching one is undefined
//!   behaviour (`L0446` refuses one written in Lullaby, in both its direct-call
//!   and its fn-value forms). `async` needs the safe-tier runtime the
//!   freestanding tier removes, and `extern` is a body-less import rather than a
//!   definition.

use lullaby_lexer::Span;

/// The prefix keywords gathered ahead of a declaration.
///
/// A struct rather than five positional `bool` parameters threaded into
/// `parse_function`: the flags are all the same type, so a positional list is
/// exactly the shape a transposition bug hides in — and every new prefix keyword
/// the freestanding tier adds would widen it further. `is_extern` lives here for
/// the exclusivity matrix but is not passed on to `parse_function`, because an
/// `extern fn` is body-less and takes the separate `parse_extern_function` path.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct FnModifiers {
    pub(crate) is_public: bool,
    pub(crate) is_async: bool,
    pub(crate) is_extern: bool,
    pub(crate) is_export: bool,
    pub(crate) is_interrupt: bool,
    pub(crate) is_naked: bool,
}

/// One rejected modifier pairing, ready for the parser to report as `L0201` at
/// the span of the token that follows the modifier run.
pub(crate) struct ModifierConflict {
    pub(crate) message: String,
    pub(crate) span: Span,
}

impl FnModifiers {
    /// Whether any modifier that only applies to a `fn` is present. `pub` is
    /// excluded: it also prefixes `struct`/`enum`/`alias`/`const`, so its presence
    /// is not by itself evidence that a `fn` was expected.
    pub(crate) fn requires_fn(self) -> bool {
        self.is_async || self.is_extern || self.is_export || self.is_interrupt || self.is_naked
    }

    /// The name to blame in "`X` must prefix a `fn` declaration" when a modifier
    /// run is not followed by `fn`. Reports the most specific one present, so an
    /// author who wrote `interrupt const` is told about `interrupt` rather than
    /// about an adjective they also happened to write.
    pub(crate) fn offending_modifier(self) -> &'static str {
        if self.is_extern {
            "extern"
        } else if self.is_export {
            "export"
        } else if self.is_interrupt {
            "interrupt"
        } else if self.is_naked {
            "naked"
        } else {
            "async"
        }
    }

    /// Every `L0201` pairing violation in this modifier run, in a stable order.
    /// Empty for the overwhelmingly common case of zero or one modifier.
    pub(crate) fn conflicts(self, span: Span) -> Vec<ModifierConflict> {
        let mut conflicts = Vec::new();
        let mut reject = |a: &str, b: &str| {
            conflicts.push(ModifierConflict {
                message: format!("`{a}` and `{b}` cannot be combined on a `fn` declaration"),
                span,
            });
        };

        // Adjective-vs-adjective: the delivered rules, unchanged.
        if self.is_extern && self.is_async {
            reject("extern", "async");
        }
        if self.is_export && self.is_extern {
            reject("export", "extern");
        }
        if self.is_export && self.is_async {
            reject("export", "async");
        }

        // `interrupt` and `naked` name two *different* calling conventions, so one
        // declaration cannot be both.
        if self.is_interrupt && self.is_naked {
            reject("interrupt", "naked");
        }
        // A function kind against every adjective — see the module docs for why
        // each is impossible rather than merely unusual. Only the first conflicting
        // adjective is reported per kind: the author has to fix the declaration
        // either way, and listing all four would bury the point.
        for (present, kind) in [(self.is_interrupt, "interrupt"), (self.is_naked, "naked")] {
            if !present {
                continue;
            }
            let adjective = if self.is_public {
                Some("pub")
            } else if self.is_async {
                Some("async")
            } else if self.is_extern {
                Some("extern")
            } else if self.is_export {
                Some("export")
            } else {
                None
            };
            if let Some(adjective) = adjective {
                reject(adjective, kind);
            }
        }
        conflicts
    }
}
