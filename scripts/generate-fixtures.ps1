#!/usr/bin/env pwsh
<#
.SYNOPSIS
    Generate deterministic test data fixtures for beads-tui

.DESCRIPTION
    Creates test beads databases with known issues, dependencies, and metadata.
    Each fixture is self-contained and can be copied to a temp directory for testing.

.PARAMETER Fixture
    Specific fixture to generate (test-small, test-medium, test-large, test-deps, test-edge, or all)

.EXAMPLE
    .\generate-fixtures.ps1
    Generate all fixtures

.EXAMPLE
    .\generate-fixtures.ps1 -Fixture test-small
    Generate only the small fixture
#>

param(
    [Parameter()]
    [ValidateSet("all", "test-small", "test-medium", "test-large", "test-deps", "test-edge")]
    [string]$Fixture = "all"
)

$ErrorActionPreference = "Stop"

# Helper functions
function Write-Info {
    param([string]$Message)
    Write-Host "→ $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "✓ $Message" -ForegroundColor Green
}

function Write-Section {
    param([string]$Title)
    Write-Host ""
    Write-Host "═══════════════════════════════════════════════" -ForegroundColor Blue
    Write-Host "  $Title" -ForegroundColor Blue
    Write-Host "═══════════════════════════════════════════════" -ForegroundColor Blue
}

# Create fixture directory if it doesn't exist
$FixturesDir = Join-Path $PSScriptRoot ".." "tests" "fixtures"
if (!(Test-Path $FixturesDir)) {
    New-Item -ItemType Directory -Path $FixturesDir | Out-Null
}

function New-Fixture {
    param(
        [string]$Name,
        [scriptblock]$Generator
    )

    Write-Section "Generating $Name"

    # Create temp directory outside repo to avoid importing git history
    $TempPath = Join-Path $env:TEMP "beads-fixture-$Name-$(Get-Random)"
    $FixturePath = Join-Path $FixturesDir $Name

    try {
        # Create temp directory and initialize clean database
        New-Item -ItemType Directory -Path $TempPath | Out-Null
        Push-Location $TempPath

        Write-Info "Initializing clean beads database in temp directory"
        git init | Out-Null
        bd init | Out-Null

        # Generate fixture data
        Write-Info "Generating fixture data"
        & $Generator

        # Export to JSONL
        Write-Info "Exporting to JSONL"
        bd export --format jsonl --output issues.jsonl | Out-Null

        # Create fixture manifest
        Write-Info "Creating fixture manifest"
        $Stats = Get-FixtureStats
        $Manifest = @{
            name = $Name
            description = (Get-FixtureDescription $Name)
            issue_count = $Stats.total
            created = (Get-Date -Format "yyyy-MM-dd")
            version = "1.0.0"
            statistics = @{
                open = $Stats.open
                closed = $Stats.closed
                blocked = $Stats.blocked
                by_type = $Stats.by_type
                by_priority = $Stats.by_priority
            }
            dependencies = @{
                total = $Stats.dep_count
                max_depth = $Stats.max_depth
                cycles = 0
            }
            focus_areas = (Get-FocusAreas $Name)
            known_issues = @()
        }

        $Manifest | ConvertTo-Json -Depth 10 | Set-Content "fixture.json"

        # Create README
        @"
# $Name

$(Get-FixtureDescription $Name)

## Statistics

- Total Issues: $($Stats.total)
- Open: $($Stats.open)
- Closed: $($Stats.closed)
- Blocked: $($Stats.blocked)

## Type Distribution

$($Stats.by_type | ConvertTo-Json)

## Priority Distribution

$($Stats.by_priority | ConvertTo-Json)

## Dependencies

- Total: $($Stats.dep_count)
- Max Depth: $($Stats.max_depth)

## Usage

``````rust
let (temp_dir, db_path) = setup_fixture("$Name");
``````

See docs/TEST_DATA.md for more information.
"@ | Set-Content "README.md"

        Write-Success "Generated $Name with $($Stats.total) issues in temp directory"

        # Copy to final location
        Pop-Location

        # Stop bd daemon to release lock files
        Stop-Process -Name "bd" -ErrorAction SilentlyContinue
        Start-Sleep -Milliseconds 500

        # Remove daemon lock files if they exist
        $LockFile = Join-Path $TempPath ".beads" "daemon.lock"
        if (Test-Path $LockFile) {
            Remove-Item -Force $LockFile -ErrorAction SilentlyContinue
        }

        Write-Info "Copying fixture to $FixturePath"

        # Remove old fixture if it exists
        if (Test-Path $FixturePath) {
            Remove-Item -Recurse -Force $FixturePath
        }

        # Copy temp to final location
        Copy-Item -Recurse -Force $TempPath $FixturePath

        Write-Success "Fixture $Name complete"
    }
    finally {
        # Clean up temp directory
        if (Test-Path $TempPath) {
            Remove-Item -Recurse -Force $TempPath -ErrorAction SilentlyContinue
        }
    }
}

