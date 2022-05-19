<template>
  <hook0-card>
    <hook0-card-header>
      <template #header>
        Delete this subscription
      </template>
      <template #subtitle>
        This action deletes
        <hook0-text class="bold">{{ subscriptionName }}</hook0-text>
        and everything this subscription contains. There is no going back.
      </template>
    </hook0-card-header>
    <hook0-card-content v-if="alert.visible">
      <hook0-alert :type="alert.type" :title="alert.title" :description="alert.description"></hook0-alert>
    </hook0-card-content>
    <hook0-card-footer>
      <hook0-button class="danger" type="button" :loading="loading" @click="remove($event)">Delete</hook0-button>
    </hook0-card-footer>
  </hook0-card>
</template>

<script lang="ts">
import {AxiosError} from 'axios';
import * as SubscriptionsService from './SubscriptionService';
import {Options, Vue} from 'vue-class-component';
import {routes} from "@/routes";
import {isAxiosError, Problem, UUID} from "@/http";
import {Alert} from '@/components/Hook0Alert';


@Options({
  props: {
    subscriptionId: {
      type: String,
      required: true
    },
    applicationId: {
      type: String,
      required: true
    },
    subscriptionName: {
      type: String,
      required: true
    }
  }
})
export default class SubscriptionsRemove extends Vue {
  private loading = false;
  private subscriptionId!: string;
  private applicationId!: string;
  private subscriptionName!: string;

  routes = routes;

  alert: Alert = {
    visible: false,
    type: 'alert',
    title: '',
    description: '',
  };

  remove(e: Event) {
    e.preventDefault();
    e.stopImmediatePropagation();

    if (!confirm(`Are you sure to delete "${this.subscriptionName}" subscription?`)) {
      return;
    }

    this.alert.visible = false; // reset alert
    this.loading = true;

    SubscriptionsService.remove(this.applicationId, this.subscriptionId).then(() =>
        this.$router.push({
          name: routes.OrganizationsDashboard,
          params: {
            organization_id: this.$route.params.organization_id
          }
        })
      , this.displayError.bind(this))
      // finally
      .finally(() => this.loading = false);
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
      this.alert.title = "An error occurred";
      this.alert.description = String(err);
    }
  }
};


</script>

<style scoped>
</style>
