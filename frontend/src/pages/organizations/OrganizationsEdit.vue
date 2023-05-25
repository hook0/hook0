<template>
  <div>
    <form @submit="upsert" ref="form">
      <hook0-card>
        <hook0-card-header>
          <template #header v-if="isNew"> Create new organization </template>
          <template #header v-else> Edit organization </template>
          <template #subtitle> An organization holds your team members </template>
        </hook0-card-header>
        <hook0-card-content>
          <hook0-card-content-line>
            <template #label> Organization Name </template>
            <template #content>
              <hook0-input
                type="text"
                v-model="organization.name"
                placeholder="my awesome api - production"
                required
              >
                <template #helpText></template>
              </hook0-input>
            </template>
          </hook0-card-content-line>
        </hook0-card-content>

        <hook0-card-content v-if="alert.visible">
          <hook0-alert
            :type="alert.type"
            :title="alert.title"
            :description="alert.description"
          ></hook0-alert>
        </hook0-card-content>
        <hook0-card-footer>
          <hook0-button class="secondary" type="button" @click="$router.back()"
            >Cancel</hook0-button
          >
          <hook0-button class="primary" type="button" :loading="loading" @click="upsert($event)"
            >{{ isNew ? 'Create' : 'Update' }}
          </hook0-button>
        </hook0-card-footer>
      </hook0-card>
    </form>

    <OrganizationRemove
      v-if="!isNew"
      :organization-id="$route.params.organization_id"
      :organization-name="organization.name"
    ></OrganizationRemove>
  </div>
</template>

<script lang="ts">
import { AxiosError } from 'axios';
import * as OrganizationService from './OrganizationService';
import { Organization, OrganizationPost } from './OrganizationService';
import { Options, Vue } from 'vue-class-component';
import { routes } from '@/routes';
import Hook0Alert from '@/components/Hook0Alert.vue';

import type { components } from '@/types';

type definitions = components['schemas'];
import { isAxiosError, Problem, UUID } from '@/http';
import { Alert } from '@/components/Hook0Alert';
import OrganizationRemove from './OrganizationsRemove.vue';

@Options({
  components: {
    Hook0Alert,
    OrganizationRemove,
  },
})
export default class OrganizationEdit extends Vue {
  private isNew = true;
  private loading = false;

  organization_id: UUID | undefined;

  routes = routes;

  organization = {
    name: '',
  };

  alert: Alert = {
    visible: false,
    type: 'alert',
    title: '',
    description: '',
  };

  mounted() {
    this._load();
  }

  updated() {
    this._load();
  }

  _load() {
    if (this.organization_id !== this.$route.params.organization_id) {
      this.organization_id = this.$route.params.organization_id as UUID;
      this.isNew = !this.organization_id;

      if (!this.isNew) {
        OrganizationService.get(this.organization_id)
          .then((organization: Organization) => {
            this.organization.name = organization.name;
          })
          .catch(this.displayError.bind(this));
      }
    }
  }

  reloadPageAndGoToOrganizationDetail(organization_id: string) {
    const href = this.$router.resolve({
      name: routes.OrganizationsDashboard,
      params: {
        organization_id: organization_id,
      },
    }).href;
    window.location.assign(href);
  }

  upsert(e: Event) {
    e.preventDefault();
    e.stopImmediatePropagation();

    this.alert.visible = false; // reset alert
    this.loading = true;

    (this.isNew
      ? // create
        OrganizationService.create({
          name: this.organization.name,
        })
          .then((organization) =>
            this.reloadPageAndGoToOrganizationDetail(organization.organization_id)
          )
          .catch(this.displayError.bind(this))
      : // update
        OrganizationService.update(this.$route.params.organization_id as string, {
          name: this.organization.name,
        })
          .then((_resp: any) =>
            this.reloadPageAndGoToOrganizationDetail(this.$route.params.organization_id as string)
          )
          .catch(this.displayError.bind(this))
    )
      // finally
      .finally(() => (this.loading = false));
  }

  displayError(err: AxiosError | unknown) {
    console.error(err);
    this.alert.visible = true;

    if (isAxiosError(err) && err.response) {
      const problem: Problem = err.response.data as Problem;
      this.alert.type = problem.status >= 500 ? 'alert' : 'warning';
      this.alert.title = problem.title;
      this.alert.description = problem.detail;
    } else {
      this.alert.type = 'alert';
      this.alert.title = 'An error occurred';
      this.alert.description = String(err);
    }
  }
}
</script>

<style scoped></style>
