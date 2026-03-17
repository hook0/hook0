## Known Issues

### Register with existing email shows wrong error
- When registering with an existing email, the API returns "Insufficient rights, You don't have the right to access or edit this resource."
- This is a backend issue — the API should return a proper "Email already registered" error
- Frontend displays whatever the API returns

### Session expiry not handled gracefully
- When session token expires during usage, the user may see errors instead of a graceful redirect to login
- Need: detect 401 responses in the HTTP interceptor → clear auth state → redirect to `/login` with a toast "Session expired"
- This requires backend coordination (token refresh endpoint or proper 401 handling)

### Service Tokens page navigation
- Service Tokens are accessible from the org-level nav (Applications / Service Tokens / Settings)
- The page is at `/organizations/:org_id/services_tokens`
- Some users may not find it — consider adding a link from org settings or a more prominent nav entry
