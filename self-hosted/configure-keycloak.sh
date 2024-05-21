#!/usr/bin/env sh
set -eu

KCADM="$KEYCLOAK_HOME/bin/kcadm.sh"
CFG="${HOME:-/tmp}/.keycloak/kcadm.config"
KEYS_VOLUME_DIR='/opt/keycloak-keys'

if [ $# -lt 4 ]; then
    echo "You need to call this script with the following arguments: SERVER_URL, ADMIN_USER, ADMIN_PASSWORD, REALM_NAME"
    exit 1
fi

SERVER_URL="$1"
ADMIN_USER="$2"
ADMIN_PASSWORD="$3"
REALM_NAME="$4"

# Login
$KCADM config credentials --config "$CFG" --server "$SERVER_URL" --realm master --user "$ADMIN_USER" --password "$ADMIN_PASSWORD"

# Stop if realm already exists
REALM_EXISTS=$($KCADM get realms --config "$CFG" -q briefRepresentation=true | jq -r "any(.realm == \"$REALM_NAME\")")
if [ "$REALM_EXISTS" = "true" ]; then
    echo "Realm '$REALM_NAME' already exist; doing nothing"
else
    # Create realm
    $KCADM create realms --config "$CFG" -s "realm=$REALM_NAME" -s enabled=true

    # Create group client scope
    GROUP_CLIENT_SCOPE=$($KCADM create client-scopes --config "$CFG" -r "$REALM_NAME" -s name=groups -s protocol=openid-connect -s 'attributes={ "display.on.consent.screen": false, "include.in.token.scope": true }' --id)
    $KCADM create "client-scopes/$GROUP_CLIENT_SCOPE/protocol-mappers/models" --config "$CFG" -r "$REALM_NAME" -s name=group_membership -s protocol=openid-connect -s protocolMapper=oidc-group-membership-mapper -s 'config={ "full.path": true, "id.token.claim": true, "access.token.claim": true, "claim.name": "groups", "userinfo.token.claim": true }'
    echo "Created group client scope with id '$GROUP_CLIENT_SCOPE'"

    # Create frontend client
    FRONTEND_CLIENT=$($KCADM create clients --config "$CFG" -r "$REALM_NAME" -s enabled=true -s name=Hook0 -s protocol=openid-connect -s clientId=hook0 -s publicClient=true -s standardFlowEnabled=true -s implicitFlowEnabled=false -s directAccessGrantsEnabled=false -s serviceAccountsEnabled=false -s rootUrl=http://localhost:8081 -s baseUrl=/ -s 'redirectUris=["/*", "http://localhost:8001/*"]' -s 'webOrigins=["+"]' -s fullScopeAllowed=false --id)
    echo "Created frontend client with id '$FRONTEND_CLIENT'"
    $KCADM update "clients/$FRONTEND_CLIENT/default-client-scopes/$GROUP_CLIENT_SCOPE" --config "$CFG" -r "$REALM_NAME"

    # Create API client
    API_CLIENT=$($KCADM create clients --config "$CFG" -r "$REALM_NAME" -s enabled=true -s 'name=Hook0 API' -s protocol=openid-connect -s clientId=hook0-api -s publicClient=false -s standardFlowEnabled=false -s implicitFlowEnabled=false -s directAccessGrantsEnabled=true -s serviceAccountsEnabled=true -s rootUrl=http://localhost:8081 -s fullScopeAllowed=true --id)
    $KCADM update "clients/$API_CLIENT/default-client-scopes/$GROUP_CLIENT_SCOPE" --config "$CFG" -r "$REALM_NAME"
    $KCADM add-roles --config "$CFG" -r "$REALM_NAME" --uusername service-account-hook0-api --cclientid realm-management --rolename manage-users
    echo "Created API client with id '$API_CLIENT'"
fi

# Get realm public key
REALM_PUBLIC_KEY=$($KCADM get keys --config "$CFG" -r "$REALM_NAME" | jq -r '.keys[] | select(.algorithm == "RS256") | .publicKey')
echo "Realm public key is '$REALM_PUBLIC_KEY'"

# Get API client's secret
API_CLIENT=$($KCADM get clients --config "$CFG" -r "$REALM_NAME" -q clientId=hook0-api -q max=1 --fields id | jq -r '.[0].id')
API_CLIENT_SECRET=$($KCADM get "clients/$API_CLIENT/client-secret" --config "$CFG" -r "$REALM_NAME" --fields value | jq -r '.value')
echo "API client (hook0-api) secret is '$API_CLIENT_SECRET'"

# If run from Docker Compose, store API client's secret in a volume so it can be picked up by API
if [ -d "$KEYS_VOLUME_DIR" ]; then
    echo "Writing keys in volume"
    echo "-----BEGIN PUBLIC KEY-----$REALM_PUBLIC_KEY-----END PUBLIC KEY-----" > "$KEYS_VOLUME_DIR/realm_public_key.pem"
    echo "$API_CLIENT_SECRET" > "$KEYS_VOLUME_DIR/api_client_secret.txt"
fi
