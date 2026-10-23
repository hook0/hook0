// Every `:something_id` segment of a dashboard route names a resource by its UUID, and pages
// splice that value straight into authenticated API paths (`/subscriptions/${id}`). vue-router
// decodes params, so an unconstrained `..%2F..%2Fapplications%2F<uuid>` would reach the page as
// `../../applications/<uuid>` and the browser would normalise the API URL onto another
// resource: a crafted link could turn "delete this subscription" into "delete this application".
// Constraining the params where routes are declared rejects such a link before any page runs,
// for every route at once, instead of trusting each of the many call sites to encode.

const UUID = '[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}';

/**
 * The same route path, with every `:<name>_id` param that carries no pattern of its own
 * restricted to a UUID. A URL whose id is anything else matches no such route and falls through
 * to the catch-all 404.
 */
export function constrainIdParams(path: string): string {
  return path.replace(/:(\w+_id)(?![\w(])/g, `:$1(${UUID})`);
}
