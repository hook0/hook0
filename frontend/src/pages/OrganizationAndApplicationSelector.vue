<template>
  <Promised :promise="applicationsPerOrganization$">
    <!-- Use the "pending" slot to display a loading message -->

    <template #pending>
      <div class="container loader">
        <hook0-loader></hook0-loader>
      </div>
    </template>

    <!-- The default scoped slot will be used as the result -->
    <template #default="applicationsPerOrganizations">
      <hook0-dropdown class="container darkmode" justify="left">
        <template v-slot:menu="parent">
          <hook0-button class="dropdown" @click="parent.toggle">
            <template #default>
              <div class="flex flex-col">
                <hook0-text class="def">{{ organization_name }}</hook0-text>
                <hook0-text class="darkmode">{{ application_name }}</hook0-text>
              </div>
            </template>
            <template #right>
              <hook0-icon name="chevron-down" class="darkmode"></hook0-icon>
            </template>
          </hook0-button>
        </template>

        <template v-slot:dropdown="parent">
          <div>
            <div v-for="(organizationGroup, index) in applicationsPerOrganizations" :key="index">
              <hook0-dropdown-menu-item-link
                class="flex justify-between darkmode"
                @click="
                  goto(parent, {
                    name: routes.OrganizationsDashboard,
                    params: { organization_id: organizationGroup.organization.organization_id },
                  })
                "
              >
                <hook0-icon name="sitemap" class="darkmode"></hook0-icon>
                <hook0-text class="ml-1 darkmode">{{
                  organizationGroup.organization.name
                }}</hook0-text>
              </hook0-dropdown-menu-item-link>

              <div class="pl-2">
                <hook0-dropdown-menu-item-link
                  class="darkmode"
                  v-for="(application, index) in organizationGroup.applications"
                  :key="index"
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
                  <hook0-text class="ml-1 darkmode">{{ application.name }}</hook0-text>
                  <hook0-text class="ml-1 def darkmode">application</hook0-text>
                </hook0-dropdown-menu-item-link>
              </div>
            </div>
          </div>
          <hook0-dropdown-menu-item-link :to="{ name: routes.OrganizationsNew }" class="darkmode">
            <hook0-icon name="plus" class="darkmode"></hook0-icon>
            <hook0-text class="ml-1 darkmode">New Organization</hook0-text>
          </hook0-dropdown-menu-item-link>
        </template>
      </hook0-dropdown>
    </template>

    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <p>Error: {{ error }}</p>
    </template>
  </Promised>
</template>

<script lang="ts">
import * as OrganizationService from './organizations/OrganizationService';
import * as ApplicationService from './organizations/applications/ApplicationService';
import { Options, Vue } from 'vue-class-component';
import { UUID } from '@/http';
import { Organization } from './organizations/OrganizationService';
import { Application } from './organizations/applications/ApplicationService';
import { routes } from '@/routes';
import * as Option from 'fp-ts/Option';
import { flow } from 'fp-ts/function';
import { RouteLocation, RouteParams } from 'vue-router';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0DropdownOptions from '@/components/Hook0DropdownOptions';

type ApplicationsPerOrganization = {
  organization: Organization;
  applications: Array<Application>;
};

@Options({
  components: { Hook0Loader },
})
export default class OrganizationSelector extends Vue {
  private applicationsPerOrganization$!: Promise<ApplicationsPerOrganization[]>;
  private routes = routes;
  private organization_name = '';
  private application_name = '';
  private removeRouterGuard!: () => void;

  getApplicationsPerOrganization(): Promise<ApplicationsPerOrganization[]> {
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
              // @ts-ignore
              organization: organization,
              applications: [],
            };
          }) as ApplicationsPerOrganization[]
        );
      })
    );
  }

  goto(parent: Hook0DropdownOptions, route: RouteLocation) {
    parent.close();
    return this.$router.push(route);
  }

  _updateDropdown(params: RouteParams) {
    return (this as OrganizationSelector).applicationsPerOrganization$.then(
      (organizationGroup: ApplicationsPerOrganization[]) => {
        const organization_name = flow(
          Option.map((organization_id) =>
            Option.fromNullable(
              organizationGroup.find(
                (group) => group.organization.organization_id === params.organization_id
              )
            )
          ),
          Option.flatten,
          Option.map((organizationGroup) => organizationGroup.organization.name)
        )(Option.fromNullable(params.organization_id as UUID));

        this.organization_name = Option.getOrElse(() => '')(organization_name);

        const application_name = flow(Option.map((application: Application) => application.name))(
          Option.fromNullable(
            organizationGroup
              .flatMap((group) => group.applications)
              .find((application) => application.application_id === params.application_id)
          )
        );

        this.application_name = Option.getOrElse(() => 'Select an application')(application_name);
      }
    );
  }

  created(): void {
    this.applicationsPerOrganization$ = this.getApplicationsPerOrganization();
  }

  updated() {
    return this._updateDropdown(this.$route.params);
  }

  mounted() {
    this.removeRouterGuard = this.$router.afterEach((to, from) => {
      return this._updateDropdown(this.$route.params);
    });

    return this._updateDropdown(this.$route.params);
  }

  unmounted() {
    this.removeRouterGuard();
  }
}
</script>

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
