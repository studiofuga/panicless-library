# Docker Deployment Guide

Complete guide for running Panicless Library with Docker.

## Quick Start

```bash
# 1. Clone and enter directory
git clone <repository-url>
cd panicless-library

# 2. Configure environment (optional)
cp .env.example .env
# Edit .env to set JWT_SECRET

# 3. Start all services
docker-compose up -d

# 4. Access the app
open http://localhost:3000
```

That's it! All services are running:
- Frontend: http://localhost:3000
- Backend API: http://localhost:8080
- PostgreSQL: localhost:5432

## Architecture

```
┌─────────────────────────────────────────────┐
│              Docker Network                  │
│                                              │
│  ┌──────────┐   ┌──────────┐   ┌─────────┐ │
│  │ Frontend │   │ Backend  │   │ Postgres│ │
│  │  (nginx) │   │  (Rust)  │   │   DB    │ │
│  │          │   │          │   │         │ │
│  │ Port 80  │   │ Port 8080│   │Port 5432│ │
│  └────┬─────┘   └────┬─────┘   └────┬────┘ │
│       │              │               │      │
└───────┼──────────────┼───────────────┼──────┘
        │              │               │
    :3000          :8080           :5432
     (host)        (host)          (host)
```

## Services

### PostgreSQL
- **Image**: postgres:16-alpine
- **Port**: 5432
- **User**: panicless
- **Password**: panicless_dev (change in production!)
- **Database**: panicless_library
- **Volumes**: postgres_data (persistent storage)
- **Migrations**: Auto-run on first start

### Backend (Rust)
- **Build**: Multi-stage Dockerfile
- **Port**: 8080
- **Health**: http://localhost:8080/health
- **Depends**: postgres (waits for healthy)
- **Environment**:
  - DATABASE_URL
  - JWT_SECRET
  - CORS_ALLOWED_ORIGINS

### Frontend (Vue.js + nginx)
- **Build**: Multi-stage Dockerfile (Node build + nginx serve)
- **Port**: 3000 (mapped to nginx port 80)
- **Health**: http://localhost:3000/health
- **Depends**: backend (waits for healthy)
- **Config**: nginx.conf with Vue Router support

## Common Commands

### Using Make (Recommended)

```bash
make help           # Show all commands
make build          # Build all images
make up             # Start all services
make down           # Stop all services
make restart        # Restart all services
make logs           # View all logs
make logs-backend   # View backend logs
make logs-frontend  # View frontend logs
make clean          # Remove everything (including volumes)
make shell-postgres # Open PostgreSQL shell
```

### Using docker-compose Directly

```bash
# Start services
docker-compose up -d

# Stop services
docker-compose down

# View logs
docker-compose logs -f

# Check status
docker-compose ps

# Restart a service
docker-compose restart backend

# Rebuild a service
docker-compose up -d --build backend

# Remove everything
docker-compose down -v
```

## Environment Variables

### Development (.env)

```bash
JWT_SECRET=dev-secret-change-in-production
POSTGRES_USER=panicless
POSTGRES_PASSWORD=panicless_dev
POSTGRES_DB=panicless_library
VITE_API_BASE_URL=http://localhost:8080
```

### Production (.env)

```bash
# Generate secure secret:
# openssl rand -base64 64
JWT_SECRET=<YOUR-64-CHAR-SECURE-RANDOM-STRING>

POSTGRES_USER=panicless
POSTGRES_PASSWORD=<SECURE-PASSWORD>
POSTGRES_DB=panicless_library

# Use your domain
VITE_API_BASE_URL=https://api.yourdomain.com
```

Also update in docker-compose.yml:
- `CORS_ALLOWED_ORIGINS` to your domain
- `VITE_API_BASE_URL` build arg to your API URL

## Build Process

### Backend Build

```dockerfile
# Stage 1: Build Rust binary
FROM rust:1.75-slim
COPY . .
RUN cargo build --release

# Stage 2: Runtime (slim Debian)
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/panicless-backend .
CMD ["./panicless-backend"]
```

Final image: ~100MB

### Frontend Build

```dockerfile
# Stage 1: Build Vue.js app
FROM node:20-alpine
COPY . .
RUN npm ci && npm run build

# Stage 2: Serve with nginx
FROM nginx:1.25-alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
```

Final image: ~50MB

## Healthchecks

All services have health checks:

- **PostgreSQL**: `pg_isready` every 10s
- **Backend**: HTTP GET /health every 30s
- **Frontend**: HTTP GET /health every 30s

Services won't be marked as healthy until they respond successfully.

## Networking

All services communicate on the `panicless-network` bridge network:

- Frontend → Backend: `http://backend:8080`
- Backend → Postgres: `postgres://panicless@postgres:5432/panicless_library`

Host access:
- Frontend: http://localhost:3000
- Backend: http://localhost:8080
- Postgres: localhost:5432

