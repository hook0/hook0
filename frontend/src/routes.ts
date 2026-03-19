export const routes = {
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
  OrganizationsTeam: 'OrganizationsTeam',
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
} as const;

export type RouteName = keyof typeof routes;

export default [
  {
    name: routes.Home,
    path: '/',
    component: () => import('@/Home.vue'),
    meta: { title: 'Home' },
  },
  {
    name: routes.Tutorial,
    path: '/tutorial',
    component: () => import('@/pages/tutorial/TutorialWizard.vue'),
    meta: { tutorial: true, title: 'Tutorial' },
  },
  {
    name: routes.TutorialCreateOrganization,
    path: '/tutorial/organization',
    component: () => import('@/pages/tutorial/TutorialWizard.vue'),
    meta: { tutorial: true, title: 'Tutorial — Organization' },
  },
  {
    name: routes.TutorialCreateApplication,
    path: '/tutorial/application/organizations/:organization_id',
    component: () => import('@/pages/tutorial/TutorialWizard.vue'),
    meta: { tutorial: true, title: 'Tutorial — Application' },
  },
  {
    name: routes.TutorialCreateEventType,
    path: '/tutorial/event_type/organizations/:organization_id/applications/:application_id',
    component: () => import('@/pages/tutorial/TutorialWizard.vue'),
    meta: { tutorial: true, title: 'Tutorial — Event Type' },
  },
  {
    name: routes.TutorialCreateSubscription,
    path: '/tutorial/subscription/organizations/:organization_id/applications/:application_id',
    component: () => import('@/pages/tutorial/TutorialWizard.vue'),
    meta: { tutorial: true, title: 'Tutorial — Subscription' },
  },
  {
    name: routes.TutorialSendEvent,
    path: '/tutorial/event/organizations/:organization_id/applications/:application_id',
    component: () => import('@/pages/tutorial/TutorialWizard.vue'),
    meta: { tutorial: true, title: 'Tutorial — Send Event' },
  },
  {
    name: routes.TutorialSuccess,
    path: '/tutorial/success/organizations/:organization_id/applications/:application_id',
    component: () => import('@/pages/tutorial/TutorialWizard.vue'),
    meta: { tutorial: true, title: 'Tutorial — Success' },
  },
  {
    name: routes.Login,
    path: '/login',
    component: () => import('@/pages/LoginPage.vue'),
    meta: { requiresAuth: false, fullScreen: true, title: 'Login' },
  },
  {
    name: routes.Register,
    path: '/register',
    component: () => import('@/pages/RegisterPage.vue'),
    meta: { requiresAuth: false, fullScreen: true, title: 'Register' },
  },
  {
    name: routes.VerifyEmail,
    path: '/verify-email',
    component: () => import('@/pages/user/VerifyEmail.vue'),
    meta: { requiresAuth: false, fullScreen: true, title: 'Verify Email' },
  },
  {
    name: routes.CheckEmail,
    path: '/check-email',
    component: () => import('@/pages/CheckEmailPage.vue'),
    meta: { requiresAuth: false, fullScreen: true, title: 'Check Email' },
  },
  {
    name: routes.UserSettings,
    path: '/settings',
    component: () => import('@/pages/user/UserSettings.vue'),
    meta: { title: 'Settings' },
  },
  {
    name: routes.BeginResetPassword,
    path: '/begin-reset-password',
    component: () => import('@/pages/BeginResetPassword.vue'),
    meta: {
      requiresAuth: false,
      redirectIfLoggedIn: false,
      fullScreen: true,
      title: 'Reset Password',
    },
  },
  {
    name: routes.ResetPassword,
    path: '/reset-password',
    component: () => import('@/pages/ResetPassword.vue'),
    meta: {
      requiresAuth: false,
      redirectIfLoggedIn: false,
      fullScreen: true,
      title: 'Reset Password',
    },
  },
  {
    name: routes.OrganizationsNew,
    path: '/organizations/new',
    component: () => import('@/pages/organizations/OrganizationsEdit.vue'),
    meta: { title: 'New Organization' },
  },
  {
    name: routes.ServicesTokenList,
    path: '/organizations/:organization_id/services_tokens',
    component: () => import('@/pages/organizations/services_token/ServicesTokenList.vue'),
    meta: { title: 'Service Tokens' },
  },
  {
    name: routes.ServiceTokenView,
    path: '/organizations/:organization_id/services_tokens/:service_token_id',
    component: () => import('@/pages/organizations/services_token/ServiceTokenView.vue'),
    meta: { title: 'Service Token' },
  },
  {
    name: routes.OrganizationsTeam,
    path: '/organizations/:organization_id/members',
    component: () => import('@/pages/organizations/MembersList.vue'),
    meta: { title: 'Members' },
  },
  {
    name: routes.OrganizationsDetail,
    path: '/organizations/:organization_id/settings',
    component: () => import('@/pages/organizations/OrganizationsEdit.vue'),
    meta: { title: 'Organization Settings' },
  },
  {
    name: routes.OrganizationsDashboard,
    path: '/organizations/:organization_id/dashboard',
    component: () => import('@/pages/organizations/OrganizationsDashboard.vue'),
    meta: { title: 'Dashboard' },
  },
  {
    name: routes.ApplicationsList,
    path: '/organizations/:organization_id/applications',
    component: () => import('@/pages/organizations/applications/ApplicationsList.vue'),
    meta: { title: 'Applications' },
  },
  {
    name: routes.ApplicationsNew,
    path: '/organizations/:organization_id/applications/new',
    component: () => import('@/pages/organizations/applications/ApplicationsEdit.vue'),
    meta: { title: 'New Application' },
  },
  {
    name: routes.ApplicationsDashboard,
    path: '/organizations/:organization_id/applications/:application_id/dashboard',
    component: () => import('@/pages/organizations/applications/ApplicationsDashboard.vue'),
    meta: { title: 'Dashboard' },
  },
  {
    name: routes.ApplicationsDetail,
    path: '/organizations/:organization_id/applications/:application_id/settings',
    component: () => import('@/pages/organizations/applications/ApplicationsEdit.vue'),
    meta: { title: 'Application Settings' },
  },

  {
    name: routes.ApplicationSecretsList,
    path: '/organizations/:organization_id/applications/:application_id/application_secrets',
    component: () =>
      import('@/pages/organizations/applications/application_secrets/ApplicationSecretsList.vue'),
    meta: { title: 'API Keys' },
  },

  {
    name: routes.EventsList,
    path: '/organizations/:organization_id/applications/:application_id/events',
    component: () => import('@/pages/organizations/applications/events/EventsList.vue'),
    meta: { title: 'Events' },
  },

  {
    name: routes.EventsDetail,
    path: '/organizations/:organization_id/applications/:application_id/events/:event_id',
    component: () => import('@/pages/organizations/applications/events/EventsDetail.vue'),
    meta: { title: 'Event Detail' },
  },

  {
    name: routes.EventTypesList,
    path: '/organizations/:organization_id/applications/:application_id/event_types',
    component: () => import('@/pages/organizations/applications/event_types/EventTypesList.vue'),
    meta: { title: 'Event Types' },
  },
  {
    // EventTypes are immutable, they can only be created or removed
    name: routes.EventTypesNew,
    path: '/organizations/:organization_id/applications/:application_id/event_types/new',
    component: () => import('@/pages/organizations/applications/event_types/EventTypesNew.vue'),
    meta: { title: 'New Event Type' },
  },

  {
    name: routes.SubscriptionsList,
    path: '/organizations/:organization_id/applications/:application_id/subscriptions',
    component: () =>
      import('@/pages/organizations/applications/subscriptions/SubscriptionsList.vue'),
    meta: { title: 'Subscriptions' },
  },
  {
    name: routes.SubscriptionsNew,
    path: '/organizations/:organization_id/applications/:application_id/subscriptions/new',
    component: () =>
      import('@/pages/organizations/applications/subscriptions/SubscriptionsEdit.vue'),
    meta: { title: 'New Subscription' },
  },
  {
    name: routes.SubscriptionsDetail,
    path: '/organizations/:organization_id/applications/:application_id/subscriptions/:subscription_id',
    component: () =>
      import('@/pages/organizations/applications/subscriptions/SubscriptionsEdit.vue'),
    meta: { title: 'Subscription' },
  },
  {
    name: routes.LogsList,
    path: '/organizations/:organization_id/applications/:application_id/logs',
    component: () => import('@/pages/organizations/applications/logs/LogList.vue'),
    meta: { title: 'Delivery Logs' },
  },

  {
    name: routes.APIDocumentationForApplication,
    path: '/organizations/:organization_id/applications/:application_id/documentation',
    component: () => import('@/pages/api/documentation/ApiDocumentation.vue'),
    meta: { title: 'API Documentation' },
  },
  {
    name: routes.APIDocumentation,
    path: '/api/documentation',
    component: () => import('@/pages/api/documentation/ApiDocumentation.vue'),
    meta: { title: 'API Documentation' },
  },
  // Route for component visual testing (used by Playwright E2E tests)
  ...(import.meta.env.DEV
    ? [
        {
          name: 'ComponentShowcase',
          path: '/__dev/components',
          component: () => import('@/pages/ComponentShowcase.vue'),
          meta: { requiresAuth: false, fullScreen: true },
        },
      ]
    : []),

  {
    name: routes.Error404,
    path: '/:pathMatch(.*)*',
    component: () => import('@/pages/Error404.vue'),
    meta: { fullScreen: true, title: 'Not Found' },
  },
];
