# Rigging Phase 2 Progress - Servo Embedding

## Summary

Started forking servoshell embedding code into Rigging. Made significant progress on initial file structure.

## Completed ✅

1. **Directory Structure** - Created `src/servoshell/` and `src/servoshell/desktop/`
2. **Core Files Copied** (~2,000 lines from servoshell):
   - `running_app_state.rs` (759 lines) - Core state, WebViewDelegate
   - `window.rs` (370 lines) - Window abstraction, PlatformWindow trait
   - `desktop/app.rs` (249 lines) - App struct, ServoBuilder
   - `desktop/event_loop.rs` (174 lines) - Event loop waking
   - `desktop/keyutils.rs` (601 lines) - Keyboard event translation
   - `desktop/headless_window.rs` (163 lines) - Headless rendering
   - `desktop/headed_window.rs` (minimal placeholder) - TODO: Extract core from 1,350 line original

3. **Module Structure** - Created mod.rs files, added to lib.rs with `#[cfg(feature = "servo")]`

## In Progress 🔄

**Task #7: Fix imports and module paths**

Current compilation errors show what needs fixing:
- Import paths: `crate::window` → `crate::servoshell::window`
- Missing dependencies: Servo, winit, surfman, etc.
- Missing servoshell modules: prefs, webdriver, GamepadSupport

## Next Steps 📋

### Immediate (to get compilation working)

1. **Add Servo dependency** to Cargo.toml (when servo feature enabled):
   ```toml
   [dependencies]
   servo = { git = "https://github.com/servo/servo.git", optional = true }
   winit = { version = "...", optional = true }
   surfman = { version = "...", optional = true }
   # etc.
   ```

2. **Fix import paths** in all copied files:
   - `running_app_state.rs`: Update imports to use `crate::servoshell::`
   - `window.rs`: Update imports
   - `desktop/app.rs`: Update imports
   - All other files

3. **Create missing modules** or stub them out:
   - `prefs.rs` - Servo preferences (copy or stub)
   - `webdriver.rs` - WebDriver support (stub for now)
   - `GamepadSupport` - Gamepad support (stub for now)

4. **Extract headed_window.rs core** (~300 lines from 1,350):
   - PlatformWindow trait implementation
   - winit/surfman setup
   - Event handling (no egui)

### After Compilation Works

5. **Wire Connector trait** into Servo initialization (Task #8)
6. **Test basic embedding** - Create simple example
7. **Integration tests** - Verify Harbor can use new embedding API

## Technical Decisions

### Why Servo as subprocess is temporary

Current `embed/servo_backend.rs` launches Servo as subprocess:
- ✅ **Pro**: Simple, no complex dependencies
- ❌ **Con**: No control over rendering, networking, events

Phase 2 embeds Servo as library:
- ✅ **Pro**: Full control, can inject Connector, proper event handling
- ✅ **Pro**: Better performance, no IPC overhead
- ❌ **Con**: Complex dependency (WebIDL, Python build scripts, large compile)

### Why fork servoshell instead of importing

We could import servoshell as dependency, but:
- ❌ servoshell includes browser chrome (toolbar, tabs) we don't want
- ❌ servoshell APIs not stable for embedding use case
- ✅ Forking lets us strip browser UI, create stable embedding API
- ✅ We only need ~2,700 lines, not full 7,939 lines

## Files Modified

- `/home/marc/Projects/rigging/src/lib.rs` - Added servoshell module
- `/home/marc/Projects/rigging/src/servoshell/mod.rs` - New module
- `/home/marc/Projects/rigging/src/servoshell/desktop/mod.rs` - New module
- `/home/marc/Projects/rigging/src/servoshell/*.rs` - Copied from servoshell
- `/home/marc/Projects/rigging/src/servoshell/desktop/*.rs` - Copied from servoshell

## Estimated Remaining Work

- Fix imports: 2-3 hours
- Add Servo dependency + configure: 3-4 hours (complex build)
- Extract headed_window core: 4-6 hours (needs careful reading of 1,350 lines)
- Wire Connector trait: 2-3 hours
- Testing/debugging: 4-8 hours

**Total**: ~15-24 hours to complete Phase 2

## Related

- Rigging IMPLEMENTATION_PLAN.md - Full Phase 2 plan
- Harbor IMPLEMENTATION_PLAN.md - Harbor waiting on Phase 2
- Rigging issue #7 - Servo sync strategy improvements (deferred)

---

**Session Date**: 2026-01-27
**Status**: Phase 2 started, ~25% complete (file structure + copies done, imports/compilation pending)
