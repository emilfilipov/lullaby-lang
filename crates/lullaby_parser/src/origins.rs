//! Per-declaration origin attribution for a merged multi-module [`crate::Program`].
//!
//! The module loader flattens every loaded `.lby` module into one
//! [`crate::Program`], which is what makes the semantic analyzer and all five backends
//! run unchanged on a multi-file build. That flattening throws away two facts
//! that later stages genuinely need:
//!
//! 1. **Which file a declaration came from.** A [`lullaby_lexer::Span`] is only a
//!    `(line, column)` pair, so a diagnostic raised inside an imported module's
//!    body has nothing to name but the entry file — sending the reader to the
//!    wrong file at a line number that belongs to a different one.
//! 2. **Which tier a declaration belongs to.** The `no-runtime` directive is a
//!    *module* property. A single program-wide flag cannot express a build where
//!    a hosted program imports a freestanding library (legal) or where a
//!    freestanding module pulls a hosted helper into its own tier (rejected).
//!
//! [`ModuleOrigins`] is the side table that carries both facts across the merge.
//! It is keyed by declaration, not by span, because the flat namespace already
//! guarantees top-level declaration names are unique across modules (`L0391`),
//! whereas `(line, column)` pairs collide freely between files.
//!
//! A single-file program has an empty table: nothing was merged, so nothing
//! needs attributing and every consumer falls back to its existing behavior.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Where one declaration of a merged multi-module program came from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclOrigin {
    /// Displayable source path of the module that declares this key, or `None`
    /// when more than one module declares it. Top-level names are unique by
    /// `L0391`, but trait/impl *method* names are not, so an ambiguous key
    /// deliberately carries no path rather than risk naming the wrong file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// True when at least one declaring module is in the freestanding
    /// (`no-runtime`) tier. "At least one" is the conservative direction: an
    /// ambiguous key is gated rather than let through.
    #[serde(default, skip_serializing_if = "crate::ast::is_false")]
    pub is_freestanding: bool,
}

/// Declaration key -> [`DeclOrigin`] for a merged multi-module program.
///
/// Keys are produced by [`decl_origin_key`], [`trait_origin_key`], and
/// [`impl_origin_key`]. Ordered (not hashed) so a serialized program is
/// byte-stable.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleOrigins {
    entries: BTreeMap<String, DeclOrigin>,
}

impl ModuleOrigins {
    /// An empty table — the single-file case, where nothing was merged.
    pub fn new() -> Self {
        Self::default()
    }

    /// True when no declaration has been attributed (a single-file program).
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Attribute `key` to the module at `path`. Recording the same key twice
    /// (only possible for trait/impl method names) clears the path, because two
    /// candidate files means neither may be named, and keeps the freestanding
    /// bit sticky so the tier gate stays default-deny.
    pub fn record(&mut self, key: String, path: &str, is_freestanding: bool) {
        self.entries
            .entry(key)
            .and_modify(|origin| {
                if origin.path.as_deref() != Some(path) {
                    origin.path = None;
                }
                origin.is_freestanding |= is_freestanding;
            })
            .or_insert_with(|| DeclOrigin {
                path: Some(path.to_string()),
                is_freestanding,
            });
    }

    /// The origin recorded for `key`, if any.
    pub fn get(&self, key: &str) -> Option<&DeclOrigin> {
        self.entries.get(key)
    }

    /// The unambiguous source path `key` was declared in, if it is known.
    pub fn path_for(&self, key: &str) -> Option<&str> {
        self.entries.get(key)?.path.as_deref()
    }

    /// True when `key` belongs to a module in the freestanding tier. An unknown
    /// key is *not* freestanding: a caller that needs the whole-program fallback
    /// (a single-file `no-runtime` file, whose table is empty) checks
    /// `Program::is_no_runtime` alongside this.
    pub fn is_freestanding(&self, key: &str) -> bool {
        self.entries
            .get(key)
            .is_some_and(|origin| origin.is_freestanding)
    }

    /// True when any recorded declaration is freestanding — the cheap test for
    /// "is the freestanding tier gate relevant to this program at all".
    pub fn has_freestanding(&self) -> bool {
        self.entries.values().any(|origin| origin.is_freestanding)
    }
}

/// The origin key of a top-level `fn`/`struct`/`enum`/`alias`/`const`/`actor`
/// declaration: its own name, which the flat namespace makes unique (`L0391`).
///
/// This is the **tier** namespace. Only declarations `L0391` guarantees unique
/// belong in it, so a lookup here is never ambiguous and the freestanding gate
/// can never over-reach onto a same-named declaration from another module.
pub fn decl_origin_key(name: &str) -> String {
    name.to_string()
}

