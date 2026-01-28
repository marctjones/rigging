# Rigging Patches Status

**Last Updated**: January 28, 2026

## Target Servo Version

Patches 0001-0008 are based on **Servo commit `97a670eb69f`** (Nov 30, 2025).

```bash
cd /path/to/servo
git checkout 97a670eb69f
```

## Patch Application Status

### ✅ Successfully Apply to Servo 97a670eb69f

| Patch | Description | Status |
|-------|-------------|--------|
| 0003-transport-types.patch | Transport enum and types (shared/net) | ✅ Applies |
| 0007-shared-net-lib.patch | Export transport types | ✅ Applies |
| 0006-net-cargo.patch | Add dependencies | ✅ Applies |
| 0005-net-lib.patch | Module exports | ✅ Applies |
| 0001-transport-url.patch | TransportUrl parsing | ✅ Applies |
| 0002-unix-connector.patch | Unix socket connector | ✅ Applies |
| 0008-tor-connector.patch | Tor connector | ✅ Applies |
| 0004-http-loader.patch | HTTP dispatch modifications | ✅ Applies |

### ⏳ Needs Update

| Patch | Description | Status | Notes |
|-------|-------------|--------|-------|
| 0009-connector-injection.patch | Custom connector injection | ⏳ Needs recreation | Created against newer Servo (Jan 2026). Needs to be recreated for Servo 97a670eb69f |

## Application Instructions

```bash
# Clone Servo
git clone https://github.com/servo/servo.git
cd servo

# Check out target commit
git checkout 97a670eb69f

# Apply Rigging patches
cd /path/to/rigging
./apply-patches.sh /path/to/servo

# Note: Patch 0009 will fail, manual application needed
```

## Manual Application: Patch 0009

For Servo commit 97a670eb69f, the connector injection approach needs to be adapted because:

1. `CoreResourceThreadOptions` struct doesn't exist in this version
2. HTTP client is created in `create_http_states()` function in `resource_thread.rs`
3. Simpler approach needed: modify `create_http_states()` to accept optional connector

**TODO**: Create updated 0009-connector-injection-v2.patch for Servo 97a670eb69f

## Compilation Testing

**Status**: Not yet tested

**Next Step**: Test that patched Servo compiles successfully

```bash
cd /path/to/servo
cargo build --package servoshell
```

## Updating Patches for New Servo Versions

When Servo updates and patches no longer apply:

1. Create a new branch in Servo:
   ```bash
   git checkout -b rigging-transport-layer
   ```

2. Manually apply and fix conflicts for each patch

3. Regenerate patches:
   ```bash
   cd /path/to/rigging
   ./regenerate-patches.sh /path/to/servo
   ```

4. Update this document with new target commit

## Phase 1 Progress (Issue #48)

- [x] Patches 0001-0008 apply cleanly to Servo 97a670eb69f
- [ ] Patch 0009 recreated for Servo 97a670eb69f
- [ ] Servo compiles with patches
- [ ] ServoBuilder::with_connector() method verified

## References

- **Servo Target Commit**: 97a670eb69f (Sync WPT with upstream, Nov 30 2025)
- **Patches Created**: Nov 27, 2025 (0001-0008), Jan 27, 2026 (0009)
- **GitHub Issue**: #48 - Create connector injection patch for Servo
- **Phase**: Phase 1 - Servo Patches & Setup
