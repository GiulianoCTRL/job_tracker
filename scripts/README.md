# Local CI/CD Testing Guide for Job Tracker

This directory contains scripts to test the CI/CD pipeline locally before pushing to GitHub.

## Quick Start

Run the quick CI test to verify everything works:

```bash
./scripts/ci.sh
```

## Available Scripts

### `ci.sh`
Local testing that mimics the essential CI checks:
- Code formatting (`cargo fmt --check`)
- Linting (`cargo clippy`)
- All tests (`cargo test`)
- Release build (`cargo build --release`)
- Security audit (`cargo audit`)

**Usage:**
```bash
./scripts/ci.sh          # Run all checks
./scripts/ci.sh --help   # Show help
```

**Prerequisites:**
- Rust and Cargo (clippy and cargo fmt) installed
- cargo-audit installed
- All project dependencies available