function Get-FixtureStats {
    $AllIssues = bd list --status all --limit 0 2>$null | Select-String "^○|^✓" | Measure-Object | Select-Object -ExpandProperty Count
    $OpenIssues = bd list --status open --limit 0 2>$null | Select-String "^○" | Measure-Object | Select-Object -ExpandProperty Count
    $ClosedIssues = bd list --status closed --limit 0 2>$null | Select-String "^✓" | Measure-Object | Select-Object -ExpandProperty Count
    $BlockedIssues = bd blocked 2>$null | Select-String "^○" | Measure-Object | Select-Object -ExpandProperty Count

    return @{
        total = $AllIssues
        open = $OpenIssues
        closed = $ClosedIssues
        blocked = $BlockedIssues
        dep_count = 0  # TODO: Calculate actual dependency count
        max_depth = 0  # TODO: Calculate actual max depth
        by_type = @{
            task = 0
            bug = 0
            feature = 0
            epic = 0
            chore = 0
        }
        by_priority = @{
            P0 = 0
            P1 = 0
            P2 = 0
            P3 = 0
            P4 = 0
        }
    }
}

function Get-FixtureDescription {
    param([string]$Name)

    switch ($Name) {
        "test-small" { "Small dataset for smoke tests and basic functionality validation" }
        "test-medium" { "Medium dataset for standard integration testing and realistic workflows" }
        "test-large" { "Large dataset for performance testing and scalability validation" }
        "test-deps" { "Dependency-heavy dataset for graph visualization and dependency management testing" }
        "test-edge" { "Edge cases and boundary conditions for error handling validation" }
        default { "Test fixture for beads-tui" }
    }
}

function Get-FocusAreas {
    param([string]$Name)

    switch ($Name) {
        "test-small" { @("Basic CRUD operations", "Simple filtering", "Quick validation") }
        "test-medium" { @("Full workflows", "Filter validation", "Dependency graphs") }
        "test-large" { @("Performance benchmarks", "Render optimization", "Large lists") }
        "test-deps" { @("Dependency trees", "Cycle detection", "Critical paths") }
        "test-edge" { @("Input validation", "Error handling", "Unicode support") }
        default { @() }
    }
}

# Fixture generators

function Extract-IssueID {
    param([string[]]$output)
    $outputStr = $output -join "`n"
    if ($outputStr -match 'Created issue: ([\w-]+)') {
        return $Matches[1]
    }
    return $null
}

function Generate-TestSmall {
    # Create 12 issues with simple dependencies

    # Epic
    $Epic1 = Extract-IssueID (bd create --title "User Authentication" --type epic --priority 1 2>&1)

    # Tasks for epic
    $Task1 = Extract-IssueID (bd create --title "Implement login form" --type task --priority 2 2>&1)
    $Task2 = Extract-IssueID (bd create --title "Add password validation" --type task --priority 2 2>&1)
    $Task3 = Extract-IssueID (bd create --title "Create session management" --type task --priority 2 2>&1)

    # Bugs
    $Bug1 = Extract-IssueID (bd create --title "Login button not clickable" --type bug --priority 0 2>&1)
    $Bug2 = Extract-IssueID (bd create --title "Password field shows plain text" --type bug --priority 1 2>&1)

    # Features
    Extract-IssueID (bd create --title "Add remember me checkbox" --type feature --priority 3 2>&1) | Out-Null
    Extract-IssueID (bd create --title "Implement OAuth login" --type feature --priority 3 2>&1) | Out-Null

    # Chore
    Extract-IssueID (bd create --title "Update authentication docs" --type chore --priority 4 2>&1) | Out-Null

    # More tasks
    $Task4 = Extract-IssueID (bd create --title "Write tests for login" --type task --priority 2 2>&1)
    $Task5 = Extract-IssueID (bd create --title "Add error messages" --type task --priority 3 2>&1)

    # Create dependencies
    if ($Task1 -and $Bug1) { bd dep add $Bug1 $Task1 2>$null }
    if ($Task2 -and $Bug2) { bd dep add $Bug2 $Task2 2>$null }
    if ($Task3 -and $Task1) { bd dep add $Task3 $Task1 2>$null }
    if ($Task4 -and $Task1) { bd dep add $Task4 $Task1 2>$null }
    if ($Task4 -and $Task2) { bd dep add $Task4 $Task2 2>$null }

    # Close some issues
    if ($Task1) { bd close $Task1 --reason "Completed" 2>$null }
    if ($Bug1) { bd close $Bug1 --reason "Fixed" 2>$null }
    if ($Task5) { bd close $Task5 --reason "Done" 2>$null }
}

