#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Validate test fixtures for beads-tui.
#>

param(
    [Parameter()]
    [switch]$ShowVerbose,

    [Parameter()]
    [switch]$NoCapture
)

$cargoArgs = @("test", "--test", "validate_fixtures")

if ($ShowVerbose) {
    $cargoArgs += "--verbose"
}

if ($NoCapture) {
    $cargoArgs += "--", "--nocapture"
}

Write-Host "Running: cargo $($cargoArgs -join ' ')" -ForegroundColor Cyan
& cargo @cargoArgs
exit $LASTEXITCODE
