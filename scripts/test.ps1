#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Cross-platform test runner for beads-tui

.DESCRIPTION
    Wraps cargo test to run different test suites (unit, integration, snapshot, property)
    with support for test fixtures and snapshot updates.

.PARAMETER Suite
    Test suite to run: unit, integration, snapshot, property, or all (default: all)

.PARAMETER Fixture
    Test fixture to use (sets BD_DB environment variable)

.PARAMETER UpdateSnapshots
    Enable snapshot update mode for UI tests

.PARAMETER Verbose
    Show detailed test output

.PARAMETER NoCap ture
    Don't capture test output (shows println! immediately)

.EXAMPLE
    .\test.ps1
    Run all tests

.EXAMPLE
    .\test.ps1 -Suite unit
    Run only unit tests

.EXAMPLE
    .\test.ps1 -Suite snapshot -UpdateSnapshots
    Run snapshot tests and update snapshots

.EXAMPLE
    .\test.ps1 -Fixture test-data-1
    Run tests with specific fixture
#>

param(
    [Parameter()]
    [ValidateSet("all", "unit", "integration", "snapshot", "property")]
    [string]$Suite = "all",

    [Parameter()]
    [string]$Fixture = "",

    [Parameter()]
    [switch]$UpdateSnapshots,

    [Parameter()]
    [switch]$ShowVerbose,

    [Parameter()]
    [switch]$NoCapture
)

# Color output functions
function Write-Info {
    param([string]$Message)
    Write-Host "→ $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "✓ $Message" -ForegroundColor Green
}

function Write-Error {
    param([string]$Message)
    Write-Host "✗ $Message" -ForegroundColor Red
}

function Write-Section {
    param([string]$Title)
    Write-Host ""
    Write-Host "═══════════════════════════════════════════════" -ForegroundColor Blue
    Write-Host "  $Title" -ForegroundColor Blue
    Write-Host "═══════════════════════════════════════════════" -ForegroundColor Blue
    Write-Host ""
}

# Set environment variables
if ($Fixture) {
    Write-Info "Using test fixture: $Fixture"
    $env:BD_DB = $Fixture
}

if ($UpdateSnapshots) {
    Write-Info "Snapshot update mode enabled"
    $env:UPDATE_SNAPSHOTS = "1"
    $env:INSTA_UPDATE = "always"
}

# Build base cargo command
$cargoArgs = @("test")

if ($ShowVerbose) {
    $cargoArgs += "--verbose"
}

if ($NoCapture) {
    $cargoArgs += "--", "--nocapture"
}

# Track test results
$exitCode = 0

# Run requested test suites
switch ($Suite) {
    "unit" {
        Write-Section "Running Unit Tests"
        $unitArgs = $cargoArgs + @("--lib")
        Write-Info "Command: cargo $($unitArgs -join ' ')"
        & cargo @unitArgs
        if ($LASTEXITCODE -ne 0) { $exitCode = 1 }
    }

    "integration" {
        Write-Section "Running Integration Tests"
        $integrationArgs = $cargoArgs + @("--test", "*")
        Write-Info "Command: cargo $($integrationArgs -join ' ')"
        & cargo @integrationArgs
        if ($LASTEXITCODE -ne 0) { $exitCode = 1 }
    }

    "snapshot" {
        Write-Section "Running Snapshot Tests"
        $snapshotArgs = $cargoArgs + @("--test", "integration", "ui_snapshots")
        Write-Info "Command: cargo $($snapshotArgs -join ' ')"
        & cargo @snapshotArgs
        if ($LASTEXITCODE -ne 0) { $exitCode = 1 }
    }

    "property" {
        Write-Section "Running Property-Based Tests"
        $propArgs = $cargoArgs + @("--lib", "proptest")
        Write-Info "Command: cargo $($propArgs -join ' ')"
        & cargo @propArgs
        if ($LASTEXITCODE -ne 0) { $exitCode = 1 }
    }

    "all" {
        # Run all test types
        Write-Section "Running All Tests"

        # Unit tests
        Write-Info "1/4 - Unit tests"
        $unitArgs = $cargoArgs + @("--lib")
        & cargo @unitArgs
        if ($LASTEXITCODE -ne 0) { $exitCode = 1 }

        Write-Host ""

        # Integration tests
        Write-Info "2/4 - Integration tests"
        $integrationArgs = $cargoArgs + @("--test", "*")
        & cargo @integrationArgs
        if ($LASTEXITCODE -ne 0) { $exitCode = 1 }

        Write-Host ""

        # Snapshot tests
        Write-Info "3/4 - Snapshot tests"
        $snapshotArgs = $cargoArgs + @("--test", "integration", "ui_snapshots")
        & cargo @snapshotArgs
        if ($LASTEXITCODE -ne 0) { $exitCode = 1 }

        Write-Host ""

        # Property tests
        Write-Info "4/4 - Property-based tests"
        $propArgs = $cargoArgs + @("--lib", "proptest")
        & cargo @propArgs
        if ($LASTEXITCODE -ne 0) { $exitCode = 1 }
    }
}

# Clean up environment variables
if ($Fixture) {
    Remove-Item Env:\BD_DB -ErrorAction SilentlyContinue
}
if ($UpdateSnapshots) {
    Remove-Item Env:\UPDATE_SNAPSHOTS -ErrorAction SilentlyContinue
    Remove-Item Env:\INSTA_UPDATE -ErrorAction SilentlyContinue
}

# Summary
Write-Host ""
if ($exitCode -eq 0) {
    Write-Success "All tests passed!"
} else {
    Write-Error "Some tests failed"
}

exit $exitCode
