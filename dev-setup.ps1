#!/usr/bin/env pwsh
# Development environment setup script for beads-tui

Write-Host "=== Beads-TUI Development Setup ===" -ForegroundColor Cyan
Write-Host ""

# Check prerequisites
Write-Host "Checking prerequisites..." -ForegroundColor Yellow

# Check Rust
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    $rustVersion = cargo --version
    Write-Host "[OK] $rustVersion" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Cargo not found. Install from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

# Check Git
if (Get-Command git -ErrorAction SilentlyContinue) {
    $gitVersion = git --version
    Write-Host "[OK] $gitVersion" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Git not found. Install from https://git-scm.com/" -ForegroundColor Red
    exit 1
}

# Check Beads
if (Get-Command bd -ErrorAction SilentlyContinue) {
    Write-Host "[OK] Beads CLI found" -ForegroundColor Green
} else {
    Write-Host "[WARNING] Beads CLI not found in PATH" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Installing cargo extensions..." -ForegroundColor Yellow

# Install recommended cargo extensions
$extensions = @(
    "cargo-watch",
    "cargo-audit",
    "cargo-outdated",
    "cargo-llvm-cov",
    "cargo-expand"
)

foreach ($ext in $extensions) {
    if (Get-Command $ext -ErrorAction SilentlyContinue) {
        Write-Host "[SKIP] $ext already installed" -ForegroundColor Gray
    } else {
        Write-Host "Installing $ext..." -ForegroundColor Yellow
        cargo install $ext
        if ($LASTEXITCODE -eq 0) {
            Write-Host "[OK] $ext installed" -ForegroundColor Green
        } else {
            Write-Host "[WARNING] Failed to install $ext" -ForegroundColor Yellow
        }
    }
}

Write-Host ""
Write-Host "Building project..." -ForegroundColor Yellow
cargo build
if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] Build failed" -ForegroundColor Red
    exit 1
}
Write-Host "[OK] Build successful" -ForegroundColor Green

Write-Host ""
Write-Host "Running tests..." -ForegroundColor Yellow
cargo test
if ($LASTEXITCODE -ne 0) {
    Write-Host "[WARNING] Some tests failed" -ForegroundColor Yellow
} else {
    Write-Host "[OK] All tests passed" -ForegroundColor Green
}

Write-Host ""
Write-Host "=== Setup Complete ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Quick start commands:" -ForegroundColor Yellow
Write-Host "  cargo run               - Run the application" -ForegroundColor White
Write-Host "  cargo test              - Run tests" -ForegroundColor White
Write-Host "  cargo clippy            - Run linter" -ForegroundColor White
Write-Host "  cargo watch -x run      - Auto-rebuild on changes" -ForegroundColor White
Write-Host "  bd ready                - Find work in beads" -ForegroundColor White
Write-Host ""
Write-Host "See DEVELOPMENT.md for detailed development guide" -ForegroundColor Cyan
Write-Host ""
