# Whole-corpus performance harness. Times every corpus category's program across
# all execution tiers, giving a BROAD picture (26 diverse programs) rather than
# the three micro-benchmarks (fib/loop/primes) that measure pure compute.
#
# Metric: whole-program wall time (parse + execute), best-of-N per tier. Small
# programs include a fixed parse/launch cost, so this measures small-program
# throughput and the interpreter-vs-native gap, not isolated compute — the
# fib/loop/primes harnesses remain for compute-only numbers.
[CmdletBinding()]
param([int]$Reps = 7)
# Native emit fails (L0339) for categories that use heap/strings — expected, not
# fatal — so keep going on errors and gate native timing on the .exe existing.
$ErrorActionPreference = 'Continue'
$cross = $PSScriptRoot
$repo = Split-Path (Split-Path $cross)
$lb = Join-Path $repo 'target\release\lullaby.exe'
if (-not (Test-Path $lb)) { throw "build release first: cargo build --release -p lullaby_cli" }

# MSVC env so native-eligible categories can link.
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
cmd /c "`"$vsPath\VC\Auxiliary\Build\vcvars64.bat`" >nul 2>&1 && set" | ForEach-Object {
    if ($_ -match '^(.*?)=(.*)$') { [Environment]::SetEnvironmentVariable($matches[1], $matches[2], 'Process') }
}

function Best([scriptblock]$run, [int]$reps) {
    $min = [double]::MaxValue
    for ($i = 0; $i -lt $reps; $i++) {
        $sw = [System.Diagnostics.Stopwatch]::StartNew(); & $run 2>&1 | Out-Null; $sw.Stop()
        if ($sw.Elapsed.TotalMilliseconds -lt $min) { $min = $sw.Elapsed.TotalMilliseconds }
    }
    $min
}

# Measure the fixed parse+launch floor (an empty program) so we can subtract it
# and show execution-only time. Small corpus programs are dominated by this floor.
$floorLby = Join-Path $env:TEMP 'corpus_perf_floor.lby'
"fn main -> i64`n    0" | Set-Content -Encoding ASCII $floorLby
$floor = Best { & $lb run --backend ast $floorLby } $Reps
Write-Host ("`nparse+launch floor (empty program): {0:N1} ms (subtracted below as exec~)`n" -f $floor)

$cats = Get-ChildItem (Join-Path $cross 'corpus') -Directory | Sort-Object Name
$rows = @()
$agg = @{ ast = 0.0; ir = 0.0; bytecode = 0.0; native = 0.0; nativeAst = 0.0 }
$nativeCount = 0

Write-Host "`n=== whole-corpus perf: parse+execute wall time, best-of-$Reps, per tier (ms) ===`n"
Write-Host ("{0,-22} {1,8} {2,8} {3,8} {4,10}" -f "category", "ast", "ir", "bytecode", "native")
Write-Host ("-" * 60)

foreach ($cat in $cats) {
    $lby = Join-Path $cat.FullName 'lullaby.lby'
    if (-not (Test-Path $lby)) { continue }
    $ast = Best { & $lb run --backend ast $lby } $Reps
    $ir = Best { & $lb run --backend ir $lby } $Reps
    $bc = Best { & $lb run --backend bytecode $lby } $Reps
    $agg.ast += $ast; $agg.ir += $ir; $agg.bytecode += $bc

    # native: emit + run only if the whole program is native-eligible.
    $exe = Join-Path $cat.FullName '_corpus_perf.exe'
    if (Test-Path $exe) { Remove-Item $exe -ErrorAction SilentlyContinue }
    & $lb native -o $exe $lby *> $null
    $nativeStr = "n/a"
    if (Test-Path $exe) {
        $nat = Best { & $exe } $Reps
        $agg.native += $nat; $agg.nativeAst += $ast; $nativeCount++
        $nativeStr = ("{0:N1}" -f $nat)
        Remove-Item $exe -ErrorAction SilentlyContinue
    }
    $exec = [Math]::Max(0, $ast - $floor)
    Write-Host ("{0,-22} {1,8:N1} {2,8:N1} {3,8:N1} {4,10}   exec~{5:N1}" -f $cat.Name, $ast, $ir, $bc, $nativeStr, $exec)
}

Write-Host ("-" * 60)
Write-Host ("{0,-22} {1,8:N1} {2,8:N1} {3,8:N1}" -f "TOTAL (all 26)", $agg.ast, $agg.ir, $agg.bytecode)
Write-Host ""
Write-Host ("Interpreter totals vs each other: ast {0:N0} ms, ir {1:N0} ms, bytecode {2:N0} ms" -f $agg.ast, $agg.ir, $agg.bytecode)
if ($nativeCount -gt 0) {
    Write-Host ("Native-eligible categories: {0}/{1}. On those, native {2:N0} ms vs ast {3:N0} ms = {4:N1}x" -f `
        $nativeCount, $cats.Count, $agg.native, $agg.nativeAst, ($agg.nativeAst / $agg.native))
}
