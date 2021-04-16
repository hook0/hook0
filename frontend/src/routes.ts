import ApplicationsList from './pages/applications/applications.vue';
import OrganizationList from './pages/organizations/organizations.vue';
import Home from './pages/Home.vue';

// 2. Define some routes
// Each route should map to a component.
// We'll talk about nested routes later.
export default [
  { path: '/', component: Home },
  { path: '/organizations', component: OrganizationList },
  { path: '/applications', component: ApplicationsList },
  { path: '/event_types', component: Home },
];
