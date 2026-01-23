#!/usr/bin/env sh
set -eu

# Set the right API endpoint URL (app was build with a placeholder so that the value can be set without rebuilding)
find /usr/share/caddy/ -name "*.js" -exec sed -i "s|API_ENDPOINT_PLACEHOLDER|${API_ENDPOINT}|g" {} \;

# Run Caddy to serve the app
caddy run --config /etc/caddy/Caddyfile --adapter caddyfile
