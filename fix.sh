#!/bin/bash

set -ueo pipefail

echo "Running formatting..."
cargo fmt

echo "Running fix..."
cargo fix --allow-dirty --all-targets --all-features

echo "Running clippy fix..."
cargo clippy --fix --allow-dirty --all-targets --all-features
