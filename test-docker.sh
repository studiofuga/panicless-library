#!/bin/bash

# Test script for Docker deployment
# This script verifies that all services are running correctly

set -e

echo "🧪 Testing Panicless Library Docker Deployment"
echo "=============================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if docker-compose is running
echo "📦 Checking if services are running..."
if ! docker-compose ps | grep -q "Up"; then
    echo -e "${RED}❌ Services are not running. Start them with: docker-compose up -d${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Services are running${NC}"
echo ""

# Wait for services to be healthy
echo "⏳ Waiting for services to be healthy (max 120 seconds)..."
timeout=120
elapsed=0
while [ $elapsed -lt $timeout ]; do
    if docker-compose ps | grep -q "healthy"; then
        backend_healthy=$(docker-compose ps backend | grep -c "healthy" || echo "0")
        postgres_healthy=$(docker-compose ps postgres | grep -c "healthy" || echo "0")

        if [ "$backend_healthy" -eq "1" ] && [ "$postgres_healthy" -eq "1" ]; then
            echo -e "${GREEN}✅ All services are healthy${NC}"
            break
        fi
    fi
    sleep 5
    elapsed=$((elapsed + 5))
    echo -n "."
done
echo ""

if [ $elapsed -ge $timeout ]; then
    echo -e "${RED}❌ Timeout waiting for services to be healthy${NC}"
    echo "Check logs with: docker-compose logs"
    exit 1
fi

# Test PostgreSQL
echo "🗄️  Testing PostgreSQL..."
DB_NAME=${POSTGRES_DB:-panicless}
DB_USER=${POSTGRES_USER:-postgres}
if docker-compose exec -T postgres psql -U "$DB_USER" -d "$DB_NAME" -c "SELECT 1" > /dev/null 2>&1; then
    echo -e "${GREEN}✅ PostgreSQL is accessible${NC}"
else
    echo -e "${RED}❌ PostgreSQL connection failed${NC}"
    exit 1
fi
echo ""

# Test Backend Health
echo "🦀 Testing Backend API..."
if curl -f -s http://localhost:8080/health > /dev/null; then
    echo -e "${GREEN}✅ Backend health check passed${NC}"
else
    echo -e "${RED}❌ Backend health check failed${NC}"
    exit 1
fi
echo ""

# Test Backend API endpoints
echo "🔧 Testing Backend API endpoints..."

# Test register endpoint
echo "  Testing POST /api/auth/register..."
REGISTER_RESPONSE=$(curl -s -X POST http://localhost:8080/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser_'$(date +%s)'","email":"test'$(date +%s)'@example.com","password":"testpass123"}')

if echo "$REGISTER_RESPONSE" | grep -q "access_token"; then
    echo -e "${GREEN}  ✅ Register endpoint working${NC}"
    TOKEN=$(echo "$REGISTER_RESPONSE" | grep -o '"access_token":"[^"]*' | cut -d'"' -f4)
else
    echo -e "${YELLOW}  ⚠️  Register endpoint response unexpected (might be duplicate user)${NC}"
fi
echo ""

# Test Frontend (served from backend)
echo "🎨 Testing Frontend (served from backend)..."
if curl -s http://localhost:8080 | grep -q "<!DOCTYPE html>"; then
    echo -e "${GREEN}✅ Frontend is serving HTML from backend${NC}"
else
    echo -e "${RED}❌ Frontend not serving HTML correctly${NC}"
    exit 1
fi
echo ""

# Test database tables exist
echo "📊 Testing database schema..."
DB_NAME=${POSTGRES_DB:-panicless}
DB_USER=${POSTGRES_USER:-postgres}
TABLES=$(docker-compose exec -T postgres psql -U "$DB_USER" -d "$DB_NAME" -t -c "SELECT tablename FROM pg_tables WHERE schemaname='public'" | grep -v "^$" | wc -l)
if [ "$TABLES" -ge 3 ]; then
    echo -e "${GREEN}✅ Database tables exist (found $TABLES tables)${NC}"
else
    echo -e "${RED}❌ Database tables missing (found only $TABLES tables)${NC}"
    exit 1
fi
echo ""

# Summary
echo "=============================================="
echo -e "${GREEN}🎉 All tests passed!${NC}"
echo ""
echo "You can now access:"
echo "  Frontend & Backend: http://localhost:8080"
echo "  API Docs:          http://localhost:8080/openapi.json"
echo "  Health Check:      http://localhost:8080/health"
echo ""
echo "To view logs:    docker-compose logs -f"
echo "To stop:         docker-compose down"
echo "To restart:      docker-compose restart"
echo ""
