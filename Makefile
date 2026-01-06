.PHONY: help build up down logs restart clean migration-up seed test

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build all Docker images
	docker-compose build

up: ## Start all services
	docker-compose up -d

down: ## Stop all services
	docker-compose down

logs: ## Show logs from all services
	docker-compose logs -f

logs-backend: ## Show backend logs
	docker-compose logs -f backend

logs-frontend: ## Show frontend logs
	docker-compose logs -f frontend

logs-postgres: ## Show postgres logs
	docker-compose logs -f postgres

restart: ## Restart all services
	docker-compose restart

restart-backend: ## Restart backend only
	docker-compose restart backend

restart-frontend: ## Restart frontend only
	docker-compose restart frontend

clean: ## Stop and remove all containers, networks, and volumes
	docker-compose down -v

ps: ## Show running containers
	docker-compose ps

migration-up: ## Run database migrations
	docker-compose exec postgres psql -U panicless -d panicless_library -f /docker-entrypoint-initdb.d/00000000000001_create_users_table.sql
	docker-compose exec postgres psql -U panicless -d panicless_library -f /docker-entrypoint-initdb.d/00000000000002_create_books_table.sql
	docker-compose exec postgres psql -U panicless -d panicless_library -f /docker-entrypoint-initdb.d/00000000000003_create_readings_table.sql

seed: ## Load seed data
	docker-compose exec -T postgres psql -U panicless -d panicless_library < database/seed_data.sql

shell-backend: ## Open shell in backend container
	docker-compose exec backend sh

shell-postgres: ## Open psql shell
	docker-compose exec postgres psql -U panicless -d panicless_library

test-backend: ## Run backend tests
	cd backend && cargo test

rebuild: down build up ## Rebuild and restart all services

dev: ## Start development environment (postgres only)
	docker-compose up -d postgres
