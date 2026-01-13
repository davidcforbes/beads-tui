# Installation Guide

Complete guide for installing beads-tui on different platforms.

## Prerequisites

Before installing beads-tui, you need:

1. **Rust 1.70 or higher**
   - Install from [rustup.rs](https://rustup.rs/)
   - Verify: `rustc --version`

2. **Beads CLI**
   - Install from [github.com/steveyegge/beads](https://github.com/steveyegge/beads)
   - Verify: `bd --version`

3. **Git** (optional, for development)
   - Install from [git-scm.com](https://git-scm.com/)
   - Verify: `git --version`

## Platform-Specific Instructions

### Windows

#### Using PowerShell

```powershell
# Install Rust (if not already installed)
Invoke-WebRequest -Uri https://win.rustup.rs -OutFile rustup-init.exe
.\rustup-init.exe -y

# Restart PowerShell to update PATH

# Clone and build beads-tui
git clone https://github.com/davidcforbes/beads-tui.git
cd beads-tui
cargo build --release

# Copy binary to a directory in PATH
Copy-Item target\release\beads-tui.exe C:\Users\$env:USERNAME\.cargo\bin\
```

#### Using Windows Terminal (Recommended)

Windows Terminal provides better color support and rendering:

1. Install Windows Terminal from Microsoft Store
2. Follow the PowerShell instructions above
3. Run `beads-tui` from Windows Terminal

### macOS

#### Using Homebrew (Recommended)

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build beads-tui
git clone https://github.com/davidcforbes/beads-tui.git
cd beads-tui
cargo build --release

# Install to local bin
cp target/release/beads-tui /usr/local/bin/
```

#### Using Cargo

```bash
# Install directly from crates.io (when published)
cargo install beads-tui
```

### Linux

#### Debian/Ubuntu

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install dependencies
sudo apt update
sudo apt install -y build-essential git

# Clone and build beads-tui
git clone https://github.com/davidcforbes/beads-tui.git
cd beads-tui
cargo build --release

# Install to local bin
sudo cp target/release/beads-tui /usr/local/bin/
```

#### Arch Linux

```bash
# Install Rust (if not already installed)
sudo pacman -S rust

# Clone and build beads-tui
git clone https://github.com/davidcforbes/beads-tui.git
cd beads-tui
cargo build --release

# Install to local bin
sudo cp target/release/beads-tui /usr/local/bin/
```

#### Fedora/RHEL

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install dependencies
sudo dnf install -y gcc git

# Clone and build beads-tui
git clone https://github.com/davidcforbes/beads-tui.git
cd beads-tui
cargo build --release

# Install to local bin
sudo cp target/release/beads-tui /usr/local/bin/
```

## Terminal Requirements

### Minimum Requirements

- 256-color support
- UTF-8 encoding
- Minimum size: 80x24 characters
- Recommended size: 120x40 or larger

### Recommended Terminals

**Windows:**
- Windows Terminal (best support)
- ConEmu
- Cmder

**macOS:**
- iTerm2 (recommended)
- Terminal.app
- Alacritty

**Linux:**
- Alacritty
- Kitty
- GNOME Terminal
- Konsole
- Terminator

## Beads CLI Setup

Before using beads-tui, initialize a beads repository:

```bash
# Navigate to your project directory
cd /path/to/your/project

# Initialize beads
bd init

# Verify setup
bd list
```

## First Run

After installation, start beads-tui:

```bash
# Run from any directory with a beads repository
beads-tui

# Or specify a path
beads-tui --path /path/to/project
```

## Verification

Verify your installation:

```bash
# Check version
beads-tui --version

# Check help
beads-tui --help

# Run in test mode
beads-tui --dry-run
```

## Configuration

### Config File Location

beads-tui looks for configuration in:

- **Linux/macOS**: `~/.config/beads-tui/config.toml`
- **Windows**: `%APPDATA%\beads-tui\config.toml`

### Sample Configuration

Create a config file with your preferences:

```toml
[ui]
theme = "default"
show_borders = true
highlight_selected = true

[colors]
primary = "cyan"
secondary = "yellow"
accent = "green"

[behavior]
auto_refresh = true
confirm_delete = true
vim_mode = true
```

## Troubleshooting

### Common Issues

#### "beads-tui: command not found"

Ensure `~/.cargo/bin` is in your PATH:

```bash
# Add to ~/.bashrc or ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"
```

#### "bd: command not found"

Install the beads CLI first:
- Follow instructions at [github.com/steveyegge/beads](https://github.com/steveyegge/beads)

#### "No such file or directory: .beads"

Initialize a beads repository first:
```bash
bd init
```

#### Color Issues

If colors look wrong:
1. Verify your terminal supports 256 colors:
   ```bash
   echo $TERM
   # Should show: xterm-256color or similar
   ```

2. Set TERM environment variable:
   ```bash
   export TERM=xterm-256color
   ```

#### Unicode/Box Drawing Issues

If box characters don't render correctly:
1. Verify UTF-8 encoding:
   ```bash
   echo $LANG
   # Should include: UTF-8
   ```

2. Use a terminal with better Unicode support (see Recommended Terminals)

### Performance Issues

If the TUI is slow:

1. **Check terminal emulator**: Some emulators are faster than others
2. **Reduce issue count**: Large issue databases (>10,000 issues) may be slow
3. **Disable auto-refresh**: Set `auto_refresh = false` in config
4. **Build in release mode**: Always use `--release` for better performance

## Uninstallation

To remove beads-tui:

```bash
# Remove binary
rm ~/.cargo/bin/beads-tui  # or wherever installed

# Remove config (optional)
rm -rf ~/.config/beads-tui  # Linux/macOS
# or
rmdir /s %APPDATA%\beads-tui  # Windows
```

## Updating

To update to the latest version:

```bash
# If installed from source
cd beads-tui
git pull origin main
cargo build --release
cargo install --path .

# If installed from crates.io (when published)
cargo install beads-tui --force
```

## Getting Help

- **Documentation**: [README.md](README.md)
- **Keyboard Shortcuts**: [KEYBOARD_SHORTCUTS.md](KEYBOARD_SHORTCUTS.md)
- **Contributing**: [CONTRIBUTING.md](CONTRIBUTING.md)
- **Issues**: [GitHub Issues](https://github.com/davidcforbes/beads-tui/issues)
- **Discussions**: [GitHub Discussions](https://github.com/davidcforbes/beads-tui/discussions)

## Next Steps

After installation:

1. Read [QUICKSTART.md](QUICKSTART.md) for a quick introduction
2. Review [KEYBOARD_SHORTCUTS.md](KEYBOARD_SHORTCUTS.md) to learn shortcuts
3. Check [CONTRIBUTING.md](CONTRIBUTING.md) if you want to contribute

---

**Note**: beads-tui is under active development. Features and installation methods may change. Check the repository for the latest instructions.
