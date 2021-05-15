<template>
  <form @submit="submit">
    <hook0-card>
      <hook0-card-header>
        <template #header>
          Create new application
        </template>
        <template #subtitle>
          An application emit events that are consumed by customers through webhooks
        </template>

      </hook0-card-header>
      <hook0-card-content>
        <hook0-card-content-line>
          <template #label>
            Application Name
          </template>
          <template #content>
            <hook0-input
              type="text"
              v-model="application.name"
              placeholder="my awesome api - production"
              required
            >
              <template #helpText></template>
            </hook0-input>
          </template>
        </hook0-card-content-line>
      </hook0-card-content>

      <hook0-card-content v-if="alert.visible">
        <hook0-alert :type="alert.type" :title="alert.title" :description="alert.description"></hook0-alert>
      </hook0-card-content>
      <hook0-card-footer>
        <hook0-button class="secondary" type="button" @click="$router.back()">Cancel</hook0-button>
        <hook0-button class="primary" type="submit">Create</hook0-button>
      </hook0-card-footer>
    </hook0-card>
  </form>

</template>

<script lang="ts">
import {Application, create, list} from './ApplicationService';
import {Options, Vue} from 'vue-class-component';
import Hook0Button from "@/components/Hook0Button.vue";
import {routes} from "@/routes";
import Hook0Input from "@/components/Hook0Input.vue";
import Hook0Card from "@/components/Hook0Card.vue";
import Hook0CardHeader from "@/components/Hook0CardHeader.vue";
import Hook0CardFooter from "@/components/Hook0CardFooter.vue";
import Hook0CardContent from "@/components/Hook0CardContent.vue";
import Hook0CardContentLine from "@/components/Hook0CardContentLine.vue";
import Hook0Alert, {AlertStatus} from "@/components/Hook0Alert.vue";

import {definitions} from '@/types';

export type Problem = definitions['Problem'];

@Options({
  components: {
    Hook0Alert,
    Hook0CardContentLine, Hook0CardContent, Hook0CardFooter, Hook0CardHeader, Hook0Card, Hook0Input, Hook0Button
  },
})
export default class ApplicationNew extends Vue {
  private applications$ !: Promise<Array<Application>>;
  private application = {
    name: '',
  };

  private alert = {
    visible: false,
    type: AlertStatus.ALERT,
    title: '',
    description: '',
  };

  data() {
    return {
      routes: routes,
      applications$: new Promise(() => {
      }),
    }
  }

  mounted() {
    this.applications$ = list(this.$route.query.organization_id as string);
  }

  submit(e: Event) {
    e.preventDefault();

    this.alert.visible = false; // reset alert

    create({
      name: this.application.name,
      organization_id: (this.$route.query.organization_id as string)
    }).then((resp) => {
      this.$router.push({
        name: routes.ApplicationsList,
      });
    }, (err) => {
      console.error(err);
      this.alert.visible = true;

      if (err.isAxiosError && err.response && err.response.data) {
        const problem: Problem = err.response.data;
        this.alert.type = problem.status >= 500 ? AlertStatus.ALERT : AlertStatus.WARNING;
        this.alert.title = problem.title;
        this.alert.description = problem.detail;
      } else {
        this.alert.type = AlertStatus.ALERT;
        this.alert.title = "An error occurred";
        this.alert.description = String(err);
      }
    })
  }
};
</script>

<style scoped>
</style>
