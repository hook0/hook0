import ApplicationsList from './pages/applications/ApplicationsList.vue';
import OrganizationList from './pages/organizations/OrganizationsList.vue';
import Default from './pages/Home.vue';

export const routes = {
  Default: 'Default',
  OrganizationsDetail: 'OrganizationsDetail',
  ApplicationsList: 'ApplicationsList',
  EventTypesList: 'EventTypesList',
  WebhooksList: 'WebhooksList',
  LogsList: 'LogsList',
  Settings: 'Settings',
  APIDocumentation: 'APIDocumentation',
};

export default [
  { name: routes.Default, path: '/', component: Default },
  {
    name: routes.OrganizationsDetail,
    path: '/organizations/:organization_id',
    component: OrganizationList,
    children: [
      { name: routes.ApplicationsList, path: 'applications', component: ApplicationsList },
      { name: routes.EventTypesList, path: 'event_types', component: Default },
      { name: routes.WebhooksList, path: 'webhooks', component: Default },
      { name: routes.LogsList, path: 'logs', component: Default },
      { name: routes.Settings, path: 'settings', component: Default },
      { name: routes.APIDocumentation, path: 'api', component: Default },
    ],
  },
];
