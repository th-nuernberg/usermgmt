#!/bin/bash

set -eu

echo "Checking formatting"
cargo fmt --check --all
echo "==================="

echo "Checking linting"
# Fail at warning:
# cargo clippy --all -- -Dwarnings
cargo clippy --all
echo "==================="

echo "Checking unit test"
cargo test --workspace

