#!/bin/bash
# polymarket-sdk publish script
# Usage:
#   ./scripts/publish.sh check     - Run all pre-publish checks only
#   ./scripts/publish.sh publish   - Run checks and publish to crates.io
#   ./scripts/publish.sh bump <version> - Bump version, check, and publish

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Print colored output
info() { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[✓]${NC} $1"; }
warn() { echo -e "${YELLOW}[!]${NC} $1"; }
error() { echo -e "${RED}[✗]${NC} $1"; exit 1; }
step() { echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"; echo -e "${BLUE}▶${NC} $1"; }

# Get current version from Cargo.toml
get_version() {
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Update version in Cargo.toml and commit
bump_version() {
    local new_version=$1
    if [[ ! "$new_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.]+)?$ ]]; then
        error "Invalid version format: $new_version (expected: x.y.z or x.y.z-suffix)"
    fi

    local old_version=$(get_version)
    sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" Cargo.toml

    # Update Cargo.lock (for local builds)
    cargo check --quiet 2>/dev/null || true

    # Commit version bump (Cargo.lock is gitignored for library crates)
    git add Cargo.toml
    git commit -m "chore: bump version to $new_version"
    success "Version bumped from $old_version to $new_version (committed)"
}

# Pre-publish checks
run_checks() {
    local current_version=$(get_version)
    info "Running pre-publish checks for polymarket-sdk v$current_version"

    # 1. Format check
    step "Checking code formatting (cargo fmt)"
    if cargo fmt -- --check; then
        success "Code formatting OK"
    else
        warn "Code formatting issues found. Run 'cargo fmt' to fix."
        cargo fmt
        success "Code formatted"
    fi

    # 2. Clippy lint check (all features)
    step "Running clippy lints (all features)"
    cargo clippy --all-features --all-targets -- -D warnings
    success "Clippy checks passed"

    # 3. Build check (all features)
    step "Building with all features"
    cargo build --all-features
    success "Build successful"

    # 4. Build check (no default features)
    step "Building with no default features"
    cargo build --no-default-features
    success "Build (no-default-features) successful"

    # 5. Test
    step "Running tests"
    cargo test --all-features
    success "All tests passed"

    # 6. Documentation check
    step "Checking documentation"
    RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
    success "Documentation builds without warnings"

    # 7. Check for uncommitted changes
    step "Checking git status"
    if [[ -n $(git status --porcelain) ]]; then
        warn "Uncommitted changes detected:"
        git status --short
        echo ""
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            error "Aborted due to uncommitted changes"
        fi
    else
        success "Working directory clean"
    fi

    # 8. Package dry-run
    step "Running cargo package (dry-run)"
    cargo package --list
    echo ""
    cargo package --allow-dirty
    success "Package created successfully"

    # 9. Publish dry-run
    step "Running cargo publish (dry-run)"
    cargo publish --dry-run --allow-dirty
    success "Publish dry-run successful"

    echo ""
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✓ All pre-publish checks passed!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

# Publish to crates.io
do_publish() {
    local current_version=$(get_version)

    step "Publishing polymarket-sdk v$current_version to crates.io"

    echo -e "${YELLOW}This will publish to crates.io. This action cannot be undone.${NC}"
    read -p "Are you sure? (y/N) " -n 1 -r
    echo

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "Publish cancelled"
        exit 0
    fi

    cargo publish

    echo ""
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✓ Successfully published polymarket-sdk v$current_version${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "View at: https://crates.io/crates/polymarket-sdk"
}

# Create git tag
# Usage: create_tag [version] [--no-push]
create_tag() {
    local version="${1:-$(get_version)}"
    local auto_push="${2:-yes}"  # Default to push after publish
    local tag="v$version"

    step "Creating git tag $tag"

    if git rev-parse "$tag" >/dev/null 2>&1; then
        warn "Tag $tag already exists"
    else
        git tag -a "$tag" -m "Release $tag"
        success "Created tag $tag"

        if [[ "$auto_push" == "yes" ]]; then
            git push origin "$tag"
            success "Pushed tag $tag to origin"
        else
            read -p "Push tag to origin? (y/N) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                git push origin "$tag"
                success "Pushed tag $tag to origin"
            fi
        fi
    fi
}

# Show usage
usage() {
    echo "polymarket-sdk publish script"
    echo ""
    echo "Usage:"
    echo "  $0 check              Run all pre-publish checks"
    echo "  $0 publish            Run checks and publish to crates.io"
    echo "  $0 bump <version>     Bump version, run checks, and publish"
    echo "  $0 tag [version]      Create and push git tag (default: current version)"
    echo ""
    echo "Examples:"
    echo "  $0 check              # Just run checks"
    echo "  $0 publish            # Check and publish current version"
    echo "  $0 bump 0.1.0         # Bump to 0.1.0, check, and publish"
    echo "  $0 bump 0.1.0-beta.1  # Bump to pre-release version"
    echo "  $0 tag                # Tag current version from Cargo.toml"
    echo "  $0 tag 0.1.0          # Tag specific version"
    echo ""
    echo "Current version: $(get_version)"
}

# Main
case "${1:-}" in
    check)
        run_checks
        ;;
    publish)
        run_checks
        do_publish
        create_tag
        ;;
    bump)
        if [[ -z "${2:-}" ]]; then
            error "Version required. Usage: $0 bump <version>"
        fi
        bump_version "$2"
        run_checks
        do_publish
        create_tag
        ;;
    tag)
        create_tag "${2:-}" "no"  # Manual tag: prompt for push
        ;;
    -h|--help|help)
        usage
        ;;
    *)
        usage
        exit 1
        ;;
esac
