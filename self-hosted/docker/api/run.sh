#!/usr/bin/env sh
set -eu

KEYS_VOLUME_DIR='/opt/keycloak-keys'
if [ -f "$KEYS_VOLUME_DIR/api_client_secret.txt" ]; then
    echo "Reading REALM_PUBLIC_KEY from $KEYS_VOLUME_DIR/realm_public_key.pem"
    KEYCLOAK_OIDC_PUBLIC_KEY=$(cat "$KEYS_VOLUME_DIR/realm_public_key.pem")
    export KEYCLOAK_OIDC_PUBLIC_KEY

    echo "Reading KEYCLOAK_CLIENT_SECRET from $KEYS_VOLUME_DIR/api_client_secret.txt"
    KEYCLOAK_CLIENT_SECRET=$(cat "$KEYS_VOLUME_DIR/api_client_secret.txt")
    export KEYCLOAK_CLIENT_SECRET
fi

/hook0-api
