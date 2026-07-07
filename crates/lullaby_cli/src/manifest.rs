//! Project manifest (`lullaby.json`) loading and resolution.
//!
//! A Lullaby *project* is a directory containing a `lullaby.json` manifest that
//! names the project, an optional executable entry file, one or more source
//! directories, and zero or more local path dependencies (other project roots
//! that each contain their own `lullaby.json`). This module parses and validates
//! the manifest with `serde_json`, then resolves the full set of source search
//! directories a build should see: the project's own `src` directories followed
//! by the `src` directories of every transitively resolved dependency.
//!
//! Remote/registry dependency *fetching* is deferred; dependencies are local
//! paths only. All manifest/resolution failures are reported as `L0343` loader
//! diagnostics.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use lullaby_diagnostics::{DiagnosticPhase, DiagnosticReport};
use serde::Deserialize;

/// The canonical manifest file name at a project root.
pub const MANIFEST_FILE_NAME: &str = "lullaby.json";

/// A parsed `lullaby.json` manifest.
///
/// All paths are stored exactly as written (relative to the manifest directory);
/// resolution against the manifest directory happens in [`ResolvedProject`].
#[derive(Debug, Clone, Deserialize)]
pub struct ProjectManifest {
    /// The project name.
    pub name: String,
    /// The executable entry `.lby` file, relative to the manifest directory.
    /// Optional: library projects have no entry.
    #[serde(default)]
    pub entry: Option<String>,
    /// Source directories relative to the manifest directory. Defaults to
    /// `["."]` when omitted.
    #[serde(default = "default_src")]
    pub src: Vec<String>,
    /// Local path dependencies: dependency name -> path to another project root
    /// (a directory containing its own `lullaby.json`), relative to the manifest
    /// directory. Defaults to empty.
    #[serde(default)]
    pub dependencies: std::collections::BTreeMap<String, String>,
}

fn default_src() -> Vec<String> {
    vec![".".to_string()]
}

/// A fully resolved project: the manifest, its root directory, the absolute
/// entry path (if any), and the ordered list of source search directories that a
/// build should consult (this project's `src` dirs first, then every dependency's
/// `src` dirs, transitively, de-duplicated).
#[derive(Debug, Clone)]
pub struct ResolvedProject {
    pub manifest: ProjectManifest,
    pub entry: Option<PathBuf>,
    pub search_dirs: Vec<PathBuf>,
}

/// Build an `L0343` loader diagnostic for a manifest/resolution failure.
fn manifest_error(message: String, path: &Path) -> Box<DiagnosticReport> {
    Box::new(
        DiagnosticReport::new("L0343", DiagnosticPhase::Loader, message)
            .with_source_path(path.display().to_string()),
    )
}

/// Given a CLI path argument that is either a project directory or a
/// `lullaby.json` file, return the directory that contains the manifest and the
/// manifest file path itself. Returns `None` when the argument is neither (the
/// caller then treats it as a single `.lby` file, preserving legacy behavior).
pub fn manifest_path_for(arg: &Path) -> Option<(PathBuf, PathBuf)> {
    if arg.is_dir() {
        let manifest = arg.join(MANIFEST_FILE_NAME);
        if manifest.is_file() {
            return Some((arg.to_path_buf(), manifest));
        }
        return None;
    }
    if arg.is_file() && arg.file_name().and_then(|name| name.to_str()) == Some(MANIFEST_FILE_NAME) {
        let dir = arg.parent().map(Path::to_path_buf).unwrap_or_default();
        return Some((dir, arg.to_path_buf()));
    }
    None
}

/// Parse and validate the manifest at `manifest_path` (whose directory is `dir`),
/// then resolve dependencies transitively to build the search-directory list.
pub fn load_manifest(
    dir: &Path,
    manifest_path: &Path,
) -> Result<ResolvedProject, Box<DiagnosticReport>> {
    let mut visited = HashSet::new();
    let mut search_dirs = Vec::new();
    let manifest = parse_manifest(manifest_path)?;

    // Record this project's own src dirs first.
    collect_src_dirs(dir, &manifest, manifest_path, &mut search_dirs)?;

    // Then resolve each dependency's project root and append its src dirs.
    resolve_dependencies(dir, &manifest, &mut visited, &mut search_dirs)?;

    let entry = manifest.entry.as_ref().map(|entry| dir.join(entry));

    Ok(ResolvedProject {
        manifest,
        entry,
        search_dirs,
    })
}

/// Read and JSON-parse a manifest file into a [`ProjectManifest`].
fn parse_manifest(manifest_path: &Path) -> Result<ProjectManifest, Box<DiagnosticReport>> {
    let text = std::fs::read_to_string(manifest_path).map_err(|error| {
        manifest_error(
            format!(
                "failed to read project manifest `{}`: {error}",
                manifest_path.display()
            ),
            manifest_path,
        )
    })?;
    serde_json::from_str::<ProjectManifest>(&text).map_err(|error| {
        manifest_error(
            format!(
                "failed to parse project manifest `{}`: {error}",
                manifest_path.display()
            ),
            manifest_path,
        )
    })
}

/// Append the (validated, existing) `src` directories of a manifest to `out`.
fn collect_src_dirs(
    dir: &Path,
    manifest: &ProjectManifest,
    manifest_path: &Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), Box<DiagnosticReport>> {
    for src in &manifest.src {
        let src_dir = dir.join(src);
        if !src_dir.is_dir() {
            return Err(manifest_error(
                format!(
                    "project `{}` names a `src` directory `{}` that does not exist (resolved to `{}`)",
                    manifest.name,
                    src,
                    src_dir.display()
                ),
                manifest_path,
            ));
        }
        if !out.iter().any(|existing| existing == &src_dir) {
            out.push(src_dir);
        }
    }
    Ok(())
}

/// Resolve every dependency transitively, appending each dependency's `src`
/// directories to `out`. Cycles between projects are harmless (a project already
/// visited is simply skipped).
fn resolve_dependencies(
    dir: &Path,
    manifest: &ProjectManifest,
    visited: &mut HashSet<PathBuf>,
    out: &mut Vec<PathBuf>,
) -> Result<(), Box<DiagnosticReport>> {
    for (dep_name, dep_path) in &manifest.dependencies {
        let dep_root = dir.join(dep_path);
        let dep_manifest_path = dep_root.join(MANIFEST_FILE_NAME);

        if !dep_root.is_dir() {
            return Err(manifest_error(
                format!(
                    "dependency `{dep_name}` of project `{}` points to `{}`, which is not a directory (resolved to `{}`)",
                    manifest.name,
                    dep_path,
                    dep_root.display()
                ),
                &dep_manifest_path,
            ));
        }
        if !dep_manifest_path.is_file() {
            return Err(manifest_error(
                format!(
                    "dependency `{dep_name}` of project `{}` at `{}` has no `{MANIFEST_FILE_NAME}`",
                    manifest.name,
                    dep_root.display()
                ),
                &dep_manifest_path,
            ));
        }

        // Canonicalize where possible so the same project reached by two paths is
        // visited once; fall back to the joined path if canonicalization fails.
        let key = std::fs::canonicalize(&dep_root).unwrap_or_else(|_| dep_root.clone());
        if !visited.insert(key) {
            continue;
        }

        let dep_manifest = parse_manifest(&dep_manifest_path)?;
        collect_src_dirs(&dep_root, &dep_manifest, &dep_manifest_path, out)?;
        resolve_dependencies(&dep_root, &dep_manifest, visited, out)?;
    }
    Ok(())
}
