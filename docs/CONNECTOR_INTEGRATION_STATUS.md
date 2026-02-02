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

## What's Incomplete ❌

### The Type System Challenge

**Problem**: `HttpState::client` is typed as `ServoClient`:
```rust
pub struct HttpState {
    pub client: ServoClient,  // = Client<InstrumentedConnector<ProxyConnector>, BoxedBody>
    // ...
}
```

But `create_http_client_with_connector<C>()` returns:
```rust
Client<C, BoxedBody>  // Different type for each connector!
```

**Why This Matters**:
- HttpState.client field has a fixed type
- Custom connectors would create a different client type
- Rust's type system prevents mixing these at runtime

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

## Recommended Next Steps

1. **Implement Option 1** (Enum-Based Connector):
   ```rust
   // In servo/components/net/connector.rs
   pub enum FlexibleConnector {
       Default(InstrumentedConnector<ProxyConnector>),
       Custom(CustomConnectorWrapper),
   }

   // Wrapper that type-erases specific connector types
   struct CustomConnectorWrapper {
       inner: Box<dyn ServiceImpl>,  // Need to define this trait
   }
   ```

2. **Update create_http_states()** to use FlexibleConnector:
   ```rust
   let connector = if let Some(custom) = custom_connector {
       FlexibleConnector::Custom(wrap_connector(custom))
   } else {
       FlexibleConnector::Default(default_connector())
   };

   let client = Client::builder(TokioExecutor {})
       .build(connector);
   ```

3. **Implement Service for FlexibleConnector**:
   Dispatch to either Default or Custom variant

4. **Test End-to-End**:
   - Harbor with UnixConnector
   - Compass with TorConnector
   - Standard browsing with default connector

## Files Modified So Far

### Servo Changes
- ✅ `components/servo/servo.rs` - Pass custom_connector to new_resource_threads()
- ✅ `components/net/resource_thread.rs` - Thread custom_connector through to HTTP client creation
- ⏸️ `components/net/connector.rs` - Needs FlexibleConnector implementation

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

## Current Blocker

Cannot proceed with end-to-end testing until FlexibleConnector enum is implemented. This is the last piece needed for full connector integration.

## References

- Servo connector.rs: `/home/marc/servo/components/net/connector.rs`
- Servo resource_thread.rs: `/home/marc/servo/components/net/resource_thread.rs`
- Rigging connectors: `/home/marc/Projects/rigging/src/*_connector.rs`
- Implementation Plan: `/home/marc/Projects/rigging/IMPLEMENTATION_PLAN.md` (Phase 2.5/2.6)
