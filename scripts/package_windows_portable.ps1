param(
    [string]$PackageName = "nous-lang-alpha1-windows-x64",
    [switch]$SkipBuild
)

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")
$PackageRoot = Join-Path $RepoRoot "dist\$PackageName"
$ArchivePath = Join-Path $RepoRoot "dist\$PackageName.zip"

Push-Location $RepoRoot
try {
    if (-not $SkipBuild) {
        cargo build --release -p nous_cli
    }

    $Binary = Join-Path $RepoRoot "target\release\nlang.exe"
    if (-not (Test-Path -LiteralPath $Binary)) {
        throw "release binary not found: $Binary"
    }

    Remove-Item -LiteralPath $PackageRoot -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path (Join-Path $PackageRoot "bin") | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $PackageRoot "docs") | Out-Null
    New-Item -ItemType Directory -Force -Path (Join-Path $PackageRoot "examples") | Out-Null

    Copy-Item -LiteralPath $Binary -Destination (Join-Path $PackageRoot "bin\nlang.exe")
    Copy-Item -LiteralPath (Join-Path $RepoRoot "offline_docs\index.html") -Destination (Join-Path $PackageRoot "docs\index.html")
    Copy-Item -LiteralPath (Join-Path $RepoRoot "tests\fixtures\valid") -Destination (Join-Path $PackageRoot "examples\valid") -Recurse
    Copy-Item -LiteralPath (Join-Path $RepoRoot "documents\alpha1_release_notes.md") -Destination (Join-Path $PackageRoot "RELEASE_NOTES.md")
    Copy-Item -LiteralPath (Join-Path $RepoRoot "scripts\install_windows_path.ps1") -Destination (Join-Path $PackageRoot "install.ps1")
    Copy-Item -LiteralPath (Join-Path $RepoRoot "scripts\uninstall_windows_path.ps1") -Destination (Join-Path $PackageRoot "uninstall.ps1")
    Copy-Item -LiteralPath (Join-Path $RepoRoot "scripts\install.cmd") -Destination (Join-Path $PackageRoot "install.cmd")
    Copy-Item -LiteralPath (Join-Path $RepoRoot "scripts\uninstall.cmd") -Destination (Join-Path $PackageRoot "uninstall.cmd")

    $LicenseStatus = "No repository license file was present when this package was created."
    foreach ($LicenseName in @("LICENSE", "LICENSE.txt", "LICENSE.md", "COPYING", "COPYING.txt")) {
        $LicensePath = Join-Path $RepoRoot $LicenseName
        if (Test-Path -LiteralPath $LicensePath) {
            Copy-Item -LiteralPath $LicensePath -Destination (Join-Path $PackageRoot $LicenseName)
            $LicenseStatus = "License file: $LicenseName"
            break
        }
    }

    $Commit = "unknown"
    try {
        $Commit = (git rev-parse --short HEAD).Trim()
    } catch {
        $Commit = "unknown"
    }

    @"
Nous Lang Alpha 1 portable package
Commit: $Commit
$LicenseStatus

Layout:
- bin\nlang.exe: command-line tool
- docs\index.html: offline documentation
- examples\valid\: executable .nl examples
- RELEASE_NOTES.md: release notes, verification evidence, and known limitations
- install.cmd / install.ps1: optional user PATH setup
- uninstall.cmd / uninstall.ps1: optional user PATH cleanup

Quick start:
1. Open PowerShell in this directory.
2. Run: .\bin\nlang.exe --version
3. Run: .\bin\nlang.exe docs
4. Run: .\bin\nlang.exe examples
5. Run: .\bin\nlang.exe check .\examples\valid\run_arithmetic.nl
6. Run: .\bin\nlang.exe compile --optimize alpha -o .\examples\valid\run_arithmetic.nbc .\examples\valid\run_arithmetic.nl
7. Run: .\bin\nlang.exe inspect .\examples\valid\run_arithmetic.nbc
8. Run: .\bin\nlang.exe run .\examples\valid\run_arithmetic.nbc

Optional PATH setup:
- Run .\install.cmd from this directory to add bin\nlang.exe to your user PATH.
- Open a new shell, then run: nlang --version
- Run .\uninstall.cmd from this directory to remove this package from your user PATH.
"@ | Set-Content -Path (Join-Path $PackageRoot "README.txt") -Encoding UTF8

    @"
package=$PackageName
commit=$Commit
binary=bin\nlang.exe
docs=docs\index.html
release_notes=RELEASE_NOTES.md
installer=install.cmd
uninstaller=uninstall.cmd
license_status=$LicenseStatus
"@ | Set-Content -Path (Join-Path $PackageRoot "VERSION.txt") -Encoding UTF8

    Remove-Item -LiteralPath $ArchivePath -Force -ErrorAction SilentlyContinue
    Compress-Archive -Path (Join-Path $PackageRoot "*") -DestinationPath $ArchivePath -Force

    Write-Output "package: $PackageRoot"
    Write-Output "archive: $ArchivePath"
} finally {
    Pop-Location
}
