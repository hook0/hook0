import { ref } from 'vue';
import { RouteLocationRaw } from 'vue-router';
import { ApplicationInfo } from '../organizations/applications/ApplicationService';
import { routes } from '@/routes';
import { OrganizationInfo } from '../organizations/OrganizationService';

export const progressItems = ref([
  { icon: 'sitemap', title: 'Organization' },
  { icon: 'rocket', title: 'Application' },
  { icon: 'folder-tree', title: 'Event Type' },
  { icon: 'link', title: 'Subscription' },
  { icon: 'file-lines', title: 'Event' },
]);

export interface Step {
  title: string;
  details: string;
  isActive: boolean;
  icon?: string;
  route?: RouteLocationRaw;
}

const enum OnboardingStepStatus {
  ToDo = 'ToDo',
  Done = 'Done',
}

function parseOnboardingStep(str: string): OnboardingStepStatus {
  return str === 'Done' ? OnboardingStepStatus.Done : OnboardingStepStatus.ToDo;
}

function tutorialAppRoute(organization_id: string) {
  return {
    name: routes.TutorialCreateApplication,
    params: {
      organization_id: organization_id,
    },
  };
}

function applicationStep(organization_id: string, isActive: boolean): Step {
  return {
    title: 'Create an application',
    details: 'You can create as many applications as you need.',
    isActive,
    icon: 'rocket',
    route: {
      name: routes.TutorialCreateApplication,
      params: {
        organization_id,
      },
    },
  };
}

function eventTypeStep(
  organization_id: string,
  application_id: string | null,
  isActive: boolean
): Step {
  return {
    title: 'Create an event type',
    details:
      'Event types are categories of events. For each subscription, you will then be able choose among your declared event types to receive only the right events.',
    isActive,
    icon: 'folder-tree',
    route:
      application_id !== null
        ? {
            name: routes.TutorialCreateEventType,
            params: {
              organization_id,
              application_id,
            },
          }
        : tutorialAppRoute(organization_id),
  };
}

function subscriptionStep(
  organization_id: string,
  application_id: string | null,
  isActive: boolean
): Step {
  return {
    title: 'Create a subscription',
    details: 'You can create as many subscriptions as you need.',
    isActive,
    icon: 'link',
    route:
      application_id !== null
        ? {
            name: routes.TutorialCreateSubscription,
            params: {
              organization_id,
              application_id,
            },
          }
        : tutorialAppRoute(organization_id),
  };
}

function eventStep(
  organization_id: string,
  application_id: string | null,
  isActive: boolean
): Step {
  return {
    title: 'Send an event',
    details: 'You can send as many events as you need.',
    isActive,
    icon: 'file-lines',
    route:
      application_id !== null
        ? {
            name: routes.TutorialSendEvent,
            params: {
              organization_id,
              application_id,
            },
          }
        : tutorialAppRoute(organization_id),
  };
}

export function organizationSteps(organization: OrganizationInfo): Step[] {
  const applicationStepIsActive =
    parseOnboardingStep(organization.onboarding_steps.application) === OnboardingStepStatus.Done;
  const eventTypeStepIsActive =
    parseOnboardingStep(organization.onboarding_steps.event_type) === OnboardingStepStatus.Done;
  const subscriptionStepIsActive =
    parseOnboardingStep(organization.onboarding_steps.subscription) === OnboardingStepStatus.Done;
  const eventStepIsActive =
    parseOnboardingStep(organization.onboarding_steps.event) === OnboardingStepStatus.Done;

  return applicationStepIsActive &&
    eventStepIsActive &&
    subscriptionStepIsActive &&
    eventStepIsActive
    ? []
    : [
        applicationStep(organization.organization_id, applicationStepIsActive),
        eventTypeStep(organization.organization_id, null, eventTypeStepIsActive),
        subscriptionStep(organization.organization_id, null, subscriptionStepIsActive),
        eventStep(organization.organization_id, null, eventStepIsActive),
      ];
}

export function applicationSteps(application: ApplicationInfo): Step[] {
  const eventTypeStepIsActive =
    parseOnboardingStep(application.onboarding_steps.event_type) === OnboardingStepStatus.Done;
  const subscriptionStepIsActive =
    parseOnboardingStep(application.onboarding_steps.subscription) === OnboardingStepStatus.Done;
  const eventStepIsActive =
    parseOnboardingStep(application.onboarding_steps.event) === OnboardingStepStatus.Done;

  return eventStepIsActive && subscriptionStepIsActive && eventStepIsActive
    ? []
    : [
        eventTypeStep(
          application.organization_id,
          application.application_id,
          eventTypeStepIsActive
        ),
        subscriptionStep(
          application.organization_id,
          application.application_id,
          subscriptionStepIsActive
        ),
        eventStep(application.organization_id, application.application_id, eventStepIsActive),
      ];
}
