import { markRaw, type Component } from 'vue';
import type { RouteLocationRaw } from 'vue-router';
import { AppWindow, FolderTree, Link, FileText } from 'lucide-vue-next';
import type { ApplicationInfo } from '@/pages/organizations/applications/ApplicationService';
import { routes } from '@/routes';
import type { OrganizationInfo } from '@/pages/organizations/OrganizationService';

export type Step = {
  title: string;
  details: string;
  explanation?: string;
  isCompleted: boolean;
  icon?: Component;
  route?: RouteLocationRaw;
};

const OnboardingStepStatus = {
  ToDo: 'ToDo',
  Done: 'Done',
} as const;

type OnboardingStepStatus = (typeof OnboardingStepStatus)[keyof typeof OnboardingStepStatus];

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

function applicationStep(organization_id: string, isCompleted: boolean): Step {
  return {
    title: 'tutorial.widget.applicationTitle',
    details: 'tutorial.widget.applicationDetails',
    isCompleted,
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
  isCompleted: boolean
): Step {
  return {
    title: 'tutorial.widget.eventTypeTitle',
    details: 'tutorial.widget.eventTypeDetails',
    explanation: 'tutorial.widget.eventTypeExplanation',
    isCompleted,
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
  isCompleted: boolean
): Step {
  return {
    title: 'tutorial.widget.subscriptionTitle',
    details: 'tutorial.widget.subscriptionDetails',
    explanation: 'tutorial.widget.subscriptionExplanation',
    isCompleted,
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
  isCompleted: boolean
): Step {
  return {
    title: 'tutorial.widget.eventTitle',
    details: 'tutorial.widget.eventDetails',
    explanation: 'tutorial.widget.eventExplanation',
    isCompleted,
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
    eventTypeStepIsActive &&
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

  return eventTypeStepIsActive && subscriptionStepIsActive && eventStepIsActive
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
