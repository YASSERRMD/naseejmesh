# NaseejMesh API Gateway

A high-performance, AI-driven API Gateway built in Rust with zero-downtime configuration updates and multi-protocol support.

## Features

### Phase 1: Zero-Copy Nucleus
- **High-Performance HTTP Core**: Built on Hyper 1.0 with automatic HTTP/1.1 and HTTP/2 negotiation
- **Embedded Configuration**: SurrealDB with RocksDB backend - no external database dependencies
- **Hot Reload**: Live Query subscriptions for real-time configuration updates
- **Zero-Lock Reads**: ArcSwap pattern for wait-free routing table access
- **Memory Safe**: Zero-copy body handling with strict size limits

### Phase 2: Polyglot Protocol Fabric
- **MQTT Adapter**: IoT device integration with topic-based routing
- **gRPC Adapter**: Dynamic JSON ↔ Protobuf transcoding with prost-reflect
- **SOAP Adapter**: Streaming XML-to-JSON conversion with quick-xml
- **OpenTelemetry**: Distributed tracing across all protocols

### Phase 3: Cognitive Control Plane
- **AI Architect**: Natural language integration design using Rig
- **Rhai Scripting**: Safe embedded scripting for data transformations
- **Schema Ingestion**: OpenAPI parsing for RAG knowledge base
- **MCP Protocol**: JSON-RPC interface for external AI tools

## Architecture

```
┌────────────────────────────────────────────────────────────────────┐
│                       NaseejMesh Gateway                            │
├────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌──────────────┐    ┌────────────────────────┐│
│  │   Hyper     │    │   ArcSwap    │    │      SurrealDB         ││
│  │  HTTP/1+2   │◄──►│  RouterMap   │◄───│  Live Query + Vectors  ││
│  │   Server    │    │  (wait-free) │    │                        ││
│  └─────────────┘    └──────────────┘    └────────────────────────┘│
│         ▲                                         ▲                │
│  ┌──────┴──────┐                          ┌───────┴───────┐       │
│  │  Protocol   │                          │   Cognitive   │       │
│  │  Adapters   │                          │     Core      │       │
│  │ MQTT|gRPC|  │                          │  AI Architect │       │
│  │   SOAP      │                          │  Rhai Engine  │       │
│  └─────────────┘                          └───────────────┘       │
└────────────────────────────────────────────────────────────────────┘
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

## Crates

| Crate | Description |
|-------|-------------|
| `gateway-core` | HTTP logic, routing, body handling |
| `surreal-config` | SurrealDB integration, Live Query watcher |
| `protocol-adapters` | MQTT, gRPC, SOAP adapters with OpenTelemetry |
| `cognitive-core` | AI Architect, Rhai scripting, MCP protocol |
| `naseejmesh-server` | Main binary entry point |

## API Endpoints

### Gateway Internal

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/_gateway/health` | GET | Health check (always 200 OK) |
| `/_gateway/ready` | GET | Readiness check (200 if routes loaded, 503 otherwise) |

### Routing

Routes are configured dynamically via SurrealDB:

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

## AI Architect (Phase 3)

Use natural language to create integrations:

```
"Create a route that listens on MQTT topic 'sensors/+' and 
forwards the payload to http://api.example.com/readings, 
transforming the timestamp to UTC."
```

### Available Tools

- **deploy_route**: Deploy integration routes
- **lookup_schema**: Search API knowledge base
- **validate_rhai**: Validate transformation scripts

### Rhai Scripting

```rhai
// Transform temperature from Celsius to Fahrenheit
payload["temp_f"] = payload["temp"] * 9 / 5 + 32;
payload["converted_at"] = now_utc();
```

Built-in functions: `parse_json()`, `to_json()`, `uuid()`, `timestamp()`, `now_utc()`, `log_info()`

## Testing

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p cognitive-core
cargo test -p protocol-adapters
cargo test -p gateway-core
cargo test -p surreal-config
```

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Throughput | 100k+ RPS | On 16 vCPU commodity hardware |
| Latency (p99) | < 1ms | Gateway "tax" only |
| Memory | Bounded | Zero-copy body handling |
| Config Reload | < 10ms | Live Query + ArcSwap |

## License

MIT License - see [LICENSE](LICENSE) for details.
