//! Front-end driving shared by every command: resolve a CLI path argument to a
//! build target (single `.lby` file or `lullaby.json` project), run it through
//! the module loader and the semantic validator, and hand back a
//! [`CompiledSource`].

use std::{collections::HashMap, fs, path::Path, path::PathBuf};

use lullaby_diagnostics::{DiagnosticPhase, DiagnosticReport};
use lullaby_ir::{OptimizationConfig, optimize};
use lullaby_loader::{loader, loader::ModuleSource, manifest};
use lullaby_parser::{ModuleOrigins, Program, report_origin_key};
use lullaby_semantics::{CheckedProgram, SemanticDiagnostic, validate, validate_executable};

use crate::args::OptimizationMode;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CompiledSource {
    pub(crate) path: PathBuf,
    pub(crate) source: String,
    pub(crate) checked: CheckedProgram,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompileFailure {
    pub(crate) reports: Vec<DiagnosticReport>,
    pub(crate) source: Option<String>,
}

impl CompileFailure {
    fn with_source(reports: Vec<DiagnosticReport>, source: String) -> Self {
        Self {
            reports,
            source: Some(source),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceMode {
    Library,
    Executable,
}

/// What a CLI path argument resolved to.
enum BuildTarget {
    /// A single `.lby` file (legacy behavior) or a project with an executable
    /// entry: load from `entry`, searching `search_dirs` for imports. The
    /// `report_path` is what diagnostics are attributed to.
    Entry {
        entry: PathBuf,
        search_dirs: Vec<PathBuf>,
    },
    /// A library project with no `entry`: load and validate every module across
    /// the project's `search_dirs`. Only valid for `check`/`test`.
    Library { search_dirs: Vec<PathBuf> },
}

/// Decide whether the CLI argument is a single `.lby` file (legacy behavior) or a
/// project directory / `lullaby.json` manifest (multi-package build), and produce
/// the corresponding [`BuildTarget`]. A bare `.lby` file resolves exactly as
/// before, so single-file behavior is byte-for-byte unchanged.
fn resolve_target(path: &Path, source_mode: SourceMode) -> Result<BuildTarget, CompileFailure> {
    let Some((dir, manifest_path)) = manifest::manifest_path_for(path) else {
        // Not a project: treat as a single `.lby` file exactly as before.
        return Ok(BuildTarget::Entry {
            entry: path.to_path_buf(),
            search_dirs: Vec::new(),
        });
    };

    let project =
        manifest::load_manifest(&dir, &manifest_path).map_err(|report| CompileFailure {
            reports: vec![*report],
            source: None,
        })?;

    match project.entry {
        Some(entry) => Ok(BuildTarget::Entry {
            entry,
            search_dirs: project.search_dirs,
        }),
        None => {
            // `run`/`build`/`compile`/`wasm`/`native` need an executable entry;
            // `check`/`test` (Library mode) can validate every module instead.
            if source_mode == SourceMode::Executable {
                return Err(CompileFailure {
                    reports: vec![
                        DiagnosticReport::new(
                            "L0343",
                            DiagnosticPhase::Loader,
                            format!(
                                "project `{}` has no `entry`; add an `entry` to `{}` to run, build, or compile it",
                                project.manifest.name,
                                manifest_path.display()
                            ),
                        )
                        .with_source_path(manifest_path.display().to_string()),
                    ],
                    source: None,
                });
            }
            Ok(BuildTarget::Library {
                search_dirs: project.search_dirs,
            })
        }
    }
}

/// Turns a [`SemanticDiagnostic`] into a [`DiagnosticReport`] that names the file
/// its span actually belongs to.
///
/// The module loader merges every loaded module into one flat program, and a
/// span is only a `(line, column)` pair — so attributing every semantic
/// diagnostic to the entry file sends the reader to the wrong file at a line
/// number that belongs to a different one. (Observed: a 5-line `no-runtime`
/// entry that imports a heap-using helper reported the helper's `L0441` at
/// `<entry>:2:9`, where line 2 of the entry is the `import` line itself.)
///
/// The loader's per-declaration origin table (`Program::origins`) is what
/// survives the merge, so attribution is keyed on the diagnostic's enclosing
/// declaration name. That makes the fix **general to every semantic
/// diagnostic** that names its declaration, not specific to `L0441`. A
/// diagnostic with no declaration name, or one whose name the table cannot
/// attribute unambiguously, falls back to the entry file exactly as before.
struct ModuleAttribution<'a> {
    origins: &'a ModuleOrigins,
    /// Origin path -> that module's source text, for the caret line.
    sources: HashMap<String, &'a str>,
    entry_path: String,
}

impl<'a> ModuleAttribution<'a> {
    fn new(program: &'a Program, modules: &'a [ModuleSource], entry: &Path) -> Self {
        Self {
            origins: &program.origins,
            sources: modules
                .iter()
                .map(|module| (module.path.display().to_string(), module.source.as_str()))
                .collect(),
            entry_path: entry.display().to_string(),
        }
    }

    /// The source path a diagnostic belongs to, when that is a module other than
    /// the entry file.
    fn origin_path(&self, diagnostic: &SemanticDiagnostic) -> Option<&str> {
        let function = diagnostic.function.as_deref()?;
        let origin = self.origins.path_for(&report_origin_key(function))?;
        (origin != self.entry_path).then_some(origin)
    }

    fn report(&self, diagnostic: SemanticDiagnostic) -> DiagnosticReport {
        let origin = self.origin_path(&diagnostic);
        let mut report = DiagnosticReport::new(
            diagnostic.code,
            DiagnosticPhase::Semantic,
            diagnostic.message,
        )
        .with_source_path(origin.unwrap_or(&self.entry_path));
        if let Some(span) = diagnostic.span {
            report = report.with_span(span);
            // Verbose rendering is handed the *entry* file's text, which this
            // span does not index, so carry the correct line on the report.
            if let Some(text) = origin.and_then(|origin| self.sources.get(origin))
                && let Some(line) = text.lines().nth(span.line.saturating_sub(1))
            {
                report = report.with_source_line(line);
            }
        }
        if let Some(function) = diagnostic.function {
            report = report.with_function(function);
        }
        report
    }
}

pub(crate) fn compile(
    path: &Path,
    source_mode: SourceMode,
) -> Result<CompiledSource, CompileFailure> {
    let target = resolve_target(path, source_mode)?;

    // The module loader lexes, parses, resolves imports, enforces visibility and
    // no-shadowing, and merges every module into one flat program. A single-file
    // program with no imports behaves exactly as a direct lex+parse would; a
    // project build additionally searches its `src` and dependency directories.
    let (loaded, report_path) = match &target {
        BuildTarget::Entry { entry, search_dirs } => (
            loader::load_program_in_project(entry, search_dirs),
            entry.clone(),
        ),
        BuildTarget::Library { search_dirs } => (
            loader::load_library_project(search_dirs),
            search_dirs
                .first()
                .cloned()
                .unwrap_or_else(|| path.to_path_buf()),
        ),
    };
    let loaded = match loaded {
        Ok(loaded) => loaded,
        Err(reports) => {
            // Attach the entry source when we have it so verbose rendering can
            // show source context for entry-file diagnostics.
            let source = fs::read_to_string(&report_path).ok();
            return Err(CompileFailure { reports, source });
        }
    };
    let path = &report_path;
    let program = loaded.program;
    let source = loaded.entry_source;
    let modules = loaded.modules;

    let checked = match match source_mode {
        SourceMode::Library => validate(&program),
        SourceMode::Executable => validate_executable(&program),
    } {
        Ok(checked) => checked,
        Err(diagnostics) => {
            let attribution = ModuleAttribution::new(&program, &modules, path);
            return Err(CompileFailure::with_source(
                diagnostics
                    .into_iter()
                    .map(|diagnostic| attribution.report(diagnostic))
                    .collect(),
                source,
            ));
        }
    };

    Ok(CompiledSource {
        path: path.clone(),
        source,
        checked,
    })
}

pub(crate) fn optimize_module(
    module: lullaby_ir::IrModule,
    optimization: OptimizationMode,
) -> lullaby_ir::IrModule {
    match optimization {
        OptimizationMode::None => module,
        OptimizationMode::ConstantFold => {
            let (module, _report) = optimize(&module, &OptimizationConfig::constant_folding());
            module
        }
        OptimizationMode::DeadCode => {
            let (module, _report) = optimize(&module, &OptimizationConfig::dead_code_elimination());
            module
        }
        OptimizationMode::Full => {
            let (module, _report) = optimize(&module, &OptimizationConfig::full());
            module
        }
    }
}
