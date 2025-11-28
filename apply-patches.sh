#!/bin/bash
# Rigging - Apply transport layer patches to Servo
#
# Usage: ./apply-patches.sh /path/to/servo
#
# This script applies the Rigging transport layer patches to a fresh
# Servo checkout, enabling Unix Domain Sockets, Named Pipes, Tor,
# and other transport mechanisms.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PATCHES_DIR="$SCRIPT_DIR/patches"

if [ -z "$1" ]; then
    echo "Usage: $0 /path/to/servo"
    echo ""
    echo "This script applies Rigging transport patches to Servo."
    echo ""
    echo "Example:"
    echo "  git clone https://github.com/servo/servo.git"
    echo "  $0 ./servo"
    exit 1
fi

SERVO_DIR="$1"

if [ ! -d "$SERVO_DIR" ]; then
    echo "Error: Directory '$SERVO_DIR' does not exist"
    exit 1
fi

if [ ! -f "$SERVO_DIR/Cargo.toml" ]; then
    echo "Error: '$SERVO_DIR' does not appear to be a Servo checkout"
    exit 1
fi

echo "Applying Rigging patches to $SERVO_DIR..."
echo ""

cd "$SERVO_DIR"

# Check for uncommitted changes
if ! git diff --quiet HEAD 2>/dev/null; then
    echo "Warning: Servo repository has uncommitted changes"
    echo "Consider committing or stashing them first."
    read -p "Continue anyway? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Apply patches in order
PATCHES=(
    "0003-transport-types.patch"      # Transport enum and types (shared/net)
    "0007-shared-net-lib.patch"       # Export transport types
    "0006-net-cargo.patch"            # Add dependencies
    "0005-net-lib.patch"              # Module exports
    "0001-transport-url.patch"        # TransportUrl parsing
    "0002-unix-connector.patch"       # Unix socket connector
    "0008-tor-connector.patch"        # Tor connector
    "0004-http-loader.patch"          # HTTP dispatch modifications
)

APPLIED=0
FAILED=0

for patch in "${PATCHES[@]}"; do
    patch_file="$PATCHES_DIR/$patch"
    if [ -f "$patch_file" ]; then
        echo -n "Applying $patch... "
        if git apply --check "$patch_file" 2>/dev/null; then
            git apply "$patch_file"
            echo "OK"
            ((APPLIED++))
        else
            echo "FAILED (may already be applied or conflicts exist)"
            ((FAILED++))
        fi
    else
        echo "Warning: Patch file not found: $patch_file"
        ((FAILED++))
    fi
done

echo ""
echo "=========================================="
echo "Patch application complete!"
echo "  Applied: $APPLIED"
echo "  Failed:  $FAILED"
echo ""
echo "Next steps:"
echo "  1. Review any failed patches manually"
echo "  2. Build Servo with: cargo build --package servoshell"
echo "  3. Run tests with: cargo test --package net"
echo "=========================================="
