import ApplicationsList from '@/pages/applications/ApplicationsList.vue';
import ApplicationsNew from '@/pages/applications/ApplicationsNew.vue';
import OrganizationList from '@/pages/organizations/OrganizationsList.vue';
import Error404 from '@/pages/Error404.vue';
import Default from '@/Default.vue';

export const routes = {
  Home: 'Home',
  OrganizationsDetail: 'OrganizationsDetail',
  ApplicationsList: 'ApplicationsList',
  ApplicationsNew: 'ApplicationsNew',
  EventTypesList: 'EventTypesList',
  WebhooksList: 'WebhooksList',
  LogsList: 'LogsList',
  Settings: 'Settings',
  APIDocumentation: 'APIDocumentation',
  Error404: '404',
};

export default [
  {
    name: routes.OrganizationsDetail,
    path: '/organizations',
    component: OrganizationList,
  },
  {
    name: routes.ApplicationsList,
    path: '/applications',
    component: ApplicationsList,
  },
  {
    name: routes.ApplicationsNew,
    path: '/applications/new',
    component: ApplicationsNew,
  },
  {
    name: routes.EventTypesList,
    path: '/event_types',
    component: { template: `<div>event_types</div>` },
  },
  {
    name: routes.WebhooksList,
    path: '/webhooks',
    component: { template: `<div>webhooks</div>` },
  },
  {
    name: routes.LogsList,
    path: '/logs',
    component: { template: `<div>logs</div>` },
  },
  {
    name: routes.Settings,
    path: '/settings',
    component: { template: `<div>settings</div>` },
  },
  {
    name: routes.APIDocumentation,
    path: '/api',
    component: { template: `<div>api</div>` },
  },
  {
    name: routes.Error404,
    path: '/(.*)',
    component: Error404,
  },
];
