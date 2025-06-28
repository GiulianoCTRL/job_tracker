#!/bin/bash

# Script to simulate CI/CD pipeline locally
# Can be run locally or with docker image in ../docker

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_step() {
    echo -e "${BLUE}$1${NC}"
}

print_success() {
    echo -e "${GREEN}PASSED: $1${NC}"
}

print_error() {
    echo -e "${RED}FAILURE: $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}WARNING: $1${NC}"
}

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

run_step() {
    local step_name="$1"
    local command="$2"

    print_step "Running: $step_name"

    if eval "$command"; then
        print_success "$step_name"
        return 0
    else
        print_error "$step_name"
        return 1
    fi
}

main() {
    cd "$(dirname "$0")/.."

    echo
    print_step "Quick CI Test for Job Tracker"
    print_step "Working directory: $(pwd)"
    echo

    if ! command_exists cargo; then
        print_error "Rust/Cargo is not installed"
        exit 1
    fi

    run_step "Format Check" "cargo fmt -- --check" || exit 1
    run_step "Clippy Lint Check" "cargo clippy-all" || exit 1
    run_step "Unit and Integration Tests" "cargo test --all-features" || exit 1
    run_step "Release Build Check" "cargo build --release" || exit 1
    # Both instant and paste are no longer maintained, but are pulled in by iced. Skipping as this is a hobby project
    # rsa 0.10 should be fixing this upon release: https://github.com/RustCrypto/RSA/pull/394
    run_step "Security Audit" "cargo audit --ignore RUSTSEC-2023-0071 --ignore RUSTSEC-2024-0384 --ignore RUSTSEC-2024-0436" || exit 1

    echo
    print_success "All CI checks passed!"
    print_step "Code ready for CI/CD!"
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    echo "Quick CI Test Script for Job Tracker"
    echo
    echo "This script runs the essential CI checks locally:"
    echo "  1. Code formatting check"
    echo "  2. Lint check"
    echo "  3. All tests"
    echo "  4. Release build"
    echo "  5. Security audit"
    echo
    echo "Usage: $0"
    echo "       $0 --help    Show this help"
    echo
    echo "Prerequisites:"
    echo "  - Rust and Cargo installed"
    echo "  - cargo-audit installed"
    echo "  - All project dependencies available"
    echo
    exit 0
fi

main "$@"
