#!/usr/bin/env pwsh
# Launch interactive tour of all views

param(
    [string]$BuildMode = "release",
    [int]$Duration = 2
)

# Build flag
$buildFlag = if ($BuildMode -eq "release") { "--release" } else { "" }
$binPath = if ($BuildMode -eq "release") { "target\release\beads-tui.exe" } else { "target\debug\beads-tui.exe" }

# Check if binary exists
if (-not (Test-Path $binPath)) {
    Write-Host "Building project..." -ForegroundColor Cyan
    cargo build $buildFlag
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        exit 1
    }
}

Write-Host "Starting view tour ($Duration seconds per view)" -ForegroundColor Cyan
Write-Host "Press 'q' to exit early" -ForegroundColor Green
Write-Host ""

& $binPath --demo --test-all-views --test-duration $Duration
