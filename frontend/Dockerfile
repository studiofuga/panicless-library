# Build stage
FROM node:20-alpine as builder

# Build argument for API URL
ARG VITE_API_BASE_URL=http://localhost:8080

WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Set environment variable for Vite build
ENV VITE_API_BASE_URL=${VITE_API_BASE_URL}

# Build the application
RUN npm run build

# Production stage
FROM nginx:1.25-alpine

# Copy nginx configuration
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Copy built application from builder
COPY --from=builder /app/dist /usr/share/nginx/html

# Copy entrypoint script
COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

# Add healthcheck
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --quiet --tries=1 --spider http://localhost/health || exit 1

# Expose port
EXPOSE 80

# Start with entrypoint script that generates runtime config
ENTRYPOINT ["/docker-entrypoint.sh"]
