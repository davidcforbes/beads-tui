#!/usr/bin/env bash
# Universal installation script for beads-tui
# Supports Linux and macOS

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REPO="davidcforbes/beads-tui"
BINARY_NAME="beads-tui"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case "$os" in
        linux*)
            OS="linux"
            ;;
        darwin*)
            OS="macos"
            ;;
        *)
            echo -e "${RED}Unsupported operating system: $os${NC}"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            echo -e "${RED}Unsupported architecture: $arch${NC}"
            exit 1
            ;;
    esac

    echo -e "${GREEN}Detected platform: $OS-$ARCH${NC}"
}

# Get latest release version
get_latest_version() {
    echo -e "${YELLOW}Fetching latest release...${NC}"
    VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"v([^"]+)".*/\1/')

    if [ -z "$VERSION" ]; then
        echo -e "${RED}Failed to fetch latest version${NC}"
        exit 1
    fi

    echo -e "${GREEN}Latest version: v$VERSION${NC}"
}

# Download and install binary
install_binary() {
    local download_url="https://github.com/$REPO/releases/download/v$VERSION/${BINARY_NAME}-${OS}-${ARCH}.tar.gz"
    local tmp_dir=$(mktemp -d)
    local archive="$tmp_dir/${BINARY_NAME}.tar.gz"

    echo -e "${YELLOW}Downloading from: $download_url${NC}"

    if ! curl -fsSL -o "$archive" "$download_url"; then
        echo -e "${RED}Failed to download release${NC}"
        rm -rf "$tmp_dir"
        exit 1
    fi

    echo -e "${YELLOW}Extracting archive...${NC}"
    tar -xzf "$archive" -C "$tmp_dir"

    # Create install directory if it doesn't exist
    mkdir -p "$INSTALL_DIR"

    echo -e "${YELLOW}Installing to $INSTALL_DIR...${NC}"
    mv "$tmp_dir/$BINARY_NAME" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    # Cleanup
    rm -rf "$tmp_dir"

    echo -e "${GREEN}✓ Successfully installed $BINARY_NAME v$VERSION${NC}"
}

# Verify installation
verify_installation() {
    if command -v "$BINARY_NAME" &> /dev/null; then
        echo -e "${GREEN}✓ $BINARY_NAME is in your PATH${NC}"
        "$BINARY_NAME" --version
    else
        echo -e "${YELLOW}⚠ $BINARY_NAME is not in your PATH${NC}"
        echo -e "Add $INSTALL_DIR to your PATH by adding this to your shell rc file:"
        echo -e "  ${GREEN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
    fi
}

# Main installation flow
main() {
    echo -e "${GREEN}=== beads-tui Installation ===${NC}\n"

    detect_platform
    get_latest_version
    install_binary
    verify_installation

    echo -e "\n${GREEN}Installation complete!${NC}"
    echo -e "Run '${GREEN}$BINARY_NAME --help${NC}' to get started"
}

main "$@"
