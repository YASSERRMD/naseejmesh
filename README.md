<p align="center">
  <img src="assets/logo.png" alt="NaseejMesh Logo" width="180" />
</p>

<h1 align="center">NaseejMesh</h1>

<p align="center">
  <strong>AI-Driven API Gateway for Enterprise Integration</strong>
</p>

<p align="center">
  A high-performance, multi-protocol API Gateway built in Rust. NaseejMesh uses AI to generate and deploy integration logic through natural language, with enterprise security and distributed clustering built-in.
</p>

<p align="center">
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/rust-1.75%2B-orange" alt="Rust"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License: MIT"></a>
</p>

---

## Features

### Multi-Protocol Support
- **HTTP/1.1 and HTTP/2** - Auto-negotiation with Hyper 1.0
- **MQTT** - IoT sensor integration with topic routing
- **gRPC** - JSON to Protobuf transcoding
- **SOAP** - XML to JSON streaming conversion

### AI-Powered Configuration
- **Natural Language Routing** - Describe integrations in plain English
- **Rhai Scripting** - Safe embedded data transformations
- **Schema Learning** - Ingest OpenAPI specs for RAG-based recommendations
- **MCP Protocol** - JSON-RPC interface for AI clients
- **Cohere Integration** - LLM chat and embeddings via Cohere API

### Visual Control Plane
- **React Flow Canvas** - Node-based flow visualization
- **Dry Run Testing** - Test scripts before deployment
- **Real-time Streaming** - SSE for live updates
- **Admin Console** - User, role, and API key management

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

## Architecture

```
+-------------------------------------------------------------------+
|                      Naseej Console (UI)                          |
|         React Flow  |  AI Chat  |  Route Management               |
+------------------------------+------------------------------------+
                               | REST/SSE
+------------------------------+------------------------------------+
|                    naseej-console (Axum API)                      |
|      /api/simulate  |  /api/validate  |  /api/chat                |
+------------------------------+------------------------------------+
                               |
+------------------------------+------------------------------------+
|                     naseejmesh-gateway                            |
|  +---------------+  +---------------+  +------------------------+ |
|  |   Security    |  |    Routing    |  |       SurrealDB        | |
|  |   WAF + Auth  |  |    ArcSwap    |  |  Config + Live Query   | |
|  +---------------+  +---------------+  +------------------------+ |
|  +--------------------------------------------------------------+ |
|  |        Protocol Adapters (HTTP | MQTT | gRPC | SOAP)         | |
|  +--------------------------------------------------------------+ |
+-------------------------------------------------------------------+
```

---

## Crates

| Crate | Description |
|-------|-------------|
| `gateway-core` | HTTP routing, body handling, Rhai transforms |
| `surreal-config` | SurrealDB integration, Live Query watcher, vector store |
| `protocol-adapters` | MQTT, gRPC, SOAP, OpenTelemetry |
| `cognitive-core` | AI Architect, MCP server, VectorStore, Cohere LLM/Embeddings |
| `naseej-console` | Axum API server for frontend |
| `naseej-security` | WAF, JWT auth, rate limiting, metering |
| `naseej-cli` | CLI for schema ingestion and management |
| `naseejmesh-server` | Main gateway binary |

---

## Quick Start

### Prerequisites

- **Rust 1.75+**
- **Node.js 18+** (for console)
- **Docker** (optional, for containerized deployment)

### Build and Run

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
cd naseej-console
npm install
npm run dev
# Open http://localhost:3000
```

### Docker Deployment

```bash
docker-compose up -d
```

### CLI Usage

```bash
# Learn from an OpenAPI specification
naseej learn petstore.yaml --source petstore

# Search for endpoints
naseej search "get user by id"

# Check system status
naseej status
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8080` | Gateway HTTP port |
| `DEV_MODE` | unset | Seed default routes |
| `SURREAL_EMBEDDED` | `true` | Use embedded DB |
| `SURREAL_URL` | - | Remote SurrealDB URL |
| `COHERE_API_KEY` | - | Cohere API key for AI features |

---

## API Reference

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
| `/api/admin/users` | GET/POST | User management |
| `/api/admin/roles` | GET/POST | Role management |
| `/api/admin/keys` | GET/POST/DELETE | API key management |

### MCP Protocol

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/mcp/tools/list` | GET | List available tools |
| `/mcp/prompts/list` | GET | List available prompts |
| `/mcp/stream` | POST | SSE chat stream |

---

## Rhai Scripting

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

## Security Features

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

## AI Features

### Cohere Integration

NaseejMesh integrates with Cohere for:
- **Embeddings**: `embed-english-v3.0` model with 1024 dimensions
- **Chat**: `command-r-plus` model for intelligent route design
- **Tool Calling**: AI can deploy routes, search schemas, and validate scripts

### AI Architect

The AI Architect helps design integration routes:

```
User: Create a route that forwards /api/users to the user service

AI: I'll create that route for you.

Actions taken:
- deploy_route: Route configured: GET /api/users -> http://user-service:8080
```

---

## Testing

```bash
# Run all tests
cargo test --all

# Test specific crate
cargo test -p naseej-security
cargo test -p cognitive-core
```

---

## Project Structure

```
naseejmesh/
├── crates/
│   ├── gateway-core/        # Core routing and transforms
│   ├── surreal-config/      # Database and configuration
│   ├── protocol-adapters/   # Multi-protocol support
│   ├── cognitive-core/      # AI and MCP server
│   ├── naseej-console/      # API server
│   ├── naseej-security/     # Security features
│   ├── naseej-cli/          # CLI tool
│   └── naseejmesh-server/   # Main binary
├── naseej-console/          # React frontend
├── assets/                  # Logo and images
└── docker-compose.yml       # Container deployment
```

---

## License

MIT License - see [LICENSE](LICENSE)

---

## Contributing

Contributions welcome! Please read our contributing guidelines before submitting PRs.
