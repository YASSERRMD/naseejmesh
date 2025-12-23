# 🕸️ NaseejMesh

**AI-Driven API Gateway for Enterprise Integration**

A high-performance, multi-protocol API Gateway built in Rust that uses AI to generate and deploy integration logic through natural language.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-passing-brightgreen)]()

## ✨ Features

### 🚀 Zero-Copy Nucleus (Phase 1)
- **Hyper 1.0** HTTP/1.1 & HTTP/2 auto-negotiation
- **Embedded SurrealDB** with RocksDB backend
- **Live Query** subscriptions for hot reload
- **ArcSwap** wait-free routing table updates

### 🔌 Polyglot Protocol Fabric (Phase 2)
- **MQTT** - IoT sensor integration
- **gRPC** - JSON ↔ Protobuf transcoding
- **SOAP** - XML ↔ JSON streaming conversion
- **OpenTelemetry** distributed tracing

### 🧠 Cognitive Control Plane (Phase 3)
- **AI Architect** - Natural language route configuration
- **Rhai Scripting** - Safe embedded transformations
- **Schema Ingestion** - OpenAPI parsing for RAG
- **MCP Protocol** - JSON-RPC for external AI clients

### 🎨 Visual Control Plane (Phase 4)
- **React Flow** - Node-based flow visualization
- **Mantine UI** - RTL-ready component library
- **Dry Run API** - Test transformations before deploy
- **SSE Streaming** - Real-time AI responses

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Naseej Console                           │
│  ┌─────────────────┐  ┌─────────────────┐  ┌───────────────┐   │
│  │   React Flow    │  │    AI Chat      │  │   Route Mgmt  │   │
│  │   Canvas        │  │    Panel        │  │   Dashboard   │   │
│  └────────┬────────┘  └────────┬────────┘  └───────┬───────┘   │
└───────────┴───────────────────┬────────────────────┴───────────┘
                                │ REST/SSE
┌───────────────────────────────┴────────────────────────────────┐
│                     naseej-console (Axum)                       │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────────────┐ │
│  │  Simulate   │  │   Validate   │  │      AI Architect      │ │
│  │    API      │  │     API      │  │   (Rig + VectorStore)  │ │
│  └──────┬──────┘  └──────┬───────┘  └───────────┬────────────┘ │
└─────────┴────────────────┴──────────────────────┴──────────────┘
                                │
┌───────────────────────────────┴────────────────────────────────┐
│                     naseejmesh-gateway                          │
│  ┌─────────────┐  ┌──────────────┐  ┌────────────────────────┐ │
│  │   Hyper     │  │   ArcSwap    │  │      SurrealDB         │ │
│  │  HTTP/1+2   │◄─│  RouterMap   │◄─│   Live Query + KV      │ │
│  └──────┬──────┘  └──────────────┘  └────────────────────────┘ │
│  ┌──────┴──────────────────────────────────────────────────┐   │
│  │          Protocol Adapters (MQTT | gRPC | SOAP)          │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## 📦 Project Structure

```
naseejmesh/
├── crates/
│   ├── gateway-core/        # HTTP, routing, Rhai transforms
│   ├── surreal-config/      # SurrealDB, Live Query watcher
│   ├── protocol-adapters/   # MQTT, gRPC, SOAP, OpenTelemetry
│   ├── cognitive-core/      # AI Architect, MCP, VectorStore
│   ├── naseej-console/      # Axum API server for frontend
│   └── naseejmesh-server/   # Main gateway binary
├── console/                 # Next.js React Flow dashboard
└── Cargo.toml               # Workspace root
```

## 🚀 Quick Start

### Prerequisites

- **Rust 1.75+**
- **Node.js 18+** (for console)

### Build & Run Gateway

```bash
# Build everything
cargo build --release

# Start gateway (embedded SurrealDB)
DEV_MODE=1 cargo run --release --bin naseejmesh-gateway

# Start API server for console
cargo run --bin naseej-console
```

### Start Console

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

## 🧪 Testing

```bash
# Run all tests
cargo test --all

# Specific crate
cargo test -p cognitive-core
cargo test -p gateway-core
```

## 🔧 API Endpoints

### Gateway (`localhost:8080`)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/_gateway/health` | GET | Health check |
| `/_gateway/ready` | GET | Readiness probe |

### Console API (`localhost:3001`)

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/routes` | GET/POST | Manage routes |
| `/api/simulate` | POST | Dry-run transformation |
| `/api/validate` | POST | Validate Rhai script |
| `/api/chat` | POST | Chat with AI Architect |
| `/api/chat/stream` | GET | SSE streaming |

## 📝 Example: Create Integration

Use the AI Chat to create integrations naturally:

```
"Create a route that listens on MQTT topic 'sensors/temp' 
and forwards to http://api.example.com/readings, 
converting Celsius to Fahrenheit."
```

The AI generates:
1. **Route config** (MQTT → HTTP)
2. **Rhai script** for transformation
3. **Deploys** via Live Query

## 🛠️ Rhai Scripting

Built-in helper functions:

```rhai
// JSON handling
let data = parse_json(input);
output = to_json(data);

// Temperature conversion
data["temp_f"] = celsius_to_fahrenheit(data["temp"]);

// Utilities
let id = uuid();
let ts = timestamp_ms();
let iso = now_iso();

// XML wrapping
output = wrap_xml("temperature", "25");
```

## 📄 License

MIT License - see [LICENSE](LICENSE)
