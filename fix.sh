#!/bin/bash

set -ueo pipefail

cargo fmt
cargo fix --allow-dirty
cargo clippy --fix --allow-dirty
