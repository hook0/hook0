export type Hook0Routes = string;

export const routes: Record<Hook0Routes, string> = {
  Home: 'Home',

  Tutorial: 'Tutorial',
  TutorialCreateOrganization: 'TutorialCreateOrganization',
  TutorialCreateApplication: 'TutorialCreateApplication',
  TutorialCreateEventType: 'TutorialCreateEventType',
  TutorialCreateSubscription: 'TutorialCreateSubscription',
  TutorialSendEvent: 'TutorialSendEvent',
  TutorialSuccess: 'TutorialSuccess',

  Login: 'Login',
  Register: 'Register',
  VerifyEmail: 'VerifyEmail',
  UserSettings: 'UserSettings',
  BeginResetPassword: 'BeginResetPassword',
  ResetPassword: 'ResetPassword',
  CheckEmail: 'CheckEmail',

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
    component: () => import('@/Home.vue'),
  },
  {
    name: routes.Tutorial,
    path: '/tutorial',
    component: () => import('@/pages/tutorial/TutorialIntroduction.vue'),
    meta: { tutorial: true },
  },
  {
    name: routes.TutorialCreateOrganization,
    path: '/tutorial/organization',
    component: () => import('@/pages/tutorial/TutorialCreateOrganization.vue'),
    meta: { tutorial: true },
  },
  {
    name: routes.TutorialCreateApplication,
    path: '/tutorial/application/organizations/:organization_id',
    component: () => import('@/pages/tutorial/TutorialCreateApplication.vue'),
    meta: { tutorial: true },
  },
  {
    name: routes.TutorialCreateEventType,
    path: '/tutorial/event_type/organizations/:organization_id/applications/:application_id',
    component: () => import('@/pages/tutorial/TutorialCreateEventType.vue'),
    meta: { tutorial: true },
  },
  {
    name: routes.TutorialCreateSubscription,
    path: '/tutorial/subscription/organizations/:organization_id/applications/:application_id',
    component: () => import('@/pages/tutorial/TutorialCreateSubscription.vue'),
    meta: { tutorial: true },
  },
  {
    name: routes.TutorialSendEvent,
    path: '/tutorial/event/organizations/:organization_id/applications/:application_id',
    component: () => import('@/pages/tutorial/TutorialSendEvent.vue'),
    meta: { tutorial: true },
  },
  {
    name: routes.TutorialSuccess,
    path: '/tutorial/success/organizations/:organization_id/applications/:application_id',
    component: () => import('@/pages/tutorial/TutorialSuccess.vue'),
  },
  {
    name: routes.Login,
    path: '/login',
    component: () => import('@/pages/LoginPage.vue'),
    meta: { requiresAuth: false },
  },
  {
    name: routes.Register,
    path: '/register',
    component: () => import('@/pages/RegisterPage.vue'),
    meta: { requiresAuth: false },
  },
  {
    name: routes.VerifyEmail,
    path: '/verify-email',
    component: () => import('@/pages/user/VerifyEmail.vue'),
    meta: { requiresAuth: false },
  },
  {
    name: routes.CheckEmail,
    path: '/check-email',
    component: () => import('@/pages/CheckEmailPage.vue'),
    meta: { requiresAuth: false },
  },
  {
    name: routes.UserSettings,
    path: '/settings',
    component: () => import('@/pages/user/UserSettings.vue'),
  },
  {
    name: routes.BeginResetPassword,
    path: '/begin-reset-password',
    component: () => import('@/pages/BeginResetPassword.vue'),
    meta: {
      requiresAuth: false,
      redirectIfLoggedIn: false,
    },
  },
  {
    name: routes.ResetPassword,
    path: '/reset-password',
    component: () => import('@/pages/ResetPassword.vue'),
    meta: {
      requiresAuth: false,
      redirectIfLoggedIn: false,
    },
  },
  {
    name: routes.OrganizationsNew,
    path: '/organizations/new',
    component: () => import('@/pages/organizations/OrganizationsEdit.vue'),
  },
  {
    name: routes.ServicesTokenList,
    path: '/organizations/:organization_id/services_tokens',
    component: () => import('@/pages/organizations/services_token/ServicesTokenList.vue'),
  },
  {
    name: routes.ServiceTokenView,
    path: '/organizations/:organization_id/services_tokens/:service_token_id',
    component: () => import('@/pages/organizations/services_token/ServiceTokenView.vue'),
  },
  {
    name: routes.OrganizationsDetail,
    path: '/organizations/:organization_id/settings',
    component: () => import('@/pages/organizations/OrganizationsEdit.vue'),
  },
  {
    name: routes.OrganizationsDashboard,
    path: '/organizations/:organization_id/dashboard',
    component: () => import('@/pages/organizations/OrganizationsDashboard.vue'),
  },
  {
    name: routes.ApplicationsList,
    path: '/organizations/:organization_id/applications',
    component: () => import('@/pages/organizations/applications/ApplicationsList.vue'),
  },
  {
    name: routes.ApplicationsNew,
    path: '/organizations/:organization_id/applications/new',
    component: () => import('@/pages/organizations/applications/ApplicationsEdit.vue'),
  },
  {
    name: routes.ApplicationsDashboard,
    path: '/organizations/:organization_id/applications/:application_id/dashboard',
    component: () => import('@/pages/organizations/applications/ApplicationsDashboard.vue'),
  },
  {
    name: routes.ApplicationsDetail,
    path: '/organizations/:organization_id/applications/:application_id/settings',
    component: () => import('@/pages/organizations/applications/ApplicationsEdit.vue'),
  },

  {
    name: routes.ApplicationSecretsList,
    path: '/organizations/:organization_id/applications/:application_id/application_secrets',
    component: () =>
      import('@/pages/organizations/applications/application_secrets/ApplicationSecretsList.vue'),
  },

  {
    name: routes.EventsList,
    path: '/organizations/:organization_id/applications/:application_id/events',
    component: () => import('@/pages/organizations/applications/events/EventsList.vue'),
  },

  {
    name: routes.EventsDetail,
    path: '/organizations/:organization_id/applications/:application_id/events/:event_id',
    component: () => import('@/pages/organizations/applications/events/EventsDetail.vue'),
  },

  {
    name: routes.EventTypesList,
    path: '/organizations/:organization_id/applications/:application_id/event_types',
    component: () => import('@/pages/organizations/applications/event_types/EventTypesList.vue'),
  },
  {
    // EventTypes are immutable, they can only be created or removed
    name: routes.EventTypesNew,
    path: '/organizations/:organization_id/applications/:application_id/event_types/new',
    component: () => import('@/pages/organizations/applications/event_types/EventTypesNew.vue'),
  },

  {
    name: routes.SubscriptionsList,
    path: '/organizations/:organization_id/applications/:application_id/subscriptions',
    component: () =>
      import('@/pages/organizations/applications/subscriptions/SubscriptionsList.vue'),
  },
  {
    name: routes.SubscriptionsNew,
    path: '/organizations/:organization_id/applications/:application_id/subscriptions/new',
    component: () =>
      import('@/pages/organizations/applications/subscriptions/SubscriptionsEdit.vue'),
  },
  {
    name: routes.SubscriptionsDetail,
    path: '/organizations/:organization_id/applications/:application_id/subscriptions/:subscription_id',
    component: () =>
      import('@/pages/organizations/applications/subscriptions/SubscriptionsEdit.vue'),
  },
  {
    name: routes.LogsList,
    path: '/organizations/:organization_id/applications/:application_id/logs',
    component: () => import('@/pages/organizations/applications/logs/LogList.vue'),
  },

  {
    name: routes.APIDocumentationForApplication,
    path: '/organizations/:organization_id/applications/:application_id/documentation',
    component: () => import('@/pages/api/documentation/ApiDocumentation.vue'),
  },
  {
    name: routes.APIDocumentation,
    path: '/api/documentation',
    component: () => import('@/pages/api/documentation/ApiDocumentation.vue'),
  },
  {
    name: routes.Error404,
    path: '/:pathMatch(.*)*',
    component: () => import('@/pages/Error404.vue'),
  },
];
