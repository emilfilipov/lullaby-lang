# One-command cross-language benchmark runner.
#
# Regenerates the whole benchmark artifact from the corpus:
#   1. tokenize the corpus (all six languages)  -> corpus_data.json
#   2. optionally run the performance harness    -> perf_data.json     (-Perf)
#   3. optionally re-measure compile speed        -> compile_data.json  (-Perf)
#   4. assemble the self-contained HTML report   -> report.html
#
# Run this whenever the corpus, the language, or the optimizer changes, so the
# published artifact stays in sync with reality. See README.md for the workflow.
#
#   pwsh benchmarks/crosslang/run_benchmark.ps1            # tokens + assemble
#   pwsh benchmarks/crosslang/run_benchmark.ps1 -Perf      # also re-run perf + compile speed
#
[CmdletBinding()]
param(
    [switch]$Perf,
    [int]$Reps = 5,
    [int]$PyReps = 1
)
$ErrorActionPreference = 'Stop'
$cross = $PSScriptRoot

# tiktoken lives in the standalone Python 3.14, NOT the shell venv.
$tikPy = 'C:\Users\emil\AppData\Local\Programs\Python\Python314\python.exe'
if (-not (Test-Path $tikPy)) { throw "tiktoken Python not found at $tikPy" }

Write-Host '=== [1/3] tokenizing corpus (6 languages) ===' -ForegroundColor Cyan
& $tikPy (Join-Path $cross 'corpus_tokens.py')
if ($LASTEXITCODE -ne 0) { throw 'corpus_tokens.py failed' }

if ($Perf) {
    Write-Host "`n=== [2/3] running performance + compile-speed harnesses ===" -ForegroundColor Cyan
    & (Join-Path $cross 'run_perf.ps1') -Reps $Reps -PyReps $PyReps
    & (Join-Path $cross 'run_compile_speed.ps1') -Reps 7
} else {
    Write-Host "`n=== [2/3] skipping perf + compile speed (pass -Perf to re-run) ===" -ForegroundColor DarkGray
}

Write-Host "`n=== [3/3] assembling report.html ===" -ForegroundColor Cyan
$out = Join-Path $cross 'report.html'
& $tikPy (Join-Path $cross 'assemble_report.py') (Join-Path $cross 'report_template.html') $out
if ($LASTEXITCODE -ne 0) { throw 'assemble_report.py failed' }

Write-Host "`nDone. Open $out or publish it as the benchmark artifact." -ForegroundColor Green
