#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

usage() {
    echo "Usage: $0 <version>"
    echo ""
    echo "Examples:"
    echo "  $0 1.0.0    # Release version v1.0.0"
    echo "  $0 1.0.1    # Release version v1.0.1"
    echo ""
    echo "This script will:"
    echo "  1. Update Cargo.toml version"
    echo "  2. Create a git tag"
    echo "  3. Push the tag to trigger GitHub Actions release"
    exit 1
}

check_prerequisites() {
    local required_commands=("git" "cargo" "jq")
    
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            log_error "Required command '$cmd' not found"
            exit 1
        fi
    done
    
    if [[ $(git status --porcelain) ]]; then
        log_error "Working directory is not clean. Please commit or stash changes first."
        exit 1
    fi
    
    local current_branch
    current_branch=$(git branch --show-current)
    if [[ "$current_branch" != "master" && "$current_branch" != "main" ]]; then
        log_error "Not on master/main branch. Currently on: $current_branch"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

update_cargo_version() {
    local version="$1"
    local cargo_file="$PROJECT_ROOT/Cargo.toml"
    
    log_info "Updating Cargo.toml version to $version"
    
    # Update version in Cargo.toml
    sed -i.bak "s/^version = \".*\"/version = \"$version\"/" "$cargo_file"
    
    # Verify the change
    local new_version
    new_version=$(grep '^version = ' "$cargo_file" | head -1 | sed 's/version = "\(.*\)"/\1/')
    
    if [[ "$new_version" != "$version" ]]; then
        log_error "Failed to update version in Cargo.toml"
        exit 1
    fi
    
    log_success "Updated Cargo.toml version to $version"
}

run_tests() {
    log_info "Running tests to ensure everything works"
    
    cd "$PROJECT_ROOT"
    cargo test --all-features
    cargo clippy --all-targets --all-features -- -D warnings
    cargo build --release
    
    log_success "All tests passed"
}

create_and_push_tag() {
    local version="$1"
    local tag_name="v$version"
    
    cd "$PROJECT_ROOT"
    
    log_info "Adding changes to git"
    git add Cargo.toml Cargo.lock
    git commit -m "Bump version to $version for release"
    
    log_info "Creating git tag: $tag_name"
    git tag -a "$tag_name" -m "Release version $tag_name"
    
    log_info "Pushing changes and tag to remote"
    git push origin HEAD
    git push origin "$tag_name"
    
    log_success "Tag $tag_name pushed successfully"
}

main() {
    if [[ $# -ne 1 ]]; then
        usage
    fi
    
    local version="$1"
    
    # Validate version format (basic semantic versioning)
    if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]]; then
        log_error "Invalid version format. Expected: X.Y.Z or X.Y.Z-suffix"
        exit 1
    fi
    
    log_info "Starting release process for version $version"
    
    check_prerequisites
    update_cargo_version "$version"
    run_tests
    create_and_push_tag "$version"
    
    log_success "Release process completed!"
    log_info "GitHub Actions will now build and publish the release artifacts."
    log_info "Check the Actions tab: https://github.com/durableprogramming/dotenvk/actions"
}

main "$@"