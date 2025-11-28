# Rigging Implementation Plan

## Phase 1: Core Foundation (Current)

### 1.1 Basic Types
- [x] `Transport` enum
- [x] `TransportChain` struct
- [x] `TransportError` error types

### 1.2 URL Parsing
- [x] `TransportUrl` struct
- [x] Parse transport from URL scheme
- [x] Extract socket path for Unix/Pipe
- [x] Standard URL component extraction

### 1.3 TCP Connector
- [x] Basic TCP connection
- [ ] IPv6 support verification
- [ ] Connection timeout configuration
- [ ] Keep-alive settings

### 1.4 Unix Connector
- [x] Basic Unix socket connection
- [x] Absolute path support
- [ ] Abstract socket support (Linux)
- [ ] Socket permission verification

## Phase 2: Tor Integration

### 2.1 Corsair IPC Protocol
- [x] Message format definition
- [x] Request/Response types
- [ ] Error handling improvements
- [ ] Connection state machine

### 2.2 Tor Connector
- [x] Basic Corsair communication
- [ ] Connection retry logic
- [ ] Circuit isolation options
- [ ] New identity request

### 2.3 Integration Testing
- [ ] Test with running Corsair daemon
- [ ] Test circuit isolation
- [ ] Test error conditions

## Phase 3: Windows Support

### 3.1 Named Pipe Connector
- [ ] Basic named pipe connection
- [ ] Pipe security attributes
- [ ] Async I/O on Windows

### 3.2 Platform Abstraction
- [ ] Unified socket path handling
- [ ] Cross-platform tests
- [ ] CI/CD for Windows

## Phase 4: Advanced Features

### 4.1 Transport Composition
- [ ] Chain multiple transports
- [ ] TCP over Tor
- [ ] Proper error propagation

### 4.2 Connection Management
- [ ] Optional connection pooling
- [ ] Health checking
- [ ] Automatic reconnection

### 4.3 Observability
- [ ] Tracing integration
- [ ] Metrics collection
- [ ] Debug logging

## Phase 5: Performance & Polish

### 5.1 Optimization
- [ ] Benchmark suite
- [ ] Memory usage optimization
- [ ] Connection establishment latency

### 5.2 Documentation
- [x] README.md
- [x] DESIGN.md
- [x] AGENTS.md
- [ ] API documentation (rustdoc)
- [ ] Usage examples

### 5.3 Testing
- [ ] Unit test coverage > 80%
- [ ] Integration test suite
- [ ] Fuzzing for URL parser

## Milestones

### v0.1.0 - Foundation
- TCP and Unix connectors
- Basic URL parsing
- Core types and traits

### v0.2.0 - Tor Support
- Corsair integration
- Binary IPC protocol
- Tor connector

### v0.3.0 - Windows Support
- Named pipe connector
- Cross-platform CI

### v0.4.0 - Composition
- Transport chaining
- Connection pooling
- Observability

### v1.0.0 - Stable Release
- Full documentation
- Comprehensive tests
- API stability guarantee

## Technical Debt

1. **URL Parser**: Current regex-based parser should be replaced with proper parser
2. **Error Messages**: Need more context in error messages
3. **Feature Flags**: Need to verify minimal builds work correctly

## Dependencies to Track

| Crate | Version | Purpose |
|-------|---------|---------|
| tokio | 1.x | Async runtime |
| hyper | 1.x | HTTP client |
| serde | 1.x | Serialization |
| bincode | 1.x | Binary serialization |
| url | 2.x | URL parsing |
| thiserror | 1.x | Error handling |

## Open Questions

1. Should we support HTTP/2 over Unix sockets?
2. Should connection pooling be built-in or external?
3. How to handle proxy chains with authentication?

## Contributing

See AGENTS.md for AI assistant guidelines and coding standards.
