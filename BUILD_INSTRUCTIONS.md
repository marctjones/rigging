# Building Rigging with Servo

## Quick Start

```bash
# In a separate terminal
cd /home/marc/Projects/harbor
cargo build -p rigging --features servo

# Or directly in Rigging
cd /home/marc/Projects/rigging
cargo build --features servo
```

## What to Expect

### First Build (20-40 minutes)
- Compiling 617 packages
- Building Servo engine (~15 GB in target/)
- High CPU/memory usage (8+ GB RAM recommended)

### Progress Indicators
You'll see packages compiling like:
```
   Compiling libservo v0.0.1 (/home/marc/servo/components/servo)
   Compiling script v0.0.1 (/home/marc/servo/components/script)
   Compiling style v0.0.1 (/home/marc/servo/components/style)
```

### When Build Completes

**If successful**, you'll see:
```
   Compiling rigging v0.2.0 (/home/marc/Projects/rigging)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in XX.XXs
```

**If errors**, you'll see compilation errors about:
- Missing imports in servoshell code
- Type mismatches
- Unresolved modules

## After Build Completes

Come back to Claude and we'll:
1. Fix any compilation errors
2. Fix import paths in servoshell code
3. Test basic compilation
4. Continue with Phase 2

## Monitoring Build

```bash
# Watch progress (in build terminal)
cargo build -p rigging --features servo

# Check what's compiling (in another terminal)
ps aux | grep rustc | wc -l  # Number of parallel rustc processes

# Check disk usage
du -sh /home/marc/Projects/harbor/target
```

## Troubleshooting

### Canvas Compilation Errors (non-exhaustive enum patterns)
If you see errors like `non-exhaustive patterns: type '&mut Canvas' is non-empty`:
- **Cause**: libservo default features are disabled in Cargo.toml
- **Fix**: Ensure `libservo` dependency does NOT have `default-features = false`
- Default features include `vello_cpu` which is required for Canvas rendering

### Out of Memory
```bash
# Limit parallel jobs
cargo build -p rigging --features servo -j 4
```

### Build Hangs
- Check memory usage (might be swapping)
- Kill and restart with fewer parallel jobs

### Missing Dependencies
Servo requires system packages. If errors about missing libraries:
```bash
sudo apt install libssl-dev libfreetype6-dev libfontconfig1-dev
```

---

**Current Status**: First Servo build in progress
**Next Step**: Wait for build, then fix compilation errors
