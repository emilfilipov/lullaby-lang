param(
    [string]$TagName = "v0.1.0-alpha.2",
    [string]$PackageName = "nous-lang-alpha1-windows-x64",
    [string]$ReleaseTitle = "Nous Lang v0.1.0-alpha.2",
    [switch]$Draft
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RepoRoot = Resolve-Path (Join-Path $ScriptDir "..")
$ArchivePath = Join-Path $RepoRoot "dist\$PackageName.zip"
$ChecksumPath = "$ArchivePath.sha256"
$ReleaseNotes = Join-Path $RepoRoot "documents\alpha1_release_notes.md"

Push-Location $RepoRoot
try {
    gh auth status | Out-Host
    if ($LASTEXITCODE -ne 0) {
        throw "gh authentication check failed"
    }

    $Status = (git status --porcelain)
    if ($Status) {
        throw "working tree is not clean; commit or discard changes before publishing a release"
    }

    & (Join-Path $ScriptDir "verify_release.ps1") -PackageName $PackageName
    if ($LASTEXITCODE -ne 0) {
        throw "release verification failed"
    }

    if (-not (Test-Path -LiteralPath $ArchivePath)) {
        throw "package archive not found: $ArchivePath"
    }
    if (-not (Test-Path -LiteralPath $ChecksumPath)) {
        throw "package checksum not found: $ChecksumPath"
    }
    if (-not (Test-Path -LiteralPath $ReleaseNotes)) {
        throw "release notes not found: $ReleaseNotes"
    }

    git rev-parse --verify "refs/tags/$TagName" *> $null
    if ($LASTEXITCODE -ne 0) {
        git tag $TagName
        if ($LASTEXITCODE -ne 0) { throw "failed to create tag $TagName" }
        git push origin $TagName
        if ($LASTEXITCODE -ne 0) { throw "failed to push tag $TagName" }
    } else {
        git ls-remote --exit-code --tags origin $TagName *> $null
        if ($LASTEXITCODE -ne 0) {
            git push origin $TagName
            if ($LASTEXITCODE -ne 0) { throw "failed to push tag $TagName" }
        }
    }

    gh release view $TagName *> $null
    if ($LASTEXITCODE -eq 0) {
        throw "GitHub release already exists for $TagName"
    }

    $ReleaseArgs = @(
        "release", "create", $TagName,
        $ArchivePath,
        $ChecksumPath,
        "--title", $ReleaseTitle,
        "--notes-file", $ReleaseNotes,
        "--prerelease"
    )
    if ($Draft) {
        $ReleaseArgs += "--draft"
    }

    gh @ReleaseArgs
    if ($LASTEXITCODE -ne 0) {
        throw "gh release create failed"
    }
} finally {
    Pop-Location
}
