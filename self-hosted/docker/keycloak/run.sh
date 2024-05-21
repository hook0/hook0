#!/usr/bin/env sh
set -eu

echo "Starting a temporary Keycloak instance on a non-exposed port"
TMP_KEYCLOAK_PORT=8888
/opt/keycloak/bin/kc.sh -v start --optimized --http-port $TMP_KEYCLOAK_PORT &
TMP_KEYCLOAK_PID="$!"

# Wait until it is up
while true; do
    UP=$(curl --silent http://localhost:$TMP_KEYCLOAK_PORT/health/live | jq -r '.status')
    if [ "$UP" = "UP" ]; then
        echo "Keycloak is up"
        break
    fi
    sleep 1
done

echo "Creating a realm for Hook0 if it does not already exist"
export KEYCLOAK_HOME="/opt/keycloak"
export HOME="/tmp"
/opt/configure-keycloak.sh "http://localhost:$TMP_KEYCLOAK_PORT" "$KEYCLOAK_ADMIN" "$KEYCLOAK_ADMIN" "$KEYCLOAK_REALM"

echo "Killing the temporary Keycloak instance and starting the final one"
kill $TMP_KEYCLOAK_PID
/opt/keycloak/bin/kc.sh -v start --optimized
