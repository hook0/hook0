import {createRouter, createWebHistory, RouteLocationNormalized} from 'vue-router';
import routes, {routes as routeNames} from '@/routes';
import {list} from '@/pages/organizations/OrganizationService';
import {UUID} from '@/http';

const router = createRouter({
  // Provide the history implementation to use
  history: createWebHistory(),
  routes,
});

export default router;

/*
router.beforeEach(async (to, from, next) => {

  const from_has_organization_id = String(from.query.organization_id).includes('-');
  const to_has_organization_id = String(to.query.organization_id).includes('-');

  if (to_has_organization_id) {
    //  move on to the next hook in the pipeline
    return next();
  }

  if (from_has_organization_id && !to_has_organization_id) {
    addOrganizationId(to, from.query.organization_id as string);
    return;
  }

  await list().then(organizations => {
    if (!Array.isArray(organizations) || organizations.length === 0) {
      return next({ name: routeNames.Error404 });
    }

    addOrganizationId(to, organizations[0].organization_id);
  });
});
*/