function Generate-TestMedium {
    Write-Info "Generating medium dataset (this may take a minute)..."

    # Create 3 epics
    for ($i = 1; $i -le 3; $i++) {
        $EpicId = bd create --title "Epic ${i}: Feature Area $i" --type epic --priority 1 2>&1 | Select-String "beads-" | ForEach-Object { $_.ToString().Trim() }

        # Create 8-10 tasks per epic
        for ($j = 1; $j -le (Get-Random -Minimum 8 -Maximum 11); $j++) {
            $Priority = Get-Random -Minimum 1 -Maximum 4
            $TaskId = bd create --title "Task ${i}.${j}" --type task --priority $Priority 2>&1 | Select-String "beads-" | ForEach-Object { $_.ToString().Trim() }

            # Randomly close 40% of tasks
            if ((Get-Random -Minimum 1 -Maximum 100) -le 40) {
                bd close $TaskId --reason "Completed" 2>$null
            }
        }
    }

    # Add bugs
    for ($i = 1; $i -le 10; $i++) {
        $Priority = Get-Random -Minimum 0 -Maximum 3
        bd create --title "Bug ${i}: Issue description" --type bug --priority $Priority 2>$null
    }

    # Add features
    for ($i = 1; $i -le 8; $i++) {
        bd create --title "Feature $i" --type feature --priority 2 2>$null
    }

    # Add chores
    for ($i = 1; $i -le 5; $i++) {
        bd create --title "Chore $i" --type chore --priority 3 2>$null
    }

    Write-Info "Medium dataset generated"
}

function Generate-TestLarge {
    Write-Info "Generating large dataset (this will take several minutes)..."

    # Create 15 epics
    for ($i = 1; $i -le 15; $i++) {
        bd create --title "Epic ${i}: Major Feature Area" --type epic --priority 1 2>$null
    }

    # Create 200 tasks
    for ($i = 1; $i -le 200; $i++) {
        $Priority = Get-Random -Minimum 1 -Maximum 5
        $TaskId = bd create --title "Task ${i}: Detailed task description" --type task --priority $Priority 2>&1 | Select-String "beads-" | ForEach-Object { $_.ToString().Trim() }

        # Close 60% of tasks
        if ((Get-Random -Minimum 1 -Maximum 100) -le 60) {
            bd close $TaskId --reason "Completed" 2>$null
        }

        # Progress indicator
        if ($i % 20 -eq 0) {
            Write-Info "Created $i/200 tasks..."
        }
    }

    # Create 50 bugs
    for ($i = 1; $i -le 50; $i++) {
        $Priority = Get-Random -Minimum 0 -Maximum 4
        bd create --title "Bug ${i}: Issue with detailed description" --type bug --priority $Priority 2>$null

        if ($i % 10 -eq 0) {
            Write-Info "Created $i/50 bugs..."
        }
    }

    # Create 30 features
    for ($i = 1; $i -le 30; $i++) {
        bd create --title "Feature ${i}: New capability" --type feature --priority 2 2>$null
    }

    # Create 20 chores
    for ($i = 1; $i -le 20; $i++) {
        bd create --title "Chore ${i}: Maintenance task" --type chore --priority 3 2>$null
    }

    Write-Info "Large dataset generated"
}

