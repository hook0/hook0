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

---

## Technical Debt

### Fonts loaded twice
- `main.ts` imports `@fontsource-variable/inter` and `@fontsource/jetbrains-mono` (npm packages that inject their own `@font-face` rules)
- `tailwind.css` also declares manual `@font-face` rules pointing to `public/fonts/*.woff2`
- This results in duplicate font declarations. Pick one approach and remove the other.
- Files: `src/main.ts:6-9`, `src/assets/styles/tailwind.css:5-50`, `public/fonts/`

### AG-Grid adapter in Hook0Table (Phase 2 removal)
- `Hook0Table.vue` contains a temporary AG-Grid `columnDefs` to TanStack `ColumnDef` adapter (~60 lines)
- Marked "temporary, removed in Phase 2" — all pages should migrate to native TanStack column definitions
- Then remove: `AgGridColDef` type, `resolveParams()`, `adaptAgGridColDefs()`, and legacy props (`columnDefs`, `rowData`, `context`)
- File: `src/components/Hook0Table.vue:20-120`

### http.ts creates a new Axios instance per request
- `getAxios()` uses a dynamic import + `axios.create()` on every single API call
- The dynamic import is needed to avoid circular deps with the auth store, but the instance creation could be cached
- Consider: create the instance once per auth state change, or use a singleton with an interceptor that reads the current token lazily
- File: `src/http.ts:11-63`

### Routes not type-safe
- Route names are typed via `as const` object, but the route definitions array is untyped (`RouteRecordRaw[]`)
- Route params (`organization_id`, `application_id`, etc.) are accessed as `string` with no compile-time safety
- Consider: typed router (e.g. unplugin-vue-router) or at minimum a helper that validates params at the boundary
- File: `src/routes.ts`

### Untracked files from redesign branch
- `frontend/design-proposals.html`, `frontend/screenshot-all-pages.ts`, `frontend/screenshots/` are untracked
- Either `.gitignore` them or clean them up before merging to master
