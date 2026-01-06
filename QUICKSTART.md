# Quick Start Guide

Get Panicless Library running in **under 2 minutes**!

## Prerequisites

- Docker and Docker Compose installed ([Get Docker](https://docs.docker.com/get-docker/))

## 🚀 Start in 3 Commands

```bash
# 1. Clone
git clone <repository-url>
cd panicless-library

# 2. Start
docker-compose up -d

# 3. Wait for services (30-60 seconds for first build)
docker-compose logs -f
# Press Ctrl+C when you see "MCP Server ready"
```

## 🌐 Access the App

**Frontend**: http://localhost:3000
**Backend API**: http://localhost:8080
**Health Check**: http://localhost:8080/health

## 👤 First Steps

1. Open http://localhost:3000
2. Click **Register** and create an account
3. Add your first book
4. Start tracking your reading!

## 🛠️ Useful Commands

```bash
# View logs
docker-compose logs -f

# Stop
docker-compose down

# Restart
docker-compose restart

# Remove everything (including data)
docker-compose down -v
```

## 📖 Next Steps

- Read [README.md](README.md) for full documentation
- Read [DOCKER.md](DOCKER.md) for Docker details
- Configure [MCP Server](mcp-server/README.md) for AI integration
- Check [API docs](backend/README.md)

## 🧪 Test Everything Works

```bash
./test-docker.sh
```

## 🆘 Something Wrong?

```bash
# Check service status
docker-compose ps

# View all logs
docker-compose logs

# Rebuild from scratch
docker-compose down -v
docker-compose up -d --build
```

## 🎯 That's It!

You now have a full-stack library management system running locally with:
- ✅ PostgreSQL database
- ✅ Rust backend API
- ✅ Vue.js frontend
- ✅ User authentication
- ✅ Book catalog
- ✅ Reading tracker
- ✅ Statistics dashboard

Enjoy tracking your reading! 📚
