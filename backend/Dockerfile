# Frontend build stage
FROM node:20-alpine AS frontend-builder

# Build argument for API URL - can be overridden at build time
ARG VITE_API_BASE_URL=http://localhost:8080

WORKDIR /frontend

# Copy frontend files
COPY frontend/package*.json ./
RUN npm ci

COPY frontend/src ./src
COPY frontend/index.html ./
COPY frontend/vite.config.js ./

# Build frontend with API URL environment variable
ENV VITE_API_BASE_URL=${VITE_API_BASE_URL}
RUN npm run build

# Generate config.json in dist directory using the same API URL
RUN echo "{\"apiBaseURL\":\"${VITE_API_BASE_URL}\"}" > /frontend/dist/config.json

# Backend build stage
# Using 1.85 for edition 2024 support (base64ct) and home 0.5.11 (pinned)
FROM rust:1.85-slim AS backend-builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /app

# Copy workspace files
COPY Cargo.toml Cargo.lock ./
COPY panicless-mcp-lib ./panicless-mcp-lib
COPY mcp-server ./mcp-server
COPY backend ./backend

# Build the backend application in release mode
RUN cargo build --release --package panicless-backend

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy the binary from backend builder
COPY --from=backend-builder /app/target/release/panicless-backend /app/panicless-backend

# Copy frontend dist files from frontend builder
COPY --from=frontend-builder /frontend/dist ./frontend/dist

# Change ownership
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Expose port
EXPOSE 8080

# Set environment variables (can be overridden)
ENV RUST_LOG="info,panicless_backend=debug"
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080

# Run the binary
CMD ["/app/panicless-backend"]
