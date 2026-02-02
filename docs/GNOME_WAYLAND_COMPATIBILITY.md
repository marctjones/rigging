# GNOME Wayland Compatibility

## Problem

When running Rigging on **GNOME with Wayland compositor** (Mutter), windows created with winit 0.30 fail to appear, even though:
- Window creation succeeds
- Events are firing correctly
- Window state queries return `None` for visibility
- Position queries fail
- Window appears in taskbar but not on screen

## Root Cause

Winit 0.30 has compatibility issues with GNOME's Wayland compositor (Mutter). The library defaults to using the Wayland backend when available, which has window mapping bugs on GNOME.

**Environment variable ineffective**: Setting `WINIT_UNIX_BACKEND=x11` in the environment does NOT force the backend in winit 0.30—the library still uses Wayland when available.

## Solution

Force X11 backend programmatically using platform-specific API:

```rust
use winit::platform::x11::EventLoopBuilderExtX11;

#[cfg(target_os = "linux")]
{
    ServoShellEventLoop::Winit(
        WinitEventLoop::with_user_event()
            .with_x11()  // Force X11 backend
            .build()
            .expect("Could not start winit event loop with X11 backend"),
    )
}
```

**Location**: `src/servoshell/desktop/event_loop.rs:57-73`

## Results

With X11 backend forced:
- ✅ Window appears immediately
- ✅ `is_visible()` returns `Some(true)`
- ✅ `has_focus()` returns `true`
- ✅ All events work correctly (resize, focus, mouse, keyboard)
- ✅ Uses XWayland compatibility layer (X11 apps on Wayland)

## Why X11 Instead of Native Wayland?

1. **Maturity**: X11 support in winit is more stable
2. **Compatibility**: XWayland provides excellent X11 compatibility
3. **No user impact**: XWayland is transparent to users
4. **Upstream issue**: This is a winit 0.30 + GNOME bug, not a Rigging issue

## Testing Wayland vs X11

Backend detection can be done via `raw_window_handle`:

```rust
use raw_window_handle::RawWindowHandle;

match window_handle.as_raw() {
    RawWindowHandle::Wayland(_) => println!("Using Wayland"),
    RawWindowHandle::Xlib(_) => println!("Using X11 (Xlib)"),
    RawWindowHandle::Xcb(_) => println!("Using X11 (XCB)"),
    _ => println!("Unknown backend"),
}
```

## Environment Details

Tested on:
- **OS**: Ubuntu 24.04 (GNOME Wayland)
- **GNOME**: 46.0
- **Display Server**: Wayland (Mutter compositor)
- **Display**: 4K HiDPI (3840x2160, scale factor 2)
- **winit**: 0.30.x (from Servo)

## Related Issues

- Winit issue tracker: Search for "GNOME Wayland visibility"
- Servo tracking: This is a known servoshell issue

## Alternative Approaches (Not Used)

1. ❌ **Environment variable**: `WINIT_UNIX_BACKEND=x11` (ignored by winit 0.30)
2. ❌ **Wait for upstream fix**: Would block Harbor/Compass development
3. ❌ **Switch to native Wayland**: Too unstable in winit 0.30
4. ✅ **Force X11 programmatically**: Works reliably via XWayland

## Recommendation

**Keep X11 backend forced until**:
1. Winit releases a fix for GNOME Wayland window mapping
2. OR servo/servoshell upstream switches to newer winit with fix
3. OR we upgrade to winit 0.31+ with confirmed GNOME Wayland support

For now, X11 via XWayland is the stable, tested solution.
