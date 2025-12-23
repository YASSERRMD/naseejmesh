# NaseejMesh API Gateway

A high-performance API Gateway built in Rust with zero-downtime configuration updates.

## Features

- **High-Performance HTTP Core**: Built on Hyper 1.0 with automatic HTTP/1.1 and HTTP/2 negotiation
- **Embedded Configuration**: SurrealDB with RocksDB backend - no external database dependencies
- **Hot Reload**: Live Query subscriptions for real-time configuration updates
- **Zero-Lock Reads**: ArcSwap pattern for wait-free routing table access
- **Memory Safe**: Zero-copy body handling with strict size limits

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    NaseejMesh Gateway                        │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌──────────────┐    ┌───────────────┐  │
│  │   Hyper     │    │   ArcSwap    │    │  SurrealDB    │  │
│  │  HTTP/1+2   │◄──►│  RouterMap   │◄───│  Live Query   │  │
│  │   Server    │    │  (wait-free) │    │   Watcher     │  │
│  └─────────────┘    └──────────────┘    └───────────────┘  │
│         ▲                                       ▲          │
│         │                                       │          │
│    TCP Accept                              RocksDB         │
│      Loop                                  Storage         │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.75+ (for workspace inheritance)
- macOS/Linux (RocksDB dependencies)

### Build

```bash
# Development build
cargo build

# Optimized release build
cargo build --release
```

### Run

```bash
# Start with default configuration
cargo run --release --bin naseejmesh-gateway

# Development mode with seeded routes
DEV_MODE=1 cargo run --bin naseejmesh-gateway

# Custom port
PORT=3000 cargo run --bin naseejmesh-gateway
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8080` | HTTP listen port |
| `HOST` | `0.0.0.0` | HTTP listen address |
| `DEV_MODE` | unset | Enable development mode (seeds default routes) |
| `SURREAL_EMBEDDED` | `true` | Use embedded RocksDB (true) or remote SurrealDB (false) |
| `SURREAL_PATH` | `./data/gateway.db` | Path for embedded database |
| `SURREAL_URL` | `ws://localhost:8000` | URL for remote SurrealDB |
| `SURREAL_USER` | `root` | Username for remote SurrealDB |
| `SURREAL_PASS` | `root` | Password for remote SurrealDB |

## API Endpoints

### Gateway Internal

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/_gateway/health` | GET | Health check (always 200 OK) |
| `/_gateway/ready` | GET | Readiness check (200 if routes loaded, 503 otherwise) |

### Routing

Routes are configured dynamically via SurrealDB. Example route structure:

```json
{
  "id": "user-service",
  "path": "/api/users",
  "upstream": "http://user-service:8080",
  "weight": 100,
  "active": true,
  "methods": ["GET", "POST", "PUT", "DELETE"],
  "timeout_ms": 30000,
  "description": "User service API"
}
```

## Testing with Docker SurrealDB

For development and testing, you can use SurrealDB in Docker:

```bash
# Start SurrealDB
docker run -d --name surrealdb -p 8000:8000 \
  surrealdb/surrealdb:latest start \
  --user root --pass root

# Configure gateway to use remote SurrealDB
SURREAL_EMBEDDED=false \
SURREAL_URL=ws://localhost:8000 \
cargo run --bin naseejmesh-gateway
```

## Project Structure

```
naseejmesh/
├── Cargo.toml              # Workspace root
├── src/
│   └── bin/
│       └── server.rs       # Main entry point
├── crates/
│   ├── gateway-core/       # HTTP logic (isolated from DB)
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── body.rs     # Secure body handling
│   │       ├── config.rs   # Route types
│   │       ├── error.rs    # Error classification
│   │       ├── executor.rs # HTTP/2 executor
│   │       ├── handler.rs  # Request handler
│   │       └── router.rs   # Path matching
│   └── surreal-config/     # Database layer
│       └── src/
│           ├── lib.rs
│           ├── db.rs       # DB initialization
│           ├── error.rs    # Config errors
│           ├── schema.rs   # CRUD operations
│           └── watcher.rs  # Live Query watcher
└── docker-compose.yml      # Development environment
```

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Throughput | 100k+ RPS | On 16 vCPU commodity hardware |
| Latency (p99) | < 1ms | Gateway "tax" only |
| Memory | Bounded | Zero-copy body handling, limited request sizes |
| Config Reload | < 10ms | Live Query + ArcSwap atomic swap |

## Phase 1 Scope

This is Phase 1 of the gateway implementation, focusing on:

- ✅ Hyper 1.0 HTTP server with auto HTTP/1+2
- ✅ Embedded SurrealDB with RocksDB
- ✅ Live Query configuration subscription
- ✅ ArcSwap wait-free routing
- ✅ Secure body handling with size limits
- ✅ Route matching (exact, prefix, wildcard)

### Phase 2 (Planned)

- Upstream HTTP client with connection pooling
- Load balancing algorithms
- Health checks for upstreams
- Distributed tracing integration

## License

MIT License - see [LICENSE](LICENSE) for details.
