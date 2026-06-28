param(
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

function Normalize-PathValue {
    param([string]$PathValue)

    $trimmed = $PathValue.Trim().TrimEnd("\")
    if ($trimmed.Length -eq 0) {
        return ""
    }

    try {
        return (Resolve-Path -LiteralPath $trimmed -ErrorAction Stop).ProviderPath.TrimEnd("\")
    } catch {
        return $trimmed
    }
}

$PackageRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$BinPath = Join-Path $PackageRoot "bin"
$LullabyExe = Join-Path $BinPath "lullaby.exe"

if (-not (Test-Path -LiteralPath $LullabyExe)) {
    throw "lullaby.exe not found at $LullabyExe. Run this script from the root of the Lullaby portable package."
}

$ResolvedBinPath = (Resolve-Path -LiteralPath $BinPath).ProviderPath
$TargetPath = Normalize-PathValue $ResolvedBinPath
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ([string]::IsNullOrWhiteSpace($CurrentPath)) {
    $Parts = @()
} else {
    $Parts = $CurrentPath -split ";" | ForEach-Object { $_.Trim() } | Where-Object { $_.Length -gt 0 }
}

$AlreadyPresent = $false
foreach ($Part in $Parts) {
    if ((Normalize-PathValue $Part) -ieq $TargetPath) {
        $AlreadyPresent = $true
        break
    }
}

if ($AlreadyPresent) {
    Write-Output "Lullaby bin directory is already in the user PATH: $ResolvedBinPath"
    exit 0
}

$NewPath = (@($Parts) + $ResolvedBinPath) -join ";"
if ($DryRun) {
    Write-Output "dry-run: would add to user PATH: $ResolvedBinPath"
    exit 0
}

[Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
Write-Output "Added Lullaby to the user PATH: $ResolvedBinPath"
Write-Output "Open a new PowerShell or cmd window, then run: lullaby --version"
