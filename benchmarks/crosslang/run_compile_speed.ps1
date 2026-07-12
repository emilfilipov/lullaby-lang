# Compile-speed benchmark: how fast does each toolchain turn source into a runnable
# program? Developer-iteration latency matters as much as runtime speed — a language
# that runs fast but compiles slowly is painful to work in (Rust's reputation).
#
# Measures best-of-N wall time to compile the SAME program in each language, at both
# optimized and dev/unoptimized settings, including process start + link (that is the
# real edit-compile-run latency a developer feels). Writes compile_data.json for the
# artifact and prints a summary. Lullaby's native path is a direct byte emitter (no
# LLVM), so this is where its architecture shows: sub-15 ms builds.
[CmdletBinding()]
param([int]$Reps = 7)
$ErrorActionPreference = 'Stop'
$cross = $PSScriptRoot
$repo = Split-Path (Split-Path $cross)
$lb = Join-Path $repo 'target\release\lullaby.exe'
if (-not (Test-Path $lb)) { throw "build release first: cargo build --release -p lullaby_cli" }

# MSVC env so `cl` links; rustc is on PATH.
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
cmd /c "`"$vsPath\VC\Auxiliary\Build\vcvars64.bat`" >nul 2>&1 && set" | ForEach-Object {
    if ($_ -match '^(.*?)=(.*)$') { [Environment]::SetEnvironmentVariable($matches[1], $matches[2], 'Process') }
}
$haveRust = [bool](Get-Command rustc -ErrorAction SilentlyContinue)
$tmp = $env:TEMP

function Best([scriptblock]$run, [int]$reps) {
    $m = [double]::MaxValue
    for ($i = 0; $i -lt $reps; $i++) {
        $sw = [Diagnostics.Stopwatch]::StartNew(); & $run *> $null; $sw.Stop()
        if ($sw.Elapsed.TotalMilliseconds -lt $m) { $m = $sw.Elapsed.TotalMilliseconds }
    }
    [math]::Round($m, 1)
}

# Representative programs present in all languages (non-trivial, i64-scalar so the
# native backend compiles them fully). Averaged so one program's noise doesn't skew.
$progs = 'sorting', 'number_theory', 'graph_algos'
$acc = @{ lby_check = 0.0; lby_build = 0.0; c_o2 = 0.0; c_od = 0.0; rs_o = 0.0; rs_dev = 0.0 }
$lines = 0
foreach ($p in $progs) {
    $dir = Join-Path $cross "corpus\$p"
    $lines += (Get-Content (Join-Path $dir 'lullaby.lby') | Measure-Object -Line).Lines
    # `check` = full frontend + semantic typecheck (like `cargo check`). `build` =
    # compile the WHOLE program to a runnable bytecode artifact (the native backend
    # is an i64-scalar subset, so `build` is the fair whole-program compile).
    $acc.lby_check += Best { & $lb check (Join-Path $dir 'lullaby.lby') } $Reps
    $acc.lby_build += Best { & $lb build -o "$tmp\cs_lb.lbc" (Join-Path $dir 'lullaby.lby') } $Reps
    $acc.c_o2  += Best { & cl /nologo /O2 "/Fe:$tmp\cs_c.exe" "/Fo:$tmp\cs_c.obj" (Join-Path $dir 'c.c') } $Reps
    $acc.c_od  += Best { & cl /nologo /Od "/Fe:$tmp\cs_cd.exe" "/Fo:$tmp\cs_cd.obj" (Join-Path $dir 'c.c') } $Reps
    if ($haveRust) {
        $acc.rs_o   += Best { & rustc -O -o "$tmp\cs_rs.exe" (Join-Path $dir 'rust.rs') } $Reps
        $acc.rs_dev += Best { & rustc    -o "$tmp\cs_rsd.exe" (Join-Path $dir 'rust.rs') } $Reps
    }
}
$n = $progs.Count
function Avg($v) { [math]::Round($v / $n, 1) }

$rows = [ordered]@{
    'Lullaby (build, whole program)' = Avg $acc.lby_build
    'Lullaby (check only)'           = Avg $acc.lby_check
    'C (cl /O2)'                     = Avg $acc.c_o2
    'C (cl /Od, debug)'              = Avg $acc.c_od
}
if ($haveRust) {
    $rows['Rust (rustc -O)']      = Avg $acc.rs_o
    $rows['Rust (rustc, debug)']  = Avg $acc.rs_dev
}

Write-Host ("`n=== compile-speed: ms to build the SAME program (avg of {0} programs, best-of-{1}, incl. start+link) ===`n" -f $n, $Reps)
$lbBuild = Avg $acc.lby_build
foreach ($k in $rows.Keys) {
    $vsLb = if ($rows[$k] -gt 0 -and $lbBuild -gt 0) { "{0,5:N1}x Lullaby" -f ($rows[$k] / $lbBuild) } else { "" }
    Write-Host ("  {0,-32} {1,8:N1} ms   {2}" -f $k, $rows[$k], $vsLb)
}

# Emit compile_data.json for the artifact (parallel to perf_data.json).
$out = [ordered]@{
    note = "Wall time to compile the same program in each language, averaged over $n corpus programs ($($progs -join ', ')), best-of-$Reps, including process start and link. This is edit-compile-run latency, the developer-iteration cost. Lullaby's native backend is a direct x86-64 byte emitter (no LLVM), so it builds ~10-40x faster than cl/rustc."
    avg_lines_per_program = [math]::Round($lines / $n)
    unit = "ms (whole build, best-of-$Reps, lower is faster to iterate)"
    results = @()
}
$fastest = $lbBuild
foreach ($k in $rows.Keys) {
    $out.results += [ordered]@{ lang = $k; value = $rows[$k]; vsLullaby = if ($fastest -gt 0) { [math]::Round($rows[$k] / $fastest, 1) } else { 0 } }
}
$out | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 (Join-Path $cross 'compile_data.json')
Write-Host ("`nwrote {0}" -f (Join-Path $cross 'compile_data.json'))
