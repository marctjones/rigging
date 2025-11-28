#!/bin/bash
# Rigging - Regenerate patches from Servo fork
#
# Usage: ./regenerate-patches.sh /path/to/servo-fork
#
# This script regenerates the Rigging patches from a Servo fork
# that has the transport layer modifications applied.
# Typically used with marctjones/servo (transport-layer branch).

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PATCHES_DIR="$SCRIPT_DIR/patches"

if [ -z "$1" ]; then
    echo "Usage: $0 /path/to/servo-fork"
    echo ""
    echo "Regenerate patches from a modified Servo fork."
    echo "The fork should have 'upstream' remote pointing to servo/servo."
    exit 1
fi

SERVO_DIR="$1"

if [ ! -d "$SERVO_DIR" ]; then
    echo "Error: Directory '$SERVO_DIR' does not exist"
    exit 1
fi

cd "$SERVO_DIR"

# Check for upstream remote
if ! git remote get-url upstream >/dev/null 2>&1; then
    echo "Error: No 'upstream' remote found. Add it with:"
    echo "  git remote add upstream https://github.com/servo/servo.git"
    exit 1
fi

echo "Regenerating patches from $SERVO_DIR..."
echo ""

mkdir -p "$PATCHES_DIR"

# Fetch upstream if needed
echo "Fetching upstream..."
git fetch upstream main --depth=1 2>/dev/null || true

echo ""
echo "Generating patches..."

# Generate patches for transport-specific files
git diff upstream/main HEAD -- components/net/transport_url.rs > "$PATCHES_DIR/0001-transport-url.patch"
git diff upstream/main HEAD -- components/net/unix_connector.rs > "$PATCHES_DIR/0002-unix-connector.patch"
git diff upstream/main HEAD -- components/shared/net/transport.rs > "$PATCHES_DIR/0003-transport-types.patch"
git diff upstream/main HEAD -- components/net/http_loader.rs > "$PATCHES_DIR/0004-http-loader.patch"
git diff upstream/main HEAD -- components/net/lib.rs > "$PATCHES_DIR/0005-net-lib.patch"
git diff upstream/main HEAD -- components/net/Cargo.toml > "$PATCHES_DIR/0006-net-cargo.patch"
git diff upstream/main HEAD -- components/shared/net/lib.rs > "$PATCHES_DIR/0007-shared-net-lib.patch"
git diff upstream/main HEAD -- components/net/tor_connector.rs > "$PATCHES_DIR/0008-tor-connector.patch"

echo ""
echo "Generated patches:"
ls -la "$PATCHES_DIR"/*.patch 2>/dev/null || echo "  (none)"

echo ""
echo "=========================================="
echo "Patch regeneration complete!"
echo ""
echo "Don't forget to commit the updated patches:"
echo "  cd $SCRIPT_DIR"
echo "  git add patches/"
echo "  git commit -m 'Update patches from servo fork'"
echo "=========================================="
