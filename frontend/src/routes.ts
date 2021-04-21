import ApplicationsList from './pages/applications/ApplicationsList.vue';
import OrganizationList from './pages/organizations/OrganizationsList.vue';
import Home from './pages/Home.vue';

export const routes = {
  Home: 'Home',
  OrganizationDetail: 'OrganizationDetail',
  ApplicationsList: 'ApplicationsList',
  EventTypesList: 'EventTypesList',
  WebhooksList: 'WebhooksList',
  LogsList: 'LogsList',
};

// 2. Define some routes
// Each route should map to a component.
// We'll talk about nested routes later.
export default [
  { name: routes.Home, path: '/', component: Home },
  {
    name: routes.OrganizationDetail,
    path: '/organization/:id',
    component: OrganizationList,
  },
  { name: routes.ApplicationsList, path: '/applications', component: ApplicationsList },
  { name: routes.EventTypesList, path: '/event_types', component: Home },
  { name: routes.WebhooksList, path: '/webhooks', component: Home },
  { name: routes.LogsList, path: '/logs', component: Home },
];