/// The origin key under which a name is looked up for **diagnostic
/// attribution** — a separate namespace from [`decl_origin_key`].
///
/// A diagnostic identifies its site by the enclosing declaration's *display*
/// name, and that includes names the tier namespace deliberately excludes:
/// trait names, and impl-method names.
///
/// Impl-method names are the reason. `L0398` already forbids a method sharing a
/// *free function's* name, so that particular collision cannot arise — but two
/// impls on **different types** may each declare `area`, and nothing forbids one
/// of them living in a freestanding module and the other in a hosted one. Since
/// [`ModuleOrigins::record`] ORs the freestanding bit, folding both into one
/// tier key would gate the hosted `Shape::area` because the freestanding
/// `Circle::area` shares its spelling — a false rejection of exactly the class
/// the per-module gate exists to remove. Attribution keys are free to be
/// ambiguous, because ambiguity there costs only a fallback to the entry file.
pub fn report_origin_key(name: &str) -> String {
    format!("report {name}")
}

/// The origin key of a `trait` declaration. Prefixed so it can never collide
/// with a top-level declaration name — an identifier contains no space.
pub fn trait_origin_key(name: &str) -> String {
    format!("trait {name}")
}

/// The origin key of an `impl` block, identified by the trait it implements and
/// the type it implements it for. Prefixed for the same reason as
/// [`trait_origin_key`]. An inherent impl carries an empty `trait_name`, which
/// is exactly what distinguishes it from a trait impl on the same type.
pub fn impl_origin_key(trait_name: &str, type_name: &str) -> String {
    format!("impl {trait_name} for {type_name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unrecorded_key_is_absent_and_not_freestanding() {
        let origins = ModuleOrigins::new();
        assert!(origins.is_empty());
        assert!(!origins.has_freestanding());
        assert_eq!(origins.path_for("main"), None);
        assert!(!origins.is_freestanding("main"));
    }

    #[test]
    fn a_recorded_key_carries_its_path_and_tier() {
        let mut origins = ModuleOrigins::new();
        origins.record(decl_origin_key("double"), "src/nrlib.lby", true);
        origins.record(decl_origin_key("main"), "src/host.lby", false);
        assert_eq!(origins.path_for("double"), Some("src/nrlib.lby"));
        assert!(origins.is_freestanding("double"));
        assert_eq!(origins.path_for("main"), Some("src/host.lby"));
        assert!(!origins.is_freestanding("main"));
        assert!(origins.has_freestanding());
    }

    #[test]
    fn a_key_claimed_by_two_modules_loses_its_path_but_keeps_the_tier() {
        let mut origins = ModuleOrigins::new();
        origins.record(decl_origin_key("area"), "src/a.lby", false);
        origins.record(decl_origin_key("area"), "src/b.lby", true);
        // Two candidate files, so neither may be named...
        assert_eq!(origins.path_for("area"), None);
        // ...but the tier gate stays default-deny.
        assert!(origins.is_freestanding("area"));
    }

    #[test]
    fn recording_the_same_key_from_the_same_module_keeps_the_path() {
        let mut origins = ModuleOrigins::new();
        origins.record(decl_origin_key("helper"), "src/a.lby", false);
        origins.record(decl_origin_key("helper"), "src/a.lby", false);
        assert_eq!(origins.path_for("helper"), Some("src/a.lby"));
    }

    #[test]
    fn the_key_namespaces_do_not_collide() {
        assert_ne!(decl_origin_key("Shape"), trait_origin_key("Shape"));
        assert_ne!(
            trait_origin_key("Shape"),
            impl_origin_key("Shape", "Circle")
        );
        // An inherent impl and a trait impl on the same type are distinct keys.
        assert_ne!(
            impl_origin_key("", "Circle"),
            impl_origin_key("Shape", "Circle")
        );
        // Attribution is a separate namespace from the tier namespace, so an
        // ambiguous display name can never leak into the freestanding gate.
        assert_ne!(decl_origin_key("area"), report_origin_key("area"));
        assert_ne!(report_origin_key("area"), trait_origin_key("area"));
    }

    /// Two impls on *different types* may each declare `area` — `L0398` only
    /// rules out a method sharing a **free function's** name. When one impl is
    /// freestanding and the other hosted, the shared display name must blur only
    /// the *attribution* key; the tier keys stay separate, so the hosted impl is
    /// not gated by the freestanding one's spelling.
    #[test]
    fn an_ambiguous_display_name_does_not_touch_the_tier_key() {
        let mut origins = ModuleOrigins::new();
        // A hosted `impl Shape for Square` with a method `area`...
        origins.record(impl_origin_key("Shape", "Square"), "host.lby", false);
        origins.record(report_origin_key("area"), "host.lby", false);
        // ...and a freestanding `impl Shape for Circle` whose method is also `area`.
        origins.record(impl_origin_key("Shape", "Circle"), "nr.lby", true);
        origins.record(report_origin_key("area"), "nr.lby", true);

        assert!(
            !origins.is_freestanding(&impl_origin_key("Shape", "Square")),
            "the hosted impl must NOT be gated because a freestanding impl \
             happens to share a method name"
        );
        assert!(origins.is_freestanding(&impl_origin_key("Shape", "Circle")));
        assert_eq!(
            origins.path_for(&report_origin_key("area")),
            None,
            "two candidate files, so the display name alone names neither"
        );
    }
}
