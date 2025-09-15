#!/bin/sh

# Exit immediately if a command exits with a non-zero status.
set -e

# Get the directory of the script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# VHS tape file path
TAPE_FILE="$SCRIPT_DIR/demo.tape"

# Output GIF path
OUTPUT_GIF="$SCRIPT_DIR/demo.gif"

export PS1="> "

# Function to check dependencies
check_dependencies() {
    if ! command -v vhs &> /dev/null; then
        echo "Error: vhs command not found. Please install VHS (https://github.com/charmbracelet/vhs)." >&2
        exit 1
    fi

    if ! command -v cargo &> /dev/null; then
        echo "Error: cargo command not found. Please install Rust and Cargo." >&2
        exit 1
    fi
}

# Function to build dotenvk if needed
build_dotenvk() {
    echo "Building dotenvk..."
    cd "$SCRIPT_DIR/.."
    cargo build --release
    
    # Check if dotenvk is in PATH or create a symlink
    if ! command -v dotenvk &> /dev/null; then
        echo "Adding dotenvk to PATH for this demo..."
        export PATH="$SCRIPT_DIR/../target/release:$PATH"
    fi
}

# Function to setup demo project
setup_demo_project() {
    echo "Setting up demo project..."
    cd "$SCRIPT_DIR"
    mkdir -p my-app
    cd my-app
    
    # Create initial .env file
    cat > .env << 'EOF'
# Database Configuration
DATABASE_URL=postgres://localhost/mydb
DATABASE_POOL_SIZE=10

# Server Configuration
PORT=3000
HOST=localhost
EOF
    
    cd "$SCRIPT_DIR"
    echo "Demo project setup complete."
}

# Function to clean up any leftover files
cleanup() {
    echo "Cleaning up..."
    cd "$SCRIPT_DIR"
    rm -rf my-app
    rm -f .env
    echo "Cleanup complete."
}

# Trap EXIT signal to ensure cleanup runs
trap cleanup EXIT

# Check for required tools
check_dependencies

# Build the project
build_dotenvk

# Setup the demo project
setup_demo_project

# Change to demo directory for recording
cd "$SCRIPT_DIR"

echo "Recording dotenvk demo with VHS..."
echo "Output will be saved to: $OUTPUT_GIF"

# Record the demo
vhs "$TAPE_FILE"

echo "Demo recording complete! GIF saved to $OUTPUT_GIF"

# Explicitly exit successfully if we reach here. Cleanup will still run.
exit 0