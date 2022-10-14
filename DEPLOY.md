# Deploying Hook0

This guide wil help you run your own Hook0 instance.

## Keycloak

Hook0 relies on [Keycloak](https://www.keycloak.org/) to manage users and permissions.
You will need administrative access to a dedicated Keycloak realm.

The following sections will help you configure your realm for Hook0.

_This documentation was written for the legacy Keycloak administration console which has been replaced in Keycloak 19. The configuration you need to do remains the same but the screens and field names might differ._

### Groups Scope

Let's create a new client scope that will include in JWT every group our users belong to.

- create a new **Client Scope**:
  - Name: `groups`
  - Protocol: `openid-connect`
  - Display On Consent Screen: OFF
  - Include In Token Scope: ON
- create a new **Mapper** for this Client Scope:
  - Name: `group_membership`
  - Mapper Type: `Group Membership`
  - Token Claim Name: `groups`
  - Full group path: ON
  - Add to ID token: ON
  - Add to access token: ON
  - Add to userinfo : ON

### Console Client

Now we need a client for Hook0 web console.

- create a new **Client**:
  - Client ID: `hook0`
  - Client Protocol: `openid-connect`
  - Root URL: the URL where you want to deploy Hook0's API/console
- configure this client's **Settings**:
  - Name: `Hook0`
  - Access Type: `public`
  - Standard Flow Enabled: ON
  - Implicit Flow Enabled: OFF
  - Direct Access Grants Enabled: OFF
  - OAuth 2.0 Device Authorization Grant Enabled: OFF
  - Valid Redirect URIs: `/*` (you need to click on **+**)
  - Base URL: `/`
  - Web Origins: `+` (you need to click on **+**)
- configure this client's **Client Scopes**:
  - move the `groups` entry to **Assigned Default Client Scopes**
- configure this client's **Scope**:
  - Full Scope Allowed: OFF

### API Client

Next, Hook0's API will need its own private client with a service account so that it can register new users or organizations.

- create a new **Client**:
  - Client ID: `hook0-api`
  - Client Protocol: `openid-connect`
  - Root URL: the URL where you want to deploy Hook0's API/console
- configure this client's **Settings**:
  - Name: `Hook0 API`
  - Access Type: `confidential`
  - Standard Flow Enabled: OFF
  - Implicit Flow Enabled: OFF
  - Direct Access Grants Enabled: ON
  - Service Accounts Enabled: ON
  - OAuth 2.0 Device Authorization Grant Enabled: OFF
  - OIDC CIBA Grant Enabled: OFF
  - Authorization Enabled: OFF
  - Valid Redirect URIs: _empty_
  - Base URL: _empty_
  - Web Origins: _empty_
- configure this client's **Scope**:
  - Full Scope Allowed: ON
- configure this client's **Service Account Roles**:
  - Client Roles: `realm-management`
  - move the `manage-users` entry to **Assigned Roles**

### Authentication Settings

We also need to enable a few settings to ensure the overall security of the system.

- in **Authentication** > **Password Policy**:
  - add a **Minimum Length** policy
  - add a **Not Email** policy
- in **Authentication** > **Required Actions**:
  - check **Update Password** as a **Default Action**

## API

_TODO_

## Output Worker

_TODO_
