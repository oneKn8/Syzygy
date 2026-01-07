#!/bin/bash
# Setup script for RFPMaker git hooks
# Run this once after cloning the repository

set -e

echo "Setting up RFPMaker git hooks..."

# Check if we're in a git repository
if [ ! -d ".git" ]; then
    echo "ERROR: Not in a git repository. Please run from the project root."
    exit 1
fi

# Check if .githooks directory exists
if [ ! -d ".githooks" ]; then
    echo "ERROR: .githooks directory not found."
    exit 1
fi

# Make hooks executable
chmod +x .githooks/*

# Configure git to use our hooks directory
git config core.hooksPath .githooks

echo ""
echo "Git hooks configured successfully!"
echo ""
echo "Hooks installed:"
ls -la .githooks/
echo ""
echo "Hooks location: .githooks/"
echo "Git hooks path: $(git config core.hooksPath)"
echo ""
echo "IMPORTANT: These hooks will:"
echo "  - Block commits with AI attribution (Co-Authored-By, Generated with)"
echo "  - Run cargo check/clippy/fmt on Rust files"
echo "  - Run tsc/eslint on TypeScript/JavaScript files"
echo "  - Warn about debug statements and large files"
echo "  - Enforce conventional commit message format"
echo ""
echo "To bypass hooks in emergencies: git commit --no-verify"
echo "(Use sparingly - hooks exist to maintain code quality)"
