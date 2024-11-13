<script setup lang="ts">
import { onMounted, onUpdated, ref } from 'vue';

import * as OrganizationService from './OrganizationService';
import { OrganizationInfo } from './OrganizationService';
import { Problem, UUID } from '@/http';
import OrganizationRemove from './OrganizationsRemove.vue';
import { useRoute, useRouter } from 'vue-router';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import { push } from 'notivue';
import { routes } from '@/routes.ts';

const router = useRouter();
const route = useRoute();

const isNew = ref(true);
const loading = ref(false);
const organization_id = ref<UUID | null>(null);
const organization = ref({
  name: '',
});

function _load() {
  if (organization_id.value !== route.params.organization_id) {
    organization_id.value = route.params.organization_id as UUID;
    isNew.value = !organization_id.value;

    if (!isNew.value) {
      OrganizationService.get(organization_id.value)
        .then((org: OrganizationInfo) => {
          organization.value.name = org.name;
        })
        .catch(displayError);
    }
  }
}

function upsert(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();

  loading.value = true;

  (isNew.value
    ? // create
      OrganizationService.create({
        name: organization.value.name,
      })
        .then((org) => {
          push.success({
            title: 'Organization created',
            message: `Organization ${org.name} has been created`,
            duration: 5000,
          });
          return router.push({
            name: routes.OrganizationsDashboard,
            params: { organization_id: org.organization_id },
          });
        })
        .catch(displayError)
    : // update
      OrganizationService.update(route.params.organization_id as string, {
        name: organization.value.name,
      })
        .then(() => {
          push.success({
            title: 'Organization updated',
            message: `Organization ${organization.value.name} has been updated`,
            duration: 5000,
          });
          return router.push({
            name: routes.OrganizationsDashboard,
            params: { organization_id: route.params.organization_id },
          });
        })
        .catch(displayError)
  )
    // finally
    .finally(() => (loading.value = false));
}

function displayError(err: Problem) {
  console.error(err);
  let options = {
    title: err.title,
    message: err.detail,
    duration: 5000,
  };
  err.status >= 500 ? push.error(options) : push.warning(options);
}

onMounted(() => {
  _load();
});

onUpdated(() => {
  _load();
});
</script>

<template>
  <div>
    <form ref="form" @submit="upsert">
      <Hook0Card>
        <Hook0CardHeader>
          <template v-if="isNew" #header> Create new organization </template>
          <template v-else #header> Edit organization </template>
          <template #subtitle> An organization holds your team members </template>
        </Hook0CardHeader>
        <Hook0CardContent>
          <Hook0CardContentLine>
            <template #label> Organization Name </template>
            <template #content>
              <Hook0Input
                v-model="organization.name"
                type="text"
                placeholder="my awesome api - production"
                required
              >
                <template #helpText></template>
              </Hook0Input>
            </template>
          </Hook0CardContentLine>
        </Hook0CardContent>

        <Hook0CardFooter>
          <Hook0Button class="secondary" type="button" @click="$router.back()">Cancel</Hook0Button>
          <Hook0Button class="primary" type="button" :loading="loading" @click="upsert($event)"
            >{{ isNew ? 'Create' : 'Update' }}
          </Hook0Button>
        </Hook0CardFooter>
      </Hook0Card>
    </form>

    <OrganizationRemove
      v-if="!isNew"
      :organization-id="
        Array.isArray(route.params.organization_id)
          ? route.params.organization_id[0]
          : route.params.organization_id
      "
      :organization-name="organization.name"
    ></OrganizationRemove>
  </div>
</template>
