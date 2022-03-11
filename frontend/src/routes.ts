import ApplicationsList from '@/pages/applications/ApplicationsList.vue';
import ApplicationsEdit from '@/pages/applications/ApplicationsEdit.vue';
import OrganizationList from '@/pages/organizations/OrganizationsList.vue';
import ApiDocumentation from '@/pages/api/documentation/ApiDocumentation.vue';
import Error404 from '@/pages/Error404.vue';

export const routes = {
  Home: 'Home',
  OrganizationsDetail: 'OrganizationsDetail',
  ApplicationsList: 'ApplicationsList',
  ApplicationsDetail: 'ApplicationsDetail',
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
    component: ApplicationsEdit,
  },
  {
    name: routes.ApplicationsDetail,
    path: '/applications/:id',
    component: ApplicationsEdit,
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
    path: '/api/documentation',
    component: ApiDocumentation,
  },
  {
    name: routes.Error404,
    path: '/(.*)',
    component: Error404,
  },
];
