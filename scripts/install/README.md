# beads-tui Installation Scripts

Quick installation scripts for beads-tui across different platforms.

## Quick Install

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/davidcforbes/beads-tui/main/scripts/install/install.sh | bash
```

Or download and run locally:

```bash
wget https://raw.githubusercontent.com/davidcforbes/beads-tui/main/scripts/install/install.sh
chmod +x install.sh
./install.sh
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/davidcforbes/beads-tui/main/scripts/install/install.ps1 | iex
```

Or download and run locally:

```powershell
Invoke-WebRequest -Uri "https://raw.githubusercontent.com/davidcforbes/beads-tui/main/scripts/install/install.ps1" -OutFile install.ps1
.\install.ps1
```

## Custom Install Location

### Linux / macOS

```bash
export INSTALL_DIR="$HOME/bin"
curl -fsSL https://raw.githubusercontent.com/davidcforbes/beads-tui/main/scripts/install/install.sh | bash
```

### Windows

```powershell
.\install.ps1 -InstallDir "C:\Tools\beads-tui"
```

## What the Scripts Do

1. **Detect Platform**: Automatically detects your OS and CPU architecture
2. **Fetch Latest Release**: Gets the latest version from GitHub Releases
3. **Download Binary**: Downloads the appropriate pre-built binary
4. **Verify Checksum**: Ensures the download is not corrupted (future enhancement)
5. **Install**: Extracts and installs to the specified directory
6. **Update PATH**: Adds the install directory to your PATH (if needed)
7. **Verify**: Tests that the installation was successful

## Supported Platforms

- **Linux**: x86_64, aarch64 (ARM64)
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x86_64

## Manual Installation

If you prefer to install manually:

1. Go to [Releases](https://github.com/davidcforbes/beads-tui/releases/latest)
2. Download the archive for your platform:
   - Linux x86_64: `beads-tui-linux-x86_64.tar.gz`
   - Linux ARM64: `beads-tui-linux-aarch64.tar.gz`
   - macOS Intel: `beads-tui-macos-x86_64.tar.gz`
   - macOS ARM: `beads-tui-macos-aarch64.tar.gz`
   - Windows: `beads-tui-windows-x86_64.zip`
3. Extract the archive
4. Move the `beads-tui` binary to a directory in your PATH

## Install from Source

If you have Rust installed:

```bash
cargo install --git https://github.com/davidcforbes/beads-tui
```

Or from crates.io (once published):

```bash
cargo install beads-tui
```

## Updating

To update to the latest version, simply run the installation script again. It will automatically download and install the newest release.

## Uninstallation

### Linux / macOS

```bash
rm ~/.local/bin/beads-tui
# Or if you used a custom install directory:
# rm $INSTALL_DIR/beads-tui
```

### Windows

```powershell
Remove-Item "$env:LOCALAPPDATA\Programs\beads-tui\beads-tui.exe"
# Then manually remove from PATH if desired
```

## Troubleshooting

### Command not found after installation

The installation directory may not be in your PATH. Add it to your shell configuration:

**Bash/Zsh** (~/.bashrc or ~/.zshrc):
```bash
export PATH="$PATH:$HOME/.local/bin"
```

**Fish** (~/.config/fish/config.fish):
```fish
set -gx PATH $PATH $HOME/.local/bin
```

**Windows**: The script should automatically add to PATH, but you may need to restart your shell.

### Permission denied

Make sure the script is executable:

```bash
chmod +x install.sh
```

### Download fails

Check your internet connection and ensure you can access github.com. If you're behind a proxy, configure your shell's proxy settings.

## Security

Always verify the integrity of scripts before running them. You can:

1. Download the script first and review it
2. Verify the SHA256 checksum of downloaded binaries (checksums are provided with each release)
3. Build from source if you prefer maximum security

## Support

For issues with the installation scripts, please [open an issue](https://github.com/davidcforbes/beads-tui/issues) on GitHub.
