#!/bin/bash

# Fast development build and run script
echo "🚀 Starting fast development build..."

# Build first to catch any compilation errors
if ! cargo build --profile dev-fast; then
    echo "❌ Compilation failed. Fix errors before continuing."
    exit 1
fi

# Use fast profile with automatic rebuild on changes
# cargo watch -x "run --profile dev-fast" -w src -w assets -w settings.yaml
cargo run --profile dev-fast
