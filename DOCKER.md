# Naseej Mesh - Docker Deployment

## Quick Start

```bash
# Start all services (SurrealDB, Backend, Frontend)
docker-compose up -d

# View logs
docker-compose logs -f

# Stop all services
docker-compose down
```

## Services

| Service | Port | Description |
|---------|------|-------------|
| **Frontend** | 3000 | Next.js Console UI |
| **Backend** | 3001 | Rust API Server |
| **SurrealDB** | 8000 | Database |

## Access

- **Console**: http://localhost:3000
- **API**: http://localhost:3001
- **Database**: http://localhost:8000

## Environment Variables

### Backend
```env
SURREAL_URL=ws://surrealdb:8000
SURREAL_USER=root
SURREAL_PASS=naseej123
SURREAL_NS=naseej
SURREAL_DB=console
```

### Frontend
```env
NEXT_PUBLIC_API_URL=http://localhost:3001
```

## Data Persistence

SurrealDB data is persisted in a Docker volume:
```bash
# View volumes
docker volume ls

# Backup data
docker run --rm -v naseejmesh_surrealdb_data:/data -v $(pwd):/backup alpine tar czf /backup/surrealdb-backup.tar.gz /data
```

## Development

For local development without Docker:

```bash
# Terminal 1: Start SurrealDB
surreal start --log trace --user root --pass naseej123 file:naseej.db

# Terminal 2: Start Backend
cargo run -p naseej-console

# Terminal 3: Start Frontend
cd naseej-console && npm run dev
```
