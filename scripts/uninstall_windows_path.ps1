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

if (-not (Test-Path -LiteralPath $BinPath)) {
    throw "bin directory not found at $BinPath. Run this script from the root of the Nous Lang portable package."
}

$ResolvedBinPath = (Resolve-Path -LiteralPath $BinPath).ProviderPath
$TargetPath = Normalize-PathValue $ResolvedBinPath
$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ([string]::IsNullOrWhiteSpace($CurrentPath)) {
    $Parts = @()
} else {
    $Parts = $CurrentPath -split ";" | ForEach-Object { $_.Trim() } | Where-Object { $_.Length -gt 0 }
}

$Kept = @()
$Removed = 0
foreach ($Part in $Parts) {
    if ((Normalize-PathValue $Part) -ieq $TargetPath) {
        $Removed += 1
    } else {
        $Kept += $Part
    }
}

if ($Removed -eq 0) {
    Write-Output "Nous Lang bin directory was not present in the user PATH: $ResolvedBinPath"
    exit 0
}

$NewPath = $Kept -join ";"
if ($DryRun) {
    Write-Output "dry-run: would remove from user PATH: $ResolvedBinPath"
    exit 0
}

[Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
Write-Output "Removed Nous Lang from the user PATH: $ResolvedBinPath"
Write-Output "Open a new PowerShell or cmd window for the PATH change to take effect."
