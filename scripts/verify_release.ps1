param(
    [string]$PackageName = "nous-lang-alpha1-windows-x64"
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")
$PackageRoot = Join-Path $RepoRoot "dist\$PackageName"
$ArchivePath = Join-Path $RepoRoot "dist\$PackageName.zip"
$ChecksumPath = "$ArchivePath.sha256"
$Nlang = Join-Path $PackageRoot "bin\nlang.exe"
$Example = Join-Path $PackageRoot "examples\valid\calculator.nl"
$InvalidExample = Join-Path $PackageRoot "examples\invalid\type_mismatch.nl"
$Artifact = Join-Path $PackageRoot "examples\valid\calculator.nbc"
$InstallScript = Join-Path $PackageRoot "install.ps1"
$UninstallScript = Join-Path $PackageRoot "uninstall.ps1"

Push-Location $RepoRoot
try {
    cargo fmt --check
    if ($LASTEXITCODE -ne 0) { throw "cargo fmt --check failed" }
    cargo test --all
    if ($LASTEXITCODE -ne 0) { throw "cargo test --all failed" }
    cargo clippy --all-targets --all-features -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw "cargo clippy failed" }
    python offline_docs\verify_offline_docs.py
    if ($LASTEXITCODE -ne 0) { throw "offline docs verification failed" }
    & (Join-Path $ScriptDir "verify_markdown_refs.ps1")
    if ($LASTEXITCODE -ne 0) { throw "markdown reference verification failed" }

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
    if (-not (Test-Path -LiteralPath $InvalidExample)) {
        throw "packaged invalid example not found: $InvalidExample"
    }
    if (-not (Test-Path -LiteralPath (Join-Path $PackageRoot "RELEASE_NOTES.md"))) {
        throw "packaged release notes not found"
    }
    if (-not (Test-Path -LiteralPath $ArchivePath)) {
        throw "package archive not found: $ArchivePath"
    }
    if (-not (Test-Path -LiteralPath $ChecksumPath)) {
        throw "package checksum not found: $ChecksumPath"
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
    if ($LASTEXITCODE -ne 0) { throw "nlang --version failed" }
    & $Nlang docs
    if ($LASTEXITCODE -ne 0) { throw "nlang docs failed" }
    & $Nlang examples
    if ($LASTEXITCODE -ne 0) { throw "nlang examples failed" }
    & $Nlang check $Example
    if ($LASTEXITCODE -ne 0) { throw "nlang check failed: $Example" }
    & $Nlang run $Example
    if ($LASTEXITCODE -ne 0) { throw "nlang run failed: $Example" }
    & $Nlang check $InvalidExample
    if ($LASTEXITCODE -eq 0) {
        throw "invalid example unexpectedly passed check: $InvalidExample"
    }
    Remove-Item -LiteralPath $Artifact -Force -ErrorAction SilentlyContinue
    & $Nlang compile --optimize alpha -o $Artifact $Example
    if ($LASTEXITCODE -ne 0) { throw "nlang compile failed: $Example" }
    & $Nlang inspect $Artifact
    if ($LASTEXITCODE -ne 0) { throw "nlang inspect failed: $Artifact" }
    & $Nlang run $Artifact
    if ($LASTEXITCODE -ne 0) { throw "nlang run failed: $Artifact" }
    powershell -ExecutionPolicy Bypass -File $InstallScript -DryRun
    if ($LASTEXITCODE -ne 0) { throw "install.ps1 dry-run failed" }
    powershell -ExecutionPolicy Bypass -File $UninstallScript -DryRun
    if ($LASTEXITCODE -ne 0) { throw "uninstall.ps1 dry-run failed" }

    $ExpectedChecksum = (Get-FileHash -LiteralPath $ArchivePath -Algorithm SHA256).Hash.ToLowerInvariant()
    $ChecksumText = (Get-Content -LiteralPath $ChecksumPath -Raw).Trim()
    if ($ChecksumText -ne "$ExpectedChecksum  $PackageName.zip") {
        throw "checksum mismatch in $ChecksumPath"
    }

    Write-Output "release verification passed: $PackageRoot"
} finally {
    Pop-Location
}
