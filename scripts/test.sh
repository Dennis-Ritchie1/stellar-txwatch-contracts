#!/usr/bin/env bash
# scripts/test.sh — Run all contract tests
set -euo pipefail

# Unit tests for individual contracts
cargo test -p alert-registry -p watcher-registry "$@"

# Integration tests exercising cross-contract behavior
cargo test -p integration-tests "$@"
