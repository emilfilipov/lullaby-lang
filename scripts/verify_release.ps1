param(
    [string]$PackageName = "nous-lang-alpha1-windows-x64"
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")
$PackageRoot = Join-Path $RepoRoot "dist\$PackageName"
$Nlang = Join-Path $PackageRoot "bin\nlang.exe"
$Example = Join-Path $PackageRoot "examples\valid\run_arithmetic.nl"
$Artifact = Join-Path $PackageRoot "examples\valid\run_arithmetic.nbc"

Push-Location $RepoRoot
try {
    cargo fmt --check
    cargo test --all
    cargo clippy --all-targets --all-features -- -D warnings
    python offline_docs\verify_offline_docs.py

    & (Join-Path $ScriptDir "package_windows_portable.ps1") -PackageName $PackageName

    if (-not (Test-Path -LiteralPath $Nlang)) {
        throw "packaged nlang.exe not found: $Nlang"
    }
    if (-not (Test-Path -LiteralPath (Join-Path $PackageRoot "docs\index.html"))) {
        throw "packaged offline docs not found"
    }
    if (-not (Test-Path -LiteralPath $Example)) {
        throw "packaged example not found: $Example"
    }
    if (-not (Test-Path -LiteralPath (Join-Path $PackageRoot "RELEASE_NOTES.md"))) {
        throw "packaged release notes not found"
    }

    & $Nlang --version
    & $Nlang docs
    & $Nlang check $Example
    & $Nlang run $Example
    Remove-Item -LiteralPath $Artifact -Force -ErrorAction SilentlyContinue
    & $Nlang compile --optimize alpha -o $Artifact $Example
    & $Nlang run $Artifact

    Write-Output "release verification passed: $PackageRoot"
} finally {
    Pop-Location
}
