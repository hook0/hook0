import { createRouter, createWebHistory } from 'vue-router';
import routes from '@/routes';

const router = createRouter({
  // Provide the history implementation to use
  history: createWebHistory(),
  routes,
});

/**
 * Manual route-name-to-page-title map. Must be updated when routes are added, renamed, or removed.
 */
const routeTitles: Record<string, string> = {
  Home: 'Home',
  Tutorial: 'Tutorial',
  TutorialCreateOrganization: 'Tutorial — Organization',
  TutorialCreateApplication: 'Tutorial — Application',
  TutorialCreateEventType: 'Tutorial — Event Type',
  TutorialCreateSubscription: 'Tutorial — Subscription',
  TutorialSendEvent: 'Tutorial — Send Event',
  TutorialSuccess: 'Tutorial — Success',
  Login: 'Login',
  Register: 'Register',
  VerifyEmail: 'Verify Email',
  CheckEmail: 'Check Email',
  UserSettings: 'Settings',
  BeginResetPassword: 'Reset Password',
  ResetPassword: 'Reset Password',
  OrganizationsNew: 'New Organization',
  OrganizationsDashboard: 'Dashboard',
  OrganizationsTeam: 'Members',
  OrganizationsDetail: 'Organization Settings',
  ServicesTokenList: 'Service Tokens',
  ServiceTokenView: 'Service Token',
  ApplicationsList: 'Applications',
  ApplicationsNew: 'New Application',
  ApplicationsDashboard: 'Dashboard',
  ApplicationsDetail: 'Application Settings',
  ApplicationSecretsList: 'API Keys',
  EventsList: 'Events',
  EventsDetail: 'Event Detail',
  EventTypesList: 'Event Types',
  EventTypesNew: 'New Event Type',
  SubscriptionsList: 'Subscriptions',
  SubscriptionsNew: 'New Subscription',
  SubscriptionsDetail: 'Subscription',
  LogsList: 'Delivery Logs',
  APIDocumentationForApplication: 'API Documentation',
  APIDocumentation: 'API Documentation',
  Error404: 'Not Found',
};

router.afterEach((to) => {
  const title = routeTitles[to.name as string];
  document.title = title ? `${title} — Hook0` : 'Hook0';
});

export default router;
