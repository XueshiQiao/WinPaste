#!/bin/bash

# Help Function
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "Usage: ./bump_version.sh [version|patch|minor|major]"
    echo ""
    echo "Examples:"
    echo "  ./bump_version.sh          # Default: patch bump (1.0.0 -> 1.0.1)"
    echo "  ./bump_version.sh patch    # Explicit patch bump"
    echo "  ./bump_version.sh minor    # Minor bump (1.0.0 -> 1.1.0)"
    echo "  ./bump_version.sh major    # Major bump (1.0.0 -> 2.0.0)"
    echo "  ./bump_version.sh 1.2.3    # Set specific version"
    exit 0
fi

# Default to patch if no argument provided
BUMP_TYPE=${1:-patch}

echo "📈 Bumping version ($BUMP_TYPE)..."

# Use tauri-version to handle package.json, tauri.conf.json, and Cargo.toml
# --no-git: Do not commit/tag automatically (let the user or CI handle it)
npx --yes tauri-version $BUMP_TYPE --no-git

echo "✅ Version bump complete!"