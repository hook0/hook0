<script setup lang="ts">
import * as Option from 'fp-ts/Option';
import { flow } from 'fp-ts/function';
import { RouteLocationRaw, RouteParams, useRoute, useRouter } from 'vue-router';
import { onMounted, onUnmounted, onUpdated, ref, watch } from 'vue';

import * as OrganizationService from './organizations/OrganizationService';
import * as ApplicationService from './organizations/applications/ApplicationService';
import { UUID } from '@/http';
import { Organization } from './organizations/OrganizationService';
import { Application } from './organizations/applications/ApplicationService';
import { routes } from '@/routes';
import Hook0DropdownOptions from '@/components/Hook0DropdownOptions';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0DropdownMenuItemLink from '@/components/Hook0DropdownMenuItemLink.vue';
import Hook0Dropdown from '@/components/Hook0Dropdown.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Icon from '@/components/Hook0Icon.vue';
import { getAccessToken } from '@/iam';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import { isPricingEnabled } from '@/instance';

type ApplicationsPerOrganization = {
  organization: Organization;
  applications: Array<Application>;
};

const router = useRouter();
const route = useRoute();

const pricingEnabled = ref<boolean>(false);

const applicationsPerOrganization = ref<null | ApplicationsPerOrganization[]>(null);
const organization_name = ref('');
const application_name = ref('');
const removeRouterGuard = ref<null | (() => void)>(null);

const props = defineProps<{
  displayAsCards: boolean;
}>();

watch(
  () => getAccessToken().value,
  async (newToken, oldToken) => {
    if (newToken !== oldToken) {
      if (newToken) {
        applicationsPerOrganization.value = await getApplicationsPerOrganization();
      } else {
        applicationsPerOrganization.value = null;
      }
    }
  },
  { deep: true }
);

function getApplicationsPerOrganization(): Promise<ApplicationsPerOrganization[]> {
  return OrganizationService.list().then((organizations) =>
    Promise.all(
      organizations.map((organization) => ApplicationService.list(organization.organization_id))
    ).then((applications) => {
      return applications.reduce(
        (m, applications) => {
          return applications.reduce((m, application) => {
            const organization = organizations.find(
              (org) => org.organization_id === application.organization_id
            );

            if (!organization) {
              console.error(
                'should never happen, application is linkedin to unknown organization. Silent fail'
              );
              return m;
            }

            let organization_in_group = m.find(
              (item) => item.organization.organization_id === application.organization_id
            );

            if (!organization_in_group) {
              console.error(
                'should never happen, application is linkedin to unknown organization. Silent fail'
              );
              return m;
            }

            organization_in_group.applications.push(application);

            return m;
          }, m);
        },
        organizations.map((organization) => {
          return {
            organization: organization,
            applications: [],
          };
        }) as ApplicationsPerOrganization[]
      );
    })
  );
}

function goto(parent: Hook0DropdownOptions, route: RouteLocationRaw) {
  parent.close();
  return router.push(route);
}

function _updateDropdown(params: RouteParams) {
  if (applicationsPerOrganization.value !== null) {
    const organizationGroup = applicationsPerOrganization.value;
    const org_name = flow(
      Option.map((_organization_id) =>
        Option.fromNullable(
          organizationGroup.find(
            (group) => group.organization.organization_id === params.organization_id
          )
        )
      ),
      Option.flatten,
      Option.map((organizationGroup) => organizationGroup.organization.name)
    )(Option.fromNullable(params.organization_id as UUID));

    organization_name.value = Option.getOrElse(() => '')(org_name);

    const app_name = flow(Option.map((application: Application) => application.name))(
      Option.fromNullable(
        organizationGroup
          .flatMap((group) => group.applications)
          .find((application) => application.application_id === params.application_id)
      )
    );

    application_name.value = Option.getOrElse(() => 'Select an application')(app_name);
  }
}

onMounted(async () => {
  if (getAccessToken().value) {
    applicationsPerOrganization.value = await getApplicationsPerOrganization();
  }

  removeRouterGuard.value = router.afterEach(() => {
    return _updateDropdown(route.params);
  });

  pricingEnabled.value = await isPricingEnabled();

  return _updateDropdown(route.params);
});

onUpdated(() => {
  return _updateDropdown(route.params);
});

onUnmounted(() => {
  if (removeRouterGuard.value !== null) {
    removeRouterGuard.value();
  }
});
</script>

