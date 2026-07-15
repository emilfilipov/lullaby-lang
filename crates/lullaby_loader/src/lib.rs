//! Multi-file module loading and project-manifest resolution for Lullaby.
//!
//! This crate is the shared frontend stage that turns a project (or a lone
//! entry file) into a single merged [`loader::LoadedProgram`] the semantic
//! analyzer and every backend consume unchanged. It was extracted verbatim from
//! the CLI so that other tools — notably the language server (`lullaby_lsp`) —
//! can run the *same* module resolution the CLI does, rather than analyzing a
//! single open buffer in isolation.
//!
//! - [`manifest`] parses and resolves `lullaby.json` project manifests into an
//!   ordered set of source search directories.
//! - [`loader`] lexes/parses the entry file plus every transitively imported
//!   module, enforces the module rules (`L0391`/`L0392`/`L0393`/`L0397`), and
//!   merges every module's declarations into one flat program.
//!
//! The loader additionally exposes an *overlay* API
//! ([`loader::SourceOverlay`]): a map of path -> in-memory source text that
//! takes precedence over reading the file from disk. This is what lets the
//! language server analyze the merged program using the editor's live,
//! possibly-unsaved buffers instead of stale on-disk bytes, while leaving the
//! CLI's on-disk behavior byte-for-byte identical (it always passes an empty
//! overlay).

pub mod loader;
pub mod manifest;
