# PowerShell script to update screenshot placeholders in UI_REFERENCE.md

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$DocsDir = Split-Path -Parent $ScriptDir
$UIRef = Join-Path $DocsDir "UI_REFERENCE.md"

Write-Host "Updating screenshot references in UI_REFERENCE.md..." -ForegroundColor Cyan

# Create backup
Copy-Item $UIRef "$UIRef.backup" -Force
Write-Host "Created backup: UI_REFERENCE.md.backup" -ForegroundColor Green

# Read content
$content = Get-Content $UIRef -Raw

# Count placeholders
$placeholderPattern = '\*\*Screenshot Placeholder:\*\*'
$totalMatches = ([regex]::Matches($content, $placeholderPattern)).Count
Write-Host "Found $totalMatches screenshot placeholders"

# Update placeholders to markdown images
# Transform: **Screenshot Placeholder:** `screenshot-XX-name.png`
# To: ![screenshot-XX-name.png](screenshots/screenshot-XX-name.png)
$content = $content -replace '\*\*Screenshot Placeholder:\*\* `(screenshot-\d{2}-[^`]+\.png)`', '![$1](screenshots/$1)'

# Update caption format
# Transform: *Caption: Some description*
# To: *Some description*
$content = $content -replace '(?m)^\*Caption: (.*)\*$', '*$1*'

# Save updated content
Set-Content -Path $UIRef -Value $content -NoNewline

# Count remaining placeholders
$remainingMatches = ([regex]::Matches($content, $placeholderPattern)).Count

Write-Host ""
Write-Host "Update complete!" -ForegroundColor Green
Write-Host "  Original placeholders: $totalMatches"
Write-Host "  Remaining placeholders: $remainingMatches"
Write-Host "  Updated references: $($totalMatches - $remainingMatches)"

if ($remainingMatches -eq 0) {
    Write-Host ""
    Write-Host "✅ All screenshot placeholders have been replaced!" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "⚠️  Some placeholders remain - check for formatting issues" -ForegroundColor Yellow
}

# Count actual screenshot files
$screenshotFiles = Get-ChildItem -Path $ScriptDir -Filter "screenshot-*.png" -ErrorAction SilentlyContinue
$screenshotCount = if ($screenshotFiles) { $screenshotFiles.Count } else { 0 }

Write-Host ""
Write-Host "Screenshot files in directory: $screenshotCount / 51"

if ($screenshotCount -lt 51) {
    $missing = 51 - $screenshotCount
    Write-Host "⚠️  Still need to capture $missing screenshots" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Backup saved at: UI_REFERENCE.md.backup" -ForegroundColor Cyan
Write-Host "To restore: Copy-Item UI_REFERENCE.md.backup UI_REFERENCE.md" -ForegroundColor Cyan
