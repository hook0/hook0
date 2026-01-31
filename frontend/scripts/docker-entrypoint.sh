#!/usr/bin/env sh
set -eu

# Set the right API endpoint URL (app was build with a placeholder so that the value can be set without rebuilding)
find /usr/share/caddy/ -name "*.js" -exec sed -i "s|API_ENDPOINT_PLACEHOLDER|${API_ENDPOINT}|g" {} \;

# Set the right API timeout duration (app was build with a placeholder so that the value can be set without rebuilding)
find /usr/share/caddy/ -name "*.js" -exec sed -i "s|API_TIMEOUT_PLACEHOLDER|${API_TIMEOUT:-3000}|g" {} \;

# Run Caddy to serve the app
exec caddy run --config /etc/caddy/Caddyfile --adapter caddyfile