<template>
  <template v-if="applicationsPerOrganization !== null && props.displayAsCards">
    <div class="flex flex-col justify-between p-4">
      <div>
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 auto-rows-fr">
          <div
            v-for="(organizationGroup, index) in applicationsPerOrganization"
            :key="index"
            class="flex flex-col mb-2"
          >
            <Hook0Card class="h-full">
              <Hook0CardHeader>
                <template #header>
                  <Hook0Button
                    class="flex items-center text-lg"
                    @click="
                      router.push({
                        name: routes.OrganizationsDashboard,
                        params: { organization_id: organizationGroup.organization.organization_id },
                      })
                    "
                  >
                    <Hook0Icon name="sitemap"></Hook0Icon>
                    <Hook0Text class="ml-1">{{ organizationGroup.organization.name }}</Hook0Text>
                    <template v-if="pricingEnabled">
                      <span
                        v-if="organizationGroup.organization.plan"
                        class="ml-2 inline-flex items-center rounded-md bg-blue-50 px-2 py-1 text-xs font-medium text-blue-700 ring-1 ring-inset ring-blue-700/10"
                        :title="'Plan: ' + organizationGroup.organization.plan?.label || ''"
                        >{{ organizationGroup.organization.plan?.label || '' }}</span
                      >
                      <span
                        v-else
                        class="ml-2 inline-flex items-center rounded-md bg-gray-50 px-2 py-1 text-xs font-medium text-gray-600 ring-1 ring-inset ring-gray-500/10"
                        title="Plan: Developer"
                        >Developer</span
                      >
                    </template>
                  </Hook0Button>
                </template>
                <template #subtitle> </template>
              </Hook0CardHeader>
              <Hook0CardContentLines>
                <Hook0CardContentLine type="full-width">
                  <template v-if="organizationGroup.applications.length > 0" #content>
                    <Hook0Button
                      v-for="(application, appIndex) in organizationGroup.applications"
                      :key="appIndex"
                      class="flex items-center"
                      @click="
                        router.push({
                          name: routes.ApplicationsDashboard,
                          params: {
                            application_id: application.application_id,
                            organization_id: organizationGroup.organization.organization_id,
                          },
                        })
                      "
                    >
                      <Hook0Text class="ml-1">{{ application.name }}</Hook0Text>
                    </Hook0Button>
                  </template>
                  <template v-else #content>
                    <div class="flex flex-col items-center">
                      <Hook0Text class="text-gray-500 mb-2">No application found</Hook0Text>
                      <Hook0Button
                        @click="
                          router.push({
                            name: routes.ApplicationsNew,
                            params: {
                              organization_id: organizationGroup.organization.organization_id,
                            },
                          })
                        "
                      >
                        <Hook0Icon name="plus"></Hook0Icon>
                        <Hook0Text class="ml-1">Create New Application</Hook0Text>
                      </Hook0Button>
                    </div>
                  </template>
                </Hook0CardContentLine>
              </Hook0CardContentLines>
            </Hook0Card>
          </div>
        </div>
      </div>
      <div class="flex justify-end">
        <Hook0Button class="primary" @click="router.push({ name: routes.OrganizationsNew })">
          <Hook0Icon name="plus"></Hook0Icon>
          <Hook0Text class="ml-1">New Organization</Hook0Text>
        </Hook0Button>
      </div>
    </div>
  </template>
  <Hook0Dropdown
    v-else-if="applicationsPerOrganization !== null && !props.displayAsCards"
    class="container darkmode"
    justify="left"
  >
    <template #menu="parent">
      <Hook0Button class="dropdown" @click="parent.toggle">
        <template #default>
          <div class="flex flex-col">
            <Hook0Text class="def">{{ organization_name }}</Hook0Text>
            <Hook0Text class="darkmode">{{ application_name }}</Hook0Text>
          </div>
        </template>
        <template #right>
          <Hook0Icon name="chevron-down" class="darkmode"></Hook0Icon>
        </template>
      </Hook0Button>
    </template>

    <template #dropdown="parent">
      <div class="max-h-96 overflow-y-scroll">
        <div v-for="(organizationGroup, index) in applicationsPerOrganization" :key="index">
          <Hook0DropdownMenuItemLink
            class="flex justify-between darkmode"
            @click="
              goto(parent, {
                name: routes.OrganizationsDashboard,
                params: { organization_id: organizationGroup.organization.organization_id },
              })
            "
          >
            <Hook0Icon name="sitemap" class="darkmode"></Hook0Icon>
            <Hook0Text class="ml-1 darkmode">{{ organizationGroup.organization.name }}</Hook0Text>
          </Hook0DropdownMenuItemLink>

          <div class="pl-2">
            <Hook0DropdownMenuItemLink
              v-for="(application, appIndex) in organizationGroup.applications"
              :key="appIndex"
              class="darkmode"
              @click="
                goto(parent, {
                  name: routes.ApplicationsDashboard,
                  params: {
                    application_id: application.application_id,
                    organization_id: organizationGroup.organization.organization_id,
                  },
                })
              "
            >
              <Hook0Text class="ml-1 darkmode">{{ application.name }}</Hook0Text>
              <Hook0Text class="ml-1 def darkmode">application</Hook0Text>
            </Hook0DropdownMenuItemLink>
          </div>
        </div>
      </div>
      <Hook0DropdownMenuItemLink :to="{ name: routes.OrganizationsNew }" class="darkmode">
        <Hook0Icon name="plus" class="darkmode"></Hook0Icon>
        <Hook0Text class="ml-1 darkmode">New Organization</Hook0Text>
      </Hook0DropdownMenuItemLink>
    </template>
  </Hook0Dropdown>
</template>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style lang="scss" scoped>
.container {
  height: 67px;
  @apply max-w-lg block w-full cursor-pointer;

  &.loader {
    @apply flex flex-grow justify-center items-center;
  }
}
</style>
