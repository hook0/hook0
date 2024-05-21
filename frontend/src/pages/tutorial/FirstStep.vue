<script setup lang="ts">
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import OrganizationsEdit from '@/pages/organizations/OrganizationsEdit.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import { ref } from 'vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { routes } from '@/routes.ts';
import { useRouter } from 'vue-router';
import { UUID } from '@/http.ts';
import Hook0Code from '@/components/Hook0Code.vue';
import Hook0Text from '@/components/Hook0Text.vue';

const router = useRouter();

const organizationId = ref<UUID | null>(null);

const goSecondStep = () => {
  if (organizationId.value) {
    return router.push({
      name: routes.TutorialStep2,
      params: { organization_id: organizationId.value },
    });
  }
};
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>Step 1: Create your first organization</template>
      <template #subtitle
        >Organizations are used to group your applications and environments. You can create multiple
        organizations to separate your projects. Like production, staging, and development.
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLines>
        <Hook0CardContentLine type="full-width">
          <template v-if="!organizationId" #content>
            <OrganizationsEdit
              :tutorial-mode="true"
              @tutorial-organization-created="organizationId = $event"
            />
          </template>
        </Hook0CardContentLine>
      </Hook0CardContentLines>
    </Hook0CardContent>
    <Hook0CardFooter>
      <Hook0Button class="primary" type="button" :disabled="!organizationId" @click="goSecondStep"
        >Next</Hook0Button
      >
    </Hook0CardFooter>
  </Hook0Card>
</template>
