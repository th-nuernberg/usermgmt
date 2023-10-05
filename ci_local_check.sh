#!/bin/bash

set -eu

echo "Checking formatting"
cargo fmt --check --all
echo "==================="

echo "Checking linting"
cargo clippy -- -Dwarnings
echo "==================="

echo "Checking unit test"
cargo test --workspace
