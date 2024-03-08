#!/usr/bin/env bash
set -euo pipefail

./build.sh

docker stop keycloak || true
docker rm keycloak || true

docker run --name keycloak -p 8080:8080 -e KEYCLOAK_ADMIN=admin -e KEYCLOAK_ADMIN_PASSWORD=admin -v /tmp/theme:/opt/keycloak/providers quay.io/keycloak/keycloak:24.0.1 start-dev
