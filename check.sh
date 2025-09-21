#!/bin/bash

set -ueo pipefail

cargo fmt -- --check
cargo clippy -- -D warnings
