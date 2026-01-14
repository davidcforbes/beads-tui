# Windows installation script for beads-tui
# Run with: irm https://raw.githubusercontent.com/davidcforbes/beads-tui/main/scripts/install/install.ps1 | iex

param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\beads-tui"
)

$ErrorActionPreference = "Stop"

# Configuration
$Repo = "davidcforbes/beads-tui"
$BinaryName = "beads-tui.exe"

function Write-ColorOutput($Color, $Message) {
    Write-Host $Message -ForegroundColor $Color
}

function Get-LatestVersion {
    Write-ColorOutput Yellow "Fetching latest release..."

    try {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
        $version = $release.tag_name -replace '^v', ''
        Write-ColorOutput Green "Latest version: v$version"
        return $version
    }
    catch {
        Write-ColorOutput Red "Failed to fetch latest version: $_"
        exit 1
    }
}

function Install-Binary($Version) {
    $downloadUrl = "https://github.com/$Repo/releases/download/v$Version/beads-tui-windows-x86_64.zip"
    $tmpDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
    $archive = Join-Path $tmpDir "beads-tui.zip"

    Write-ColorOutput Yellow "Downloading from: $downloadUrl"

    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $archive
    }
    catch {
        Write-ColorOutput Red "Failed to download release: $_"
        Remove-Item -Recurse -Force $tmpDir
        exit 1
    }

    Write-ColorOutput Yellow "Extracting archive..."
    Expand-Archive -Path $archive -DestinationPath $tmpDir -Force

    # Create install directory
    if (-not (Test-Path $InstallDir)) {
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    Write-ColorOutput Yellow "Installing to $InstallDir..."
    $binaryPath = Join-Path $tmpDir $BinaryName
    $targetPath = Join-Path $InstallDir $BinaryName

    Copy-Item -Path $binaryPath -Destination $targetPath -Force

    # Cleanup
    Remove-Item -Recurse -Force $tmpDir

    Write-ColorOutput Green "✓ Successfully installed beads-tui v$Version"
    return $targetPath
}

function Add-ToPath($Directory) {
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")

    if ($currentPath -notlike "*$Directory*") {
        Write-ColorOutput Yellow "Adding $Directory to PATH..."
        $newPath = "$currentPath;$Directory"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        $env:Path = "$env:Path;$Directory"
        Write-ColorOutput Green "✓ Added to PATH (restart shell to take effect)"
    }
    else {
        Write-ColorOutput Green "✓ Already in PATH"
    }
}

function Test-Installation {
    try {
        $version = & beads-tui --version 2>&1
        Write-ColorOutput Green "✓ Installation verified: $version"
    }
    catch {
        Write-ColorOutput Yellow "⚠ Please restart your shell for PATH changes to take effect"
    }
}

# Main installation flow
try {
    Write-ColorOutput Green "=== beads-tui Installation ===`n"

    $version = Get-LatestVersion
    $binaryPath = Install-Binary $version
    Add-ToPath $InstallDir
    Test-Installation

    Write-ColorOutput Green "`nInstallation complete!"
    Write-ColorOutput White "Run 'beads-tui --help' to get started"
}
catch {
    Write-ColorOutput Red "Installation failed: $_"
    exit 1
}
