# Connector Integration Status

## Summary

We've made significant progress on integrating Rigging's transport-aware connectors with Servo, but there's one remaining architectural challenge to solve.

## What's Complete ✅

### 1. Connector Infrastructure (Phase 1 & Early Phase 2)
- ✅ TransportUrl parsing (http::unix://, http::tor://, etc.)
- ✅ UnixConnector implements Service<Uri>
- ✅ TcpConnector implements Service<Uri>
- ✅ TorConnector implements Service<Uri>
- ✅ ComposedConnector routes to appropriate connector based on transport type
- ✅ All connectors work with Hyper/Tower

### 2. ServoBuilder Integration
- ✅ `ServoBuilder::with_connector()` method exists (servo.rs:1294-1300)
- ✅ `custom_connector` field in ServoBuilder (servo.rs:1231)
- ✅ Custom connector passed from ServoBuilder to new_resource_threads()
- ✅ Custom connector threaded through to create_http_states()

### 3. Connector Creation Function
- ✅ `create_http_client_with_connector<C>()` exists in net/connector.rs:637
- ✅ Function accepts generic connector implementing Service<Uri>
- ✅ Returns properly configured Hyper client

### 4. Rigging Integration
- ✅ `servoshell/desktop/app.rs` calls `.with_connector(UnixConnector::dynamic())` (line 118)
- ✅ Servo embedding code ready to inject connector

## What Was Completed ✅ (Phase 2.6)

### The Type System Solution

**Problem Solved**: `HttpState::client` needed a single type that could handle both default and custom connectors.

**Solution Implemented**: FlexibleConnector enum with manual vtable approach:

```rust
// Enum connector supporting both default and custom variants
pub enum FlexibleConnector {
    Default(InstrumentedConnector<ProxyConnector>),
    Custom(Arc<Mutex<Box<dyn DynConnector>>>),
}

// Manual vtable trait for type-erasing connections
pub trait DynConnection: Send + Sync {
    fn dyn_poll_read(&mut self, cx: &mut Context<'_>, buf: ReadBufCursor<'_>) -> Poll<Result<(), io::Error>>;
    fn dyn_poll_write(&mut self, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, io::Error>>;
    fn dyn_poll_flush(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>>;
    fn dyn_poll_shutdown(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>>;
    fn dyn_connected(&self) -> Connected;
}

// Type-erased connection wrapper
pub struct BoxedConnection {
    inner: Box<dyn DynConnection>,
}

// FlexibleConnection dispatches to both variants
pub enum FlexibleConnection {
    Default(<InstrumentedConnector<ProxyConnector> as Service<Uri>>::Response),
    Custom(BoxedConnection),
}

pub type ServoClient = Client<FlexibleConnector, BoxedBody>;
```

**Why This Works**:
- Single `ServoClient` type for all use cases
- Zero-cost dispatch for Default variant (direct enum match)
- Type-erased dispatch for Custom variant (manual vtable)
- No unsafe code required
- MutexGuard released before await (avoids Send issues)

### Options to Fix This

**Option 1: Enum-Based Connector Wrapper** (Recommended)
Create a new connector type that wraps both default and custom:
```rust
pub enum FlexibleConnector {
    Default(InstrumentedConnector<ProxyConnector>),
    Custom(Box<dyn ServiceTrait>),  // Needs trait object support
}

pub type ServoClient = Client<FlexibleConnector, BoxedBody>;
```

**Pros**:
- Minimal API changes
- Type-safe
- No performance impact for default case

**Cons**:
- Need to implement Service for the enum
- Slight complexity in connector dispatch

**Option 2: Make HttpState Generic** (Too Invasive)
```rust
pub struct HttpState<C> {
    pub client: Client<C, BoxedBody>,
    // ...
}
```

**Pros**:
- Most flexible

**Cons**:
- Requires changing LOTS of code throughout Servo
- Breaking change to Servo's API
- Not practical for upstream contribution

**Option 3: Dynamic Dispatch Only** (Performance Cost)
Use trait objects everywhere:
```rust
pub type ServoClient = Client<Box<dyn DynConnector>, BoxedBody>;
```

**Pros**:
- Fully dynamic

**Cons**:
- Performance overhead from dynamic dispatch
- Hyper doesn't support trait object connectors well
- Complex lifetime management

## Next Steps (Integration Testing)

FlexibleConnector is now fully implemented in Servo. The remaining work is integration and testing:

1. **Wire up Rigging's Connectors to Servo**:
   - Test that UnixConnector works with ServoBuilder.with_connector()
   - Verify ComposedConnector routes correctly
   - Ensure transport-aware URLs flow through to connectors

2. **Update Rigging's Servo Integration**:
   - Remove URL conversion hack in `src/embed/servoshell_backend.rs`
   - Pass ComposedConnector to ServoBuilder in app.rs
   - Test that `http::unix://` URLs connect via Unix sockets

3. **Test End-to-End with Harbor**:
   - Start Flask backend on Unix socket
   - Load page via `http::unix:///tmp/app.sock/`
   - Verify no TCP connections attempted
   - Test relative URLs inherit transport from base

4. **Add Transport Restriction Config**:
   - Implement BrowserConfig.allowed_transports
   - Reject disallowed transport types at connector level
   - Add external link detection for Harbor

## Files Modified

### Servo Changes (COMPLETE)
- ✅ `components/servo/servo.rs` - Added ServoBuilder.with_connector() method
- ✅ `components/net/resource_thread.rs` - Thread custom_connector through to HTTP client creation
- ✅ `components/net/connector.rs` - FlexibleConnector with DynConnection manual vtable (~290 lines added)

### Rigging Changes
- ✅ `src/servoshell/desktop/app.rs` - Calls .with_connector()
- ✅ `src/unix_connector.rs` - Implements Service<Uri> with dynamic URL parsing
- ✅ `src/composed.rs` - Routes based on transport type

## Testing Strategy

Once FlexibleConnector is implemented:

1. **Unit Test**: FlexibleConnector dispatch
2. **Integration Test**: Rigging with UnixConnector
3. **Full Stack Test**: Harbor loading from Unix socket
4. **Regression Test**: Standard HTTP still works with default connector

## Estimated Remaining Work

- **FlexibleConnector implementation**: ~100 lines
- **Service trait implementation**: ~50 lines
- **Type erasure wrapper**: ~75 lines
- **Testing**: ~100 lines
- **Total**: ~325 lines in Servo's net component

## Current Status

✅ **FlexibleConnector COMPLETE** - Custom variant fully implemented with manual vtable approach.

🚧 **Integration Testing IN PROGRESS** - Need to wire up Rigging's connectors and test end-to-end.

The Servo-side infrastructure is complete. Next phase is Rigging-side integration and testing.

## References

- Servo connector.rs: `/home/marc/servo/components/net/connector.rs`
- Servo resource_thread.rs: `/home/marc/servo/components/net/resource_thread.rs`
- Rigging connectors: `/home/marc/Projects/rigging/src/*_connector.rs`
- Implementation Plan: `/home/marc/Projects/rigging/IMPLEMENTATION_PLAN.md` (Phase 2.5/2.6)
