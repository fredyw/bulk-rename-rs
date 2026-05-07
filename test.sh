#!/bin/bash

set -ueo pipefail

echo "Running formatting check..."
cargo fmt -- --check

echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "Running tests..."
cargo test "$@"
