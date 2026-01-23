#!/usr/bin/env bash
set -euo pipefail

# Configuration with defaults (can be overridden by environment)
API_HOST="${API_HOST:-localhost}"
API_PORT="${API_PORT:-8080}"
API_URL="http://${API_HOST}:${API_PORT}"
POSTGRES_HOST="${POSTGRES_HOST:-localhost}"
POSTGRES_PORT="${POSTGRES_PORT:-5432}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-postgres}"
POSTGRES_DB="${POSTGRES_DB:-hook0}"
MASTER_API_KEY="${MASTER_API_KEY:-00000000-0000-0000-0000-000000000000}"
TEST_ORGANIZATION_ID="${TEST_ORGANIZATION_ID:-11111111-1111-1111-1111-111111111111}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
MCP_DIR="$(dirname "$SCRIPT_DIR")"

echo "==> Waiting for API at ${API_URL}..."
timeout 30 bash -c "until curl -sf ${API_URL}/api/v1/swagger.json > /dev/null; do sleep 1; done"
echo "==> API is ready"

echo "==> Creating test organization in database..."
PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$POSTGRES_HOST" -p "$POSTGRES_PORT" -U "$POSTGRES_USER" -d "$POSTGRES_DB" -c \
  "INSERT INTO iam.organization (organization__id, name) VALUES ('${TEST_ORGANIZATION_ID}', 'mcp-test-org') ON CONFLICT DO NOTHING;"

echo "==> Creating service token via API..."
SERVICE_TOKEN=$(curl -sf -X POST "${API_URL}/api/v1/service_token" \
  -H "Authorization: Bearer ${MASTER_API_KEY}" \
  -H "Content-Type: application/json" \
  -d "{\"organization_id\": \"${TEST_ORGANIZATION_ID}\", \"name\": \"mcp-integration-test\"}" \
  | jq -r '.biscuit')

if [ -z "$SERVICE_TOKEN" ] || [ "$SERVICE_TOKEN" = "null" ]; then
  echo "ERROR: Failed to create service token"
  exit 1
fi
echo "==> Service token created"

echo "==> Building MCP with local API OpenAPI spec..."
cd "$MCP_DIR"
HOOK0_OPENAPI_URL="${API_URL}/api/v1/swagger.json" cargo build --release

echo "==> Running integration tests..."
HOOK0_API_URL="$API_URL" MCP_SERVICE_TOKEN="$SERVICE_TOKEN" HOOK0_OPENAPI_URL="${API_URL}/api/v1/swagger.json" cargo test --all-features -- --ignored

echo "==> All integration tests passed!"
