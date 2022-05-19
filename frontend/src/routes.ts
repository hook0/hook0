import ApplicationsList from '@/pages/organizations/applications/ApplicationsList.vue';
import ApplicationsEdit from '@/pages/organizations/applications/ApplicationsEdit.vue';
import ApplicationsDashboard from "@/pages/organizations/applications/ApplicationsDashboard.vue";

import OrganizationsList from '@/pages/organizations/OrganizationsList.vue';
import OrganizationsEdit from '@/pages/organizations/OrganizationsEdit.vue';
import OrganizationsDashboard from "@/pages/organizations/OrganizationsDashboard.vue";

import ApiDocumentation from '@/pages/api/documentation/ApiDocumentation.vue';
import Error404 from '@/pages/Error404.vue';
import EventTypesList from '@/pages/organizations/applications/event_types/EventTypesList.vue';
import EventTypesNew from '@/pages/organizations/applications/event_types/EventTypesNew.vue';
import Home from '@/Home.vue';
import ComingSoon from "@/pages/ComingSoon.vue";
import SubscriptionsList from "@/pages/organizations/applications/subscriptions/SubscriptionsList.vue";
import SubscriptionsEdit from "@/pages/organizations/applications/subscriptions/SubscriptionsEdit.vue";
import ApplicationSecretsList from "@/pages/organizations/applications/application_secrets/ApplicationSecretsList.vue";

export type Hook0Routes = string;

export const routes: Record<Hook0Routes, string> = {
  Home: 'Home',

  OrganizationsDashboard: 'OrganizationsDashboard',
  OrganizationsDetail: 'OrganizationsDetail',
  OrganizationsNew: 'OrganizationsNew',

  ApplicationsDashboard: 'ApplicationsDashboard',
  ApplicationsList: 'ApplicationsList',
  ApplicationsDetail: 'ApplicationsDetail',
  ApplicationsNew: 'ApplicationsNew',

  ApplicationSecretsList: 'ApplicationSecretsList',
  ApplicationSecretsDetail: 'ApplicationSecretsDetail',
  ApplicationSecretsNew: 'ApplicationSecretsNew',

  EventTypesList: 'EventTypesList',
  EventTypesNew: 'EventTypesNew',

  SubscriptionsList: 'SubscriptionsList',
  SubscriptionsNew: 'SubscriptionsNew',
  SubscriptionsDetail: 'SubscriptionsDetail',

  WebhooksList: 'WebhooksList',
  LogsList: 'LogsList',
  Settings: 'Settings',
  APIDocumentation: 'APIDocumentation',
  Error404: '404',
};

export default [
  {
    name: routes.Home,
    path: '/',
    component: Home,
  },
  {
    name: routes.OrganizationsNew,
    path: '/organizations/new',
    component: OrganizationsEdit,
  },
  {
    name: routes.OrganizationsDetail,
    path: '/organizations/:organization_id/settings',
    component: OrganizationsEdit,
  },
  {
    name: routes.OrganizationsDashboard,
    path: '/organizations/:organization_id/dashboard',
    component: OrganizationsDashboard,
  },
  {
    name: routes.ApplicationsList,
    path: '/organizations/:organization_id/applications',
    component: ApplicationsList,
  },
  {
    name: routes.ApplicationsNew,
    path: '/organizations/:organization_id/applications/new',
    component: ApplicationsEdit,
  },
  {
    name: routes.ApplicationsDashboard,
    path: '/organizations/:organization_id/applications/:application_id/dashboard',
    component: ApplicationsDashboard,
  },
  {
    name: routes.ApplicationsDetail,
    path: '/organizations/:organization_id/applications/:application_id/settings',
    component: ApplicationsEdit,
  },

  {
    name: routes.ApplicationSecretsList,
    path: '/organizations/:organization_id/applications/:application_id/application_secrets',
    component: ApplicationSecretsList,
  },

  {
    name: routes.EventTypesList,
    path: '/organizations/:organization_id/applications/:application_id/event_types',
    component: EventTypesList,
  },
  {
    // EventTypes are immutable, they can only be created or removed
    name: routes.EventTypesNew,
    path: '/organizations/:organization_id/applications/:application_id/event_types/new',
    component: EventTypesNew,
  },

  {
    name: routes.SubscriptionsList,
    path: '/organizations/:organization_id/applications/:application_id/subscriptions',
    component: SubscriptionsList,
  },
  {
    name: routes.SubscriptionsNew,
    path: '/organizations/:organization_id/applications/:application_id/subscriptions/new',
    component: SubscriptionsEdit,
  },
  {
    name: routes.SubscriptionsDetail,
    path: '/organizations/:organization_id/applications/:application_id/subscriptions/:subscription_id',
    component: SubscriptionsEdit,
  },
  {
    name: routes.LogsList,
    path: '/organizations/:organization_id/applications/:application_id/logs',
    component: ComingSoon,
  },
  {
    name: routes.Settings,
    path: '/settings',
    component: ComingSoon,
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
