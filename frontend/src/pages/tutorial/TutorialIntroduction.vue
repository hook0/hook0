<script setup lang="ts">
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0IconBadge from '@/components/Hook0IconBadge.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0Illustration from '@/components/Hook0Illustration.vue';
import Hook0ListItem from '@/components/Hook0ListItem.vue';
import { useRouter } from 'vue-router';
import { routes } from '@/routes.ts';
import { useTracking } from '@/composables/useTracking';
import { useI18n } from 'vue-i18n';
import {
  Rocket,
  Building2,
  AppWindow,
  FolderTree,
  Link,
  FileText,
  ArrowRight,
  X,
} from 'lucide-vue-next';

const { t } = useI18n();

const router = useRouter();

// Analytics tracking
const { trackEvent } = useTracking();

interface TutorialStep {
  icon: typeof Building2;
  label: string;
}

const tutorialSteps: TutorialStep[] = [
  { icon: Building2, label: 'tutorial.intro.anOrganization' },
  { icon: AppWindow, label: 'tutorial.intro.anApplication' },
  { icon: FolderTree, label: 'tutorial.intro.anEventType' },
  { icon: Link, label: 'tutorial.intro.aSubscription' },
  { icon: FileText, label: 'tutorial.intro.anEvent' },
];

function startTutorial() {
  trackEvent('tutorial', 'start');
  return router.push({
    name: routes.TutorialCreateOrganization,
  });
}

function skipTutorial() {
  trackEvent('tutorial', 'skip');
  return router.push({ name: routes.Home });
}
</script>

<template>
  <Hook0Stack direction="column" gap="none">
    <Hook0Card data-test="tutorial-card">
      <Hook0CardHeader data-test="tutorial-header">
        <template #header>
          <Hook0Stack direction="row" align="center" gap="sm">
            <Hook0IconBadge variant="primary" size="md">
              <Rocket :size="18" aria-hidden="true" />
            </Hook0IconBadge>
            <Hook0Stack direction="row" align="center" gap="none">
              {{ t('tutorial.intro.title') }}
            </Hook0Stack>
          </Hook0Stack>
        </template>
        <template #subtitle>{{ t('tutorial.intro.subtitle') }}</template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <Hook0Stack direction="column" gap="lg">
              <Hook0Illustration
                variant="tutorial"
                size="hero"
                :alt="t('tutorial.intro.illustrationAlt')"
              />
              <Hook0Stack direction="column" gap="md">
                {{ t('tutorial.intro.inThisTutorial') }}
              </Hook0Stack>
              <Hook0Stack direction="column" gap="none">
                <Hook0ListItem v-for="step in tutorialSteps" :key="step.label">
                  <template #icon>
                    <component :is="step.icon" :size="16" aria-hidden="true" />
                  </template>
                  <template #left>
                    <Hook0Stack direction="row" align="center" gap="none">
                      {{ t(step.label) }}
                    </Hook0Stack>
                  </template>
                </Hook0ListItem>
              </Hook0Stack>
              <Hook0Stack direction="column" gap="none">
                <i18n-t keypath="tutorial.intro.skipNote" tag="span">
                  <template #link>
                    <Hook0Button
                      variant="link"
                      target="_blank"
                      href="https://documentation.hook0.com/docs/getting-started"
                      >{{ t('tutorial.intro.programmatically') }}</Hook0Button
                    >
                  </template>
                </i18n-t>
              </Hook0Stack>
            </Hook0Stack>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContent>
      <Hook0CardFooter>
        <Hook0Stack direction="row" align="center" gap="md">
          <Hook0Button
            variant="secondary"
            type="button"
            data-test="tutorial-skip-button"
            @click="skipTutorial"
          >
            <X :size="16" aria-hidden="true" />
            {{ t('tutorial.intro.skipButton') }}
          </Hook0Button>
          <Hook0Button
            variant="primary"
            type="button"
            data-test="tutorial-start-button"
            @click="startTutorial"
          >
            {{ t('tutorial.intro.startStep1') }}
            <ArrowRight :size="16" aria-hidden="true" />
          </Hook0Button>
        </Hook0Stack>
      </Hook0CardFooter>
    </Hook0Card>
  </Hook0Stack>
</template>

<style scoped>
/* No custom styles - using Hook0* components only */
</style>
