#!/bin/sh
# Generate runtime configuration from environment variables

CONFIG_FILE="/usr/share/nginx/html/config.json"

# Create config.json from environment variable
cat > "$CONFIG_FILE" << EOF
{
  "apiBaseURL": "${VITE_API_BASE_URL:-http://localhost:8080}"
}
EOF

echo "Generated $CONFIG_FILE with apiBaseURL: ${VITE_API_BASE_URL:-http://localhost:8080}"

# Start nginx
exec nginx -g "daemon off;"
