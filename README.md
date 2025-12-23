# 🕸️ NaseejMesh

**AI-Driven API Gateway for Enterprise Integration**

A high-performance, multi-protocol API Gateway built in Rust. NaseejMesh uses AI to generate and deploy integration logic through natural language, with enterprise security and distributed clustering built-in.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## ✨ Features

### Multi-Protocol Support
- **HTTP/1.1 & HTTP/2** - Auto-negotiation with Hyper 1.0
- **MQTT** - IoT sensor integration with topic routing
- **gRPC** - JSON ↔ Protobuf transcoding
- **SOAP** - XML ↔ JSON streaming conversion

### AI-Powered Configuration
- **Natural Language Routing** - Describe integrations in plain English
- **Rhai Scripting** - Safe embedded data transformations
- **Schema Learning** - Ingest OpenAPI specs for RAG
- **MCP Protocol** - JSON-RPC interface for AI clients

### Visual Control Plane
- **React Flow Canvas** - Node-based flow visualization
- **Dry Run Testing** - Test scripts before deployment
- **Real-time Streaming** - SSE for live updates

### Enterprise Security
- **Web Application Firewall** - SQL injection, XSS, path traversal detection
- **JWT Authentication** - HS256/RS256 with local caching
- **Token Bucket Rate Limiting** - Per-client with burst support
- **Usage Metering** - Async tracking without blocking requests

### Performance
- **Zero-Copy I/O** - Memory-safe without garbage collection
- **Live Configuration** - Hot reload via SurrealDB Live Query
- **Wait-Free Routing** - ArcSwap for lock-free updates
- **Sub-millisecond Latency** - Optimized for high throughput

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Naseej Console (UI)                        │
│         React Flow  │  AI Chat  │  Route Management             │
└────────────────────────────┬────────────────────────────────────┘
                             │ REST/SSE
┌────────────────────────────┴────────────────────────────────────┐
│                    naseej-console (Axum API)                    │
│      /api/simulate  │  /api/validate  │  /api/chat              │
└────────────────────────────┬────────────────────────────────────┘
                             │
┌────────────────────────────┴────────────────────────────────────┐
│                     naseejmesh-gateway                          │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────────┐ │
│  │   Security   │  │   Routing    │  │      SurrealDB         │ │
│  │  WAF + Auth  │  │   ArcSwap    │  │   Config + Live Query  │ │
│  └──────────────┘  └──────────────┘  └────────────────────────┘ │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │        Protocol Adapters (HTTP │ MQTT │ gRPC │ SOAP)     │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📦 Crates

| Crate | Description |
|-------|-------------|
| `gateway-core` | HTTP routing, body handling, Rhai transforms |
| `surreal-config` | SurrealDB integration, Live Query watcher |
| `protocol-adapters` | MQTT, gRPC, SOAP, OpenTelemetry |
| `cognitive-core` | AI Architect, MCP server, VectorStore |
| `naseej-console` | Axum API server for frontend |
| `naseej-security` | WAF, JWT auth, rate limiting, metering |
| `naseejmesh-server` | Main gateway binary |

---

## 🚀 Quick Start

### Prerequisites

- **Rust 1.75+**
- **Node.js 18+** (for console)

### Build & Run

```bash
# Build all crates
cargo build --release

# Start gateway with embedded SurrealDB
DEV_MODE=1 cargo run --release --bin naseejmesh-gateway

# Start console API server (port 3001)
cargo run --bin naseej-console
```

### Start Console UI

```bash
cd console
npm install
npm run dev
# Open http://localhost:3000
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8080` | Gateway HTTP port |
| `DEV_MODE` | unset | Seed default routes |
| `SURREAL_EMBEDDED` | `true` | Use embedded DB |
| `SURREAL_URL` | - | Remote SurrealDB URL |

---

## 🔧 API Reference

### Gateway Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/_gateway/health` | GET | Liveness probe |
| `/_gateway/ready` | GET | Readiness probe |

### Console API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/routes` | GET | List all routes |
| `/api/routes` | POST | Create route |
| `/api/simulate` | POST | Dry-run transformation |
| `/api/validate` | POST | Validate Rhai script |
| `/api/chat` | POST | Chat with AI Architect |
| `/api/chat/stream` | GET | SSE streaming |
| `/api/state` | GET | Gateway state |

---

## 📝 Rhai Scripting

Built-in transformation functions:

```rhai
// JSON handling
let data = parse_json(input);
data["processed"] = true;
output = to_json(data);

// Temperature conversion
data["temp_f"] = celsius_to_fahrenheit(data["temp"]);

// XML wrapping
output = wrap_xml("temperature", "25");

// Utilities
let id = uuid();
let ts = timestamp_ms();
let iso = now_iso();

// Logging
log("Processing request");
```

---

## 🛡️ Security Features

### WAF (Web Application Firewall)

Detects and blocks:
- SQL Injection (`SELECT`, `UNION`, `DROP`)
- Cross-Site Scripting (`<script>`, `javascript:`)
- Path Traversal (`../`, `/etc/passwd`)
- Command Injection (`|`, `;`, backticks)

### JWT Authentication

```rust
// Validate tokens with caching
let claims = validator.validate(token).await?;

// Check scopes
if JwtValidator::has_scope(&claims, "write:routes") {
    // Authorized
}
```

### Rate Limiting

```rust
// Token bucket per client
let result = limiter.check("client_id");
if !result.allowed {
    // Return 429 with retry_after_ms
}
```

---

## 🧪 Testing

```bash
# Run all tests
cargo test --all

# Test specific crate
cargo test -p naseej-security
cargo test -p cognitive-core
```

---

## 📄 License

MIT License - see [LICENSE](LICENSE)

---

## 🤝 Contributing

Contributions welcome! Please read our contributing guidelines before submitting PRs.