## Data Persistence

### Volumes

- `postgres_data`: PostgreSQL data directory
  - Location: `/var/lib/postgresql/data` (in container)
  - Persists across restarts

### Backups

```bash
# Manual backup
docker-compose exec -T postgres pg_dump -U panicless panicless_library > backup.sql

# Compressed backup
docker-compose exec -T postgres pg_dump -U panicless panicless_library | gzip > backup.sql.gz

# Restore
docker-compose exec -T postgres psql -U panicless -d panicless_library < backup.sql
```

### Automatic Backups

```bash
# Create backup script
cat > backup.sh << 'EOF'
#!/bin/bash
BACKUP_DIR="./backups"
mkdir -p $BACKUP_DIR
docker-compose exec -T postgres pg_dump -U panicless panicless_library | \
  gzip > $BACKUP_DIR/backup-$(date +%Y%m%d-%H%M%S).sql.gz
# Keep only last 30 days
find $BACKUP_DIR -name "backup-*.sql.gz" -mtime +30 -delete
EOF

chmod +x backup.sh

# Add to crontab (daily at 2 AM)
(crontab -l 2>/dev/null; echo "0 2 * * * /path/to/panicless-library/backup.sh") | crontab -
```

## Production Deployment

### 1. Server Setup

```bash
# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

### 2. Clone and Configure

```bash
git clone <repository-url>
cd panicless-library

# Configure production environment
cp .env.example .env
nano .env  # Set secure JWT_SECRET and passwords

# Update CORS in docker-compose.yml
nano docker-compose.yml
# Change CORS_ALLOWED_ORIGINS to your domain
```

### 3. Start Services

```bash
docker-compose up -d --build
```

### 4. Reverse Proxy (Optional)

If you want to use a domain name with SSL:

```nginx
# /etc/nginx/sites-available/panicless
server {
    listen 80;
    server_name yourdomain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}

server {
    listen 80;
    server_name api.yourdomain.com;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

Enable and get SSL:
```bash
sudo ln -s /etc/nginx/sites-available/panicless /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
sudo certbot --nginx -d yourdomain.com -d api.yourdomain.com
```

## Monitoring

### View Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f backend

# Last 100 lines
docker-compose logs --tail=100 backend

# Since timestamp
docker-compose logs --since 2025-01-06T10:00:00 backend
```

### Resource Usage

```bash
# Container stats
docker stats

# Disk usage
docker system df

# Service status
docker-compose ps
```

## Troubleshooting

### Services won't start

```bash
# Check logs
docker-compose logs

# Check disk space
df -h
docker system df

# Rebuild from scratch
docker-compose down -v
docker-compose up -d --build
```

### Permission issues

```bash
# Backend container runs as user 1000
# Ensure files are accessible
sudo chown -R 1000:1000 backend/
```

### Database connection errors

```bash
# Check postgres is healthy
docker-compose ps postgres

# Check logs
docker-compose logs postgres

# Manual connection test
docker-compose exec postgres psql -U panicless -d panicless_library -c "SELECT 1"
```

### Port conflicts

```bash
# Check what's using ports
sudo lsof -i :3000
sudo lsof -i :8080
sudo lsof -i :5432

# Change ports in docker-compose.yml
# Example: "3001:80" instead of "3000:80"
```

## Updating

### Pull Latest Changes

```bash
git pull origin main
docker-compose up -d --build
```

### Rebuild Single Service

```bash
# Just backend
docker-compose up -d --build backend

# Just frontend
docker-compose up -d --build frontend
```

## Cleanup

### Remove Containers

```bash
# Stop and remove containers
docker-compose down
```

### Remove Volumes (WARNING: Deletes data!)

```bash
# Remove everything including data
docker-compose down -v
```

### Remove Images

```bash
# Remove project images
docker-compose down --rmi all

# Clean up unused images
docker image prune -a
```

## Performance Tuning

### PostgreSQL

Edit docker-compose.yml:
```yaml
postgres:
  command:
    - postgres
    - -c
    - max_connections=100
    - -c
    - shared_buffers=256MB
    - -c
    - effective_cache_size=1GB
```

### Backend

Adjust healthcheck timing:
```yaml
healthcheck:
  interval: 60s  # Check every minute instead of 30s
  timeout: 5s
  retries: 3
```

### Resource Limits

```yaml
backend:
  deploy:
    resources:
      limits:
        cpus: '1.0'
        memory: 512M
      reservations:
        cpus: '0.5'
        memory: 256M
```

## Support

- Main README: [README.md](README.md)
- Backend docs: [backend/README.md](backend/README.md)
- Database docs: [database/README.md](database/README.md)
- MCP Server: [mcp-server/README.md](mcp-server/README.md)

## License

Copyright (c) 2025 Federico Fuga
