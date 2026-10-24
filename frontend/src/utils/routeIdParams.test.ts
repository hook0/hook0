import { createMemoryHistory, createRouter } from 'vue-router';
import { constrainIdParams } from './routeIdParams';

const ORG = '0b9b1c5e-2f0a-4c55-9a51-6f3e1c2d7a10';
const APP = '5d1f3a0c-8e2b-4d77-b1c4-2a9e6f0b3c21';
const SUB = '9e4c2b1a-7f3d-4a68-8c05-1d2e3f4a5b6c';

function routerFor(...paths: string[]) {
  return createRouter({
    history: createMemoryHistory(),
    routes: [
      ...paths.map((path, i) => ({
        name: `route${i}`,
        path: constrainIdParams(path),
        component: {},
      })),
      { name: 'NotFound', path: '/:pathMatch(.*)*', component: {} },
    ],
  });
}

describe('constrainIdParams', () => {
  it('restricts every id param to a UUID and leaves the rest of the path alone', () => {
    const constrained = constrainIdParams(
      '/organizations/:organization_id/applications/:application_id/subscriptions/:subscription_id'
    );

    expect(constrained.match(/\(\[0-9a-fA-F\]\{8\}/g)).toHaveLength(3);
    expect(constrained.startsWith('/organizations/:organization_id(')).toBe(true);
    expect(constrainIdParams('/organizations/:pathMatch(.*)*')).toBe(
      '/organizations/:pathMatch(.*)*'
    );
    expect(constrainIdParams('/tutorial/organization')).toBe('/tutorial/organization');
  });

  it('still resolves a link whose ids are UUIDs, upper or lower case', () => {
    const router = routerFor(
      '/organizations/:organization_id/applications/:application_id/subscriptions/:subscription_id'
    );

    const resolved = router.resolve(
      `/organizations/${ORG}/applications/${APP}/subscriptions/${SUB.toUpperCase()}`
    );

    expect(resolved.name).toBe('route0');
    expect(resolved.params.subscription_id).toBe(SUB.toUpperCase());
  });

  it('sends an id that walks out of its segment to the 404 instead of a page', () => {
    const router = routerFor(
      '/organizations/:organization_id/applications/:application_id/subscriptions/:subscription_id'
    );

    const resolved = router.resolve(
      `/organizations/${ORG}/applications/${APP}/subscriptions/..%2Fapplications%2F${APP}`
    );

    expect(resolved.name).toBe('NotFound');
  });

  it('keeps a static sibling such as `new` reachable', () => {
    const router = routerFor(
      '/organizations/:organization_id/applications/:application_id/subscriptions/new',
      '/organizations/:organization_id/applications/:application_id/subscriptions/:subscription_id'
    );

    expect(router.resolve(`/organizations/${ORG}/applications/${APP}/subscriptions/new`).name).toBe(
      'route0'
    );
  });
});