function Generate-TestDeps {
    Write-Info "Generating dependency-heavy dataset..."

    # Create a complex dependency tree
    $Root = bd create --title "Root Epic" --type epic --priority 1 2>&1 | Select-String "beads-" | ForEach-Object { $_.ToString().Trim() }

    # Level 1: 4 tasks depending on root
    $Level1 = @()
    for ($i = 1; $i -le 4; $i++) {
        $TaskId = bd create --title "Level 1 Task $i" --type task --priority 2 2>&1 | Select-String "beads-" | ForEach-Object { $_.ToString().Trim() }
        $Level1 += $TaskId
    }

    # Level 2: 8 tasks depending on level 1
    $Level2 = @()
    for ($i = 1; $i -le 8; $i++) {
        $TaskId = bd create --title "Level 2 Task $i" --type task --priority 2 2>&1 | Select-String "beads-" | ForEach-Object { $_.ToString().Trim() }
        $Level2 += $TaskId

        # Create dependency on random level 1 task
        $ParentIdx = Get-Random -Minimum 0 -Maximum $Level1.Count
        if ($Level1[$ParentIdx]) {
            bd dep add $TaskId $Level1[$ParentIdx] 2>$null
        }
    }

    # Level 3: 12 tasks depending on level 2
    for ($i = 1; $i -le 12; $i++) {
        $TaskId = bd create --title "Level 3 Task $i" --type task --priority 2 2>&1 | Select-String "beads-" | ForEach-Object { $_.ToString().Trim() }

        # Create dependency on random level 2 task
        $ParentIdx = Get-Random -Minimum 0 -Maximum $Level2.Count
        if ($Level2[$ParentIdx]) {
            bd dep add $TaskId $Level2[$ParentIdx] 2>$null
        }
    }

    # Add some diamond dependencies (task depends on two tasks that share a common parent)
    $Diamond1 = bd create --title "Diamond Task 1" --type task --priority 2 2>&1 | Select-String "beads-" | ForEach-Object { $_.ToString().Trim() }
    $Diamond2 = bd create --title "Diamond Task 2" --type task --priority 2 2>&1 | Select-String "beads-" | ForEach-Object { $_.ToString().Trim() }
    $DiamondChild = bd create --title "Diamond Child" --type task --priority 2 2>&1 | Select-String "beads-" | ForEach-Object { $_.ToString().Trim() }

    if ($Diamond1 -and $Diamond2 -and $DiamondChild) {
        bd dep add $DiamondChild $Diamond1 2>$null
        bd dep add $DiamondChild $Diamond2 2>$null
    }

    Write-Info "Dependency-heavy dataset generated"
}

function Generate-TestEdge {
    Write-Info "Generating edge case dataset..."

    # Empty title (if allowed)
    bd create --title " " --type task --priority 2 2>$null

    # Very long title (max length)
    $LongTitle = "A" * 200
    bd create --title $LongTitle --type task --priority 2 2>$null

    # Unicode and emoji
    bd create --title "Task with emoji 🚀 🎉 ✨" --type task --priority 2 2>$null
    bd create --title "Task with unicode: 日本語 中文 한글 العربية" --type task --priority 2 2>$null

    # Special characters
    bd create --title "Task with <HTML> & `"quotes`" and \backslashes\" --type task --priority 2 2>$null

    # All priorities
    bd create --title "P0 Critical" --type bug --priority 0 2>$null
    bd create --title "P1 High" --type task --priority 1 2>$null
    bd create --title "P2 Medium" --type task --priority 2 2>$null
    bd create --title "P3 Low" --type chore --priority 3 2>$null
    bd create --title "P4 Backlog" --type task --priority 4 2>$null

    # Many labels (if supported)
    bd create --title "Task with many labels" --type task --priority 2 2>$null

    # Closed immediately
    $ClosedId = bd create --title "Immediately closed" --type task --priority 2 2>&1 | Select-String "beads-" | ForEach-Object { $_.ToString().Trim() }
    if ($ClosedId) { bd close $ClosedId --reason "Closed immediately" 2>$null }

    # Whitespace variations
    bd create --title "Task`twith`ttabs" --type task --priority 2 2>$null
    bd create --title "Task`nwith`nnewlines" --type task --priority 2 2>$null

    Write-Info "Edge case dataset generated"
}

# Main execution

if ($Fixture -eq "all" -or $Fixture -eq "test-small") {
    New-Fixture -Name "test-small" -Generator ${function:Generate-TestSmall}
}

if ($Fixture -eq "all" -or $Fixture -eq "test-medium") {
    New-Fixture -Name "test-medium" -Generator ${function:Generate-TestMedium}
}

if ($Fixture -eq "all" -or $Fixture -eq "test-large") {
    New-Fixture -Name "test-large" -Generator ${function:Generate-TestLarge}
}

if ($Fixture -eq "all" -or $Fixture -eq "test-deps") {
    New-Fixture -Name "test-deps" -Generator ${function:Generate-TestDeps}
}

if ($Fixture -eq "all" -or $Fixture -eq "test-edge") {
    New-Fixture -Name "test-edge" -Generator ${function:Generate-TestEdge}
}

Write-Host ""
Write-Success "Fixture generation complete!"
Write-Info "Fixtures location: $FixturesDir"
Write-Info "See docs/TEST_DATA.md for usage information"
