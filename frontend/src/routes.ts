import ApplicationsList from '@/pages/organizations/applications/ApplicationsList.vue';
import ApplicationsEdit from '@/pages/organizations/applications/ApplicationsEdit.vue';
import ApplicationsDashboard from '@/pages/organizations/applications/ApplicationsDashboard.vue';

import OrganizationsEdit from '@/pages/organizations/OrganizationsEdit.vue';
import OrganizationsDashboard from '@/pages/organizations/OrganizationsDashboard.vue';

import ApiDocumentation from '@/pages/api/documentation/ApiDocumentation.vue';
import Error404 from '@/pages/Error404.vue';
import Login from '@/pages/LoginPage.vue';
import Register from '@/pages/RegisterPage.vue';
import EventTypesList from '@/pages/organizations/applications/event_types/EventTypesList.vue';
import EventTypesNew from '@/pages/organizations/applications/event_types/EventTypesNew.vue';
import Home from '@/Home.vue';
//import ComingSoon from '@/pages/ComingSoon.vue';
import UsersSettings from '@/pages/users/UsersSettings.vue';
import SubscriptionsList from '@/pages/organizations/applications/subscriptions/SubscriptionsList.vue';
import SubscriptionsEdit from '@/pages/organizations/applications/subscriptions/SubscriptionsEdit.vue';
import LogList from '@/pages/organizations/applications/logs/LogList.vue';
import ApplicationSecretsList from '@/pages/organizations/applications/application_secrets/ApplicationSecretsList.vue';
import EventsList from '@/pages/organizations/applications/events/EventsList.vue';
import EventsDetail from '@/pages/organizations/applications/events/EventsDetail.vue';
import VerifyUser from '@/pages/users/VerifyEmail.vue';
import BeginResetPassword from '@/pages/BeginResetPassword.vue';
import ResetPassword from '@/pages/ResetPassword.vue';
import ServicesTokenList from '@/pages/organizations/services_token/ServicesTokenList.vue';
import ServiceTokenView from '@/pages/organizations/services_token/ServiceTokenView.vue';

export type Hook0Routes = string;

export const routes: Record<Hook0Routes, string> = {
  Home: 'Home',

  Login: 'Login',
  Register: 'Register',
  VerifyEmail: 'VerifyEmail',
  UserSettings: 'UserSettings',
  BeginResetPassword: 'BeginResetPassword',
  ResetPassword: 'ResetPassword',

  OrganizationsDashboard: 'OrganizationsDashboard',
  OrganizationsDetail: 'OrganizationsDetail',
  OrganizationsNew: 'OrganizationsNew',

  ServicesTokenList: 'ServicesTokenList',
  ServiceTokenView: 'ServiceTokenView',

  ApplicationsDashboard: 'ApplicationsDashboard',
  ApplicationsList: 'ApplicationsList',
  ApplicationsDetail: 'ApplicationsDetail',
  ApplicationsNew: 'ApplicationsNew',

  ApplicationSecretsList: 'ApplicationSecretsList',
  ApplicationSecretsDetail: 'ApplicationSecretsDetail',
  ApplicationSecretsNew: 'ApplicationSecretsNew',

  EventsList: 'EventsList',
  EventsDetail: 'EventsDetail',

  EventTypesList: 'EventTypesList',
  EventTypesNew: 'EventTypesNew',

  SubscriptionsList: 'SubscriptionsList',
  SubscriptionsNew: 'SubscriptionsNew',
  SubscriptionsDetail: 'SubscriptionsDetail',

  WebhooksList: 'WebhooksList',
  LogsList: 'LogsList',
  APIDocumentation: 'APIDocumentation',
  APIDocumentationForApplication: 'APIDocumentationForApplication',
  Error404: '404',
};

export default [
  {
    name: routes.Home,
    path: '/',
    component: Home,
  },
  {
    name: routes.Login,
    path: '/login',
    component: Login,
    meta: { requiresAuth: false },
  },
  {
    name: routes.Register,
    path: '/register',
    component: Register,
    meta: { requiresAuth: false },
  },
  {
    name: routes.VerifyEmail,
    path: '/verify-email',
    component: VerifyUser,
    meta: { requiresAuth: false },
  },
  {
    name: routes.UserSettings,
    path: '/settings',
    component: UsersSettings,
  },
  {
    name: routes.BeginResetPassword,
    path: '/begin-reset-password',
    component: BeginResetPassword,
    meta: {
      requiresAuth: false,
      redirectIfLoggedIn: false,
    },
  },
  {
    name: routes.ResetPassword,
    path: '/reset-password',
    component: ResetPassword,
    meta: {
      requiresAuth: false,
      redirectIfLoggedIn: false,
    },
  },
  {
    name: routes.OrganizationsNew,
    path: '/organizations/new',
    component: OrganizationsEdit,
  },
  {
    name: routes.ServicesTokenList,
    path: '/organizations/:organization_id/services_tokens',
    component: ServicesTokenList,
  },
  {
    name: routes.ServiceTokenView,
    path: '/organizations/:organization_id/services_tokens/:service_token_id',
    component: ServiceTokenView,
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
    name: routes.EventsList,
    path: '/organizations/:organization_id/applications/:application_id/events',
    component: EventsList,
  },

  {
    name: routes.EventsDetail,
    path: '/organizations/:organization_id/applications/:application_id/events/:event_id',
    component: EventsDetail,
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
    component: LogList,
  },

  {
    name: routes.APIDocumentationForApplication,
    path: '/organizations/:organization_id/applications/:application_id/documentation',
    component: ApiDocumentation,
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
