import { markRaw, ref, type Component } from 'vue';
import { RouteLocationRaw } from 'vue-router';
import { Building2, AppWindow, FolderTree, Link, FileText } from 'lucide-vue-next';
import { ApplicationInfo } from '../organizations/applications/ApplicationService';
import { routes } from '@/routes';
import { OrganizationInfo } from '../organizations/OrganizationService';

export const progressItems = ref([
  { icon: markRaw(Building2), title: 'Organization' },
  { icon: markRaw(AppWindow), title: 'Application' },
  { icon: markRaw(FolderTree), title: 'Event Type' },
  { icon: markRaw(Link), title: 'Subscription' },
  { icon: markRaw(FileText), title: 'Event' },
]);

export interface Step {
  title: string;
  details: string;
  isActive: boolean;
  icon?: Component;
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
    details: 'Isolated environment that contains everything webhook-related.',
    isActive,
    icon: markRaw(AppWindow),
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
    details: 'Category of events you can pick or not when subscribing.',
    isActive,
    icon: markRaw(FolderTree),
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
    details: 'Filter events and choose where you want them dispatched as webhooks.',
    isActive,
    icon: markRaw(Link),
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
    details: 'Something that happened in your application and could end up as a webhook.',
    isActive,
    icon: markRaw(FileText),
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
