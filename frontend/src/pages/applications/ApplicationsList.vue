<template>
  <Promised :promise="applications$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <p>Loading...</p>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="applications">
      <hook0-card>
        <hook0-card-header>
          <template v-slot:header>
            Applications
          </template>
          <template v-slot:subtitle>
            Each application send events to Hook0 API and Hook0 dispatch these extends to customers through webhooks
          </template>
        </hook0-card-header>

        <hook0-card-content v-if="applications.length > 0">
          <hook0-card-content-line v-for="application in applications" :key="application.application_id">
            <template v-slot:label>
              {{ application.name }}
            </template>
            <template v-slot:content>
              id: {{ application.application_id }}
            </template>
          </hook0-card-content-line>
        </hook0-card-content>

        <hook0-card-footer>
          <hook0-button class="primary" type="button" @click="$router.push({name:routes.ApplicationsNew})">Create new application</hook0-button>
        </hook0-card-footer>
      </hook0-card>
    </template>
    <!-- The "rejected" scoped slot will be used if there is an error -->
    <template #rejected="error">
      <p>Error: {{ error.message }}</p>
    </template>
  </Promised>
</template>

<script lang="ts">
import {Application, list} from './ApplicationService';
import {Options, Vue} from 'vue-class-component';
import Hook0Button from "@/components/Hook0Button.vue";
import {routes} from "@/routes";
import Hook0CardContentLine from "@/components/Hook0CardContentLine.vue";
import Hook0CardContent from "@/components/Hook0CardContent.vue";
import Hook0CardFooter from "@/components/Hook0CardFooter.vue";
import Hook0CardHeader from "@/components/Hook0CardHeader.vue";
import Hook0Card from "@/components/Hook0Card.vue";
import Hook0Input from "@/components/Hook0Input.vue";

@Options({
  components: {Hook0Button, Hook0CardContentLine, Hook0CardContent, Hook0CardFooter, Hook0CardHeader, Hook0Card, Hook0Input, Hook0Button},
  props: {
    msg: String
  },
})
export default class ApplicationList extends Vue {
  private applications$ !: Promise<Array<Application>>;

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
};
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
</style>
