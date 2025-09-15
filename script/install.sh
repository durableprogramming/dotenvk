#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default installation directory
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Create installation directory if it doesn't exist
mkdir -p "$INSTALL_DIR"

# Detect OS and architecture
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$ARCH" in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        echo -e "${RED}Unsupported architecture: $ARCH${NC}"
        exit 1
        ;;
esac

# Build the binary
echo -e "${YELLOW}Building dotenvk...${NC}"
cargo build --release

# Copy binary to installation directory
echo -e "${YELLOW}Installing dotenvk to $INSTALL_DIR...${NC}"
cp target/release/dotenvk "$INSTALL_DIR/"

# Make sure the binary is executable
chmod +x "$INSTALL_DIR/dotenvk"

# Check if install directory is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo -e "${YELLOW}Warning: $INSTALL_DIR is not in your PATH${NC}"
    echo -e "${YELLOW}Add the following line to your shell configuration file:${NC}"
    echo -e "${GREEN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
fi

echo -e "${GREEN}dotenvk installed successfully!${NC}"

