#!/usr/bin/env pwsh
# Test all views and generate snapshots

param(
    [string]$BuildMode = "release"
)

Write-Host "====================================" -ForegroundColor Cyan
Write-Host "  Beads-TUI View Testing Suite" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host ""

$views = 0..10
$sizes = @("80x24", "120x40", "160x50")
$datasets = @("small", "medium")

# Create output directory
New-Item -ItemType Directory -Force -Path test_output | Out-Null
Write-Host "Created output directory: test_output/" -ForegroundColor Green
Write-Host ""

# Build flag
$buildFlag = if ($BuildMode -eq "release") { "--release" } else { "" }
$binPath = if ($BuildMode -eq "release") { "target\release\beads-tui.exe" } else { "target\debug\beads-tui.exe" }

Write-Host "Using build mode: $BuildMode" -ForegroundColor Cyan
Write-Host "Binary path: $binPath" -ForegroundColor Cyan
Write-Host ""

# Check if binary exists
if (-not (Test-Path $binPath)) {
    Write-Host "Binary not found at $binPath" -ForegroundColor Red
    Write-Host "Building project..." -ForegroundColor Yellow
    cargo build $buildFlag
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        exit 1
    }
}

Write-Host "Starting view tests..." -ForegroundColor Cyan
Write-Host ""

$totalTests = $views.Length * $sizes.Length * $datasets.Length
$currentTest = 0
$successCount = 0
$failCount = 0

foreach ($dataset in $datasets) {
    Write-Host "Testing dataset: $dataset" -ForegroundColor Magenta
    Write-Host ("-" * 50) -ForegroundColor DarkGray

    foreach ($view in $views) {
        foreach ($size in $sizes) {
            $currentTest++
            $output = "test_output\view_${view}_${dataset}_${size}.txt"

            Write-Progress -Activity "Testing Views" `
                -Status "View $view | Dataset: $dataset | Size: $size" `
                -PercentComplete (($currentTest / $totalTests) * 100)

            Write-Host "  [$currentTest/$totalTests] View $view @ $size " -NoNewline

            & $binPath --demo --dataset $dataset --view $view --snapshot --size $size --output $output 2>&1 | Out-Null

            if ($LASTEXITCODE -eq 0) {
                Write-Host "[OK]" -ForegroundColor Green
                $successCount++
            } else {
                Write-Host "[FAILED]" -ForegroundColor Red
                $failCount++
            }
        }
    }
    Write-Host ""
}

Write-Progress -Activity "Testing Views" -Completed

Write-Host ""
Write-Host "====================================" -ForegroundColor Cyan
Write-Host "  Test Summary" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host "Total tests:    $totalTests" -ForegroundColor White
Write-Host "Successful:     $successCount" -ForegroundColor Green
Write-Host "Failed:         $failCount" -ForegroundColor $(if ($failCount -eq 0) { "Green" } else { "Red" })
Write-Host "Output directory: test_output\" -ForegroundColor Cyan
Write-Host ""

# List generated files
$files = Get-ChildItem -Path test_output -Filter "*.txt" | Measure-Object
Write-Host "Generated $($files.Count) snapshot files" -ForegroundColor Cyan

if ($failCount -eq 0) {
    Write-Host ""
    Write-Host "All tests passed!" -ForegroundColor Green
    exit 0
} else {
    Write-Host ""
    Write-Host "Some tests failed. Check the output above for details." -ForegroundColor Red
    exit 1
}
