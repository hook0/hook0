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

type ApplicationsPerOrganization = {
  organization: Organization;
  applications: Array<Application>;
};

const router = useRouter();
const route = useRoute();

const applicationsPerOrganization = ref<null | ApplicationsPerOrganization[]>(null);
const organization_name = ref('');
const application_name = ref('');
const removeRouterGuard = ref<null | (() => void)>(null);

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
  <Hook0Dropdown
    v-if="applicationsPerOrganization !== null"
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
      <div>
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
