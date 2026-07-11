# Versioning

Lullaby releases use a two-number version with a status suffix:

```
MAJOR.PATCH-STATUS        e.g.  1.0-preview,  1.3-stable,  2.0-experimental
```

- **MAJOR** — the language generation. Bumped for a large, intentional shift in
  the language or toolchain identity.
- **PATCH** — increments on every release within a generation (`1.0`, `1.1`,
  `1.2`, …). There is no separate "minor"; a single running counter keeps
  version math simple.
- **STATUS** — the maturity of the build:
  | Status | Meaning |
  | --- | --- |
  | `stable` | Released and supported; the recommended download. |
  | `preview` | Feature-complete for its scope, seeking real-world feedback before it is blessed stable. |
  | `experimental` | Unproven / bleeding-edge work (e.g. a new backend or optimization round) that may change or regress. |

## Mapping to Cargo / semver

Cargo requires three-part semver (`MAJOR.MINOR.PATCH[-prerelease]`), so the
two-number form maps as:

| Display / tag | `Cargo.toml` version |
| --- | --- |
| `1.0-preview` | `1.0.0-preview` |
| `1.3-experimental` | `1.3.0-experimental` |
| `1.0-stable` | `1.0.0` (a `stable` build carries **no** prerelease suffix, so semver ranks it above every `-preview`/`-experimental` of the same number) |

The scheme's PATCH lives in semver's **minor** slot; semver's patch slot is
always `0` filler. `lullaby --version` prints the two-number display form
(`1.0-preview`), reconstructed from `CARGO_PKG_VERSION`.

## Release tags

Git tags and GitHub Releases use `vMAJOR.PATCH-STATUS`, e.g. `v1.0-preview`.
The website's download page and the `curl | sh` / `irm | iex` installers resolve
the newest release from the GitHub API, so publishing a release makes it the
offered download automatically.

## Bumping the version

The version is centralized in the workspace: `[workspace.package] version` in
the root `Cargo.toml`, inherited by every crate via `version.workspace = true`.
The standalone `crates/lullaby_wasm` (excluded from the workspace) carries the
version literally and is updated in the same commit.

### MSI ProductVersion (upgrade encoding)

The WiX installer needs a numeric `MAJOR.MINOR.PATCH` `ProductVersion` with no
suffix, and Windows Installer ranks upgrades by that number alone. Because the
scheme parks semver's patch slot at `0` for every build, `1.0-preview` and
`1.0-stable` would otherwise both map to `1.0.0` and a stable MSI would refuse to
replace an installed preview. So `scripts/build_windows_installer.py` encodes the
**status** into the MSI's PATCH field (`experimental`=10, `preview`=20,
`stable`=30): `1.0-preview` → ProductVersion `1.0.20`, `1.0-stable` → `1.0.30`.
Each successive release of the same `MAJOR.PATCH` then strictly increases, so a
`<MajorUpgrade>` replaces the prior install in place. This affects only the MSI
ProductVersion — the display/tag form and Cargo's semver are untouched.
