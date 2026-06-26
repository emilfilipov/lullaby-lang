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
$InstallScript = Join-Path $PackageRoot "install.ps1"
$UninstallScript = Join-Path $PackageRoot "uninstall.ps1"

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
    if (-not (Test-Path -LiteralPath $InstallScript)) {
        throw "packaged install.ps1 not found"
    }
    if (-not (Test-Path -LiteralPath $UninstallScript)) {
        throw "packaged uninstall.ps1 not found"
    }
    if (-not (Test-Path -LiteralPath (Join-Path $PackageRoot "install.cmd"))) {
        throw "packaged install.cmd not found"
    }
    if (-not (Test-Path -LiteralPath (Join-Path $PackageRoot "uninstall.cmd"))) {
        throw "packaged uninstall.cmd not found"
    }

    & $Nlang --version
    & $Nlang docs
    & $Nlang examples
    & $Nlang check $Example
    & $Nlang run $Example
    Remove-Item -LiteralPath $Artifact -Force -ErrorAction SilentlyContinue
    & $Nlang compile --optimize alpha -o $Artifact $Example
    & $Nlang inspect $Artifact
    & $Nlang run $Artifact
    powershell -ExecutionPolicy Bypass -File $InstallScript -DryRun
    powershell -ExecutionPolicy Bypass -File $UninstallScript -DryRun

    Write-Output "release verification passed: $PackageRoot"
} finally {
    Pop-Location
}
