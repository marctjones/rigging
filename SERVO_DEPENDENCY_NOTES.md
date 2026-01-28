# Servo Dependency Integration Notes

## Current Approach (2026-01-27)

Using **path dependencies** to local Servo checkout at `/home/marc/servo`.

### Dependencies Added to Cargo.toml

```toml
# Core Servo components
libservo = { path = "/home/marc/servo/components/servo", optional = true }
net = { path = "/home/marc/servo/components/net", optional = true }
webdriver_server = { path = "/home/marc/servo/components/webdriver_server", optional = true }

# Window/Graphics (when servo feature enabled)
winit = { version = "0.30", optional = true }
surfman = { version = "0.9", optional = true }
euclid = { version = "0.22", optional = true }
keyboard-types = { version = "0.7", optional = true }
raw-window-handle = { version = "0.6", optional = true }

# Supporting libraries
crossbeam-channel, dpi, ipc-channel, rustls, headers, gilrs, glow, mime_guess
```

### servo Feature Flag

```toml
servo = [
    "dep:libservo",
    "dep:net",
    "dep:webdriver_server",
    # ... plus all windowing/graphics dependencies
]
```

## Pros & Cons of Path Dependencies

### ✅ Pros
- Works immediately with local development
- Can iterate quickly on Servo patches
- Uses existing `/home/marc/servo` checkout

### ❌ Cons
- **Not portable** - Requires Servo at exact path on every machine
- **Not usable by others** - Contributors can't build without Servo checkout
- **CI/CD broken** - GitHub Actions won't work

## Future: Making This Portable

### Option A: Git Dependency (Preferred)

Point to marctjones/servo fork (with patches applied):

```toml
libservo = { git = "https://github.com/marctjones/servo.git", branch = "transport-layer", optional = true }
```

**Pros**: Portable, anyone can build
**Cons**: Requires maintaining transport-layer branch, coordinating Servo updates

### Option B: Hybrid (Development + CI)

Use path for local dev, git for CI:

```toml
# In Cargo.toml:
libservo = { path = "/home/marc/servo/components/servo", optional = true }

# Override in .cargo/config.toml for CI:
[patch."path:/home/marc/servo/components/servo"]
libservo = { git = "https://github.com/marctjones/servo.git", branch = "transport-layer" }
```

### Option C: Published Crates (Long-term)

If Servo components were published to crates.io:

```toml
libservo = { version = "0.0.2", optional = true }
```

**Pros**: Standard approach, easy for everyone
**Cons**: Servo doesn't publish components individually (yet)

## Build Time Expectations

### First Build
- **Time**: 20-40 minutes (depends on CPU cores)
- **Disk**: ~15 GB for target/ directory
- **RAM**: 8+ GB recommended (parallel compilation)

### Incremental Builds
- **Rigging changes**: 1-5 minutes
- **Servo changes**: 5-15 minutes (depends on what changed)

## Common Build Issues

### Issue: Out of Memory
```bash
# Limit parallel jobs
cargo build -j 4 --features servo
```

### Issue: Missing System Dependencies
Servo requires:
- Python 3.9+ (for build scripts)
- OpenSSL
- FreeType
- Fontconfig
- X11 development headers (Linux)

### Issue: Servo Version Mismatch
If Servo updates break patches:
1. Check `/home/marc/Projects/rigging/patches/`
2. Re-apply patches to newer Servo
3. Regenerate patches: `./regenerate-patches.sh /home/marc/servo`

## Testing Servo Integration

Once build succeeds, test with:

```bash
# Unit tests
cargo test -p rigging --features servo

# Example (when ready)
cargo run -p rigging --features servo --example servo_embedding
```

## Documentation TODO

- [ ] Add Servo setup instructions to README.md
- [ ] Document how to use git dependency for CI
- [ ] Create contributing guide for Servo integration
- [ ] Document Servo version compatibility matrix

## Related Files

- `/home/marc/Projects/rigging/Cargo.toml` - Dependency declarations
- `/home/marc/Projects/rigging/PHASE2_PROGRESS.md` - Overall Phase 2 progress
- `/home/marc/servo/` - Local Servo checkout (upstream)
- `/home/marc/Projects/rigging/patches/` - Transport layer patches for Servo

---

**Status**: First Servo build in progress (2026-01-27)
**Expected**: Build will take 20-40 minutes, then we'll see compilation errors to fix
