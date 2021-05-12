<template>
  <Promised :promise="applications$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <p>Loading...</p>
    </template>
    <!-- The default scoped slot will be used as the result -->
    <template #default="applications">
      <div>
        <div class="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
          <h1 class="text-2xl font-semibold text-gray-900">Applications</h1>
        </div>
        <div class="max-w-7xl mx-auto px-4 sm:px-6 md:px-8 h-96">
          <div class="py-4">
            <template v-if="applications.length > 0">
              <ul>
                <li v-for="application in applications" :key="application.application_id">
                  {{ application.application_id }} - {{ application.name }}
                </li>
              </ul>
            </template>
            <template v-if="applications.length === 0">
              <hook0-button class="primary" @click="$router.push({name:routes.ApplicationsNew})">PLop</hook0-button>
            </template>
          </div>
        </div>
      </div>
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

@Options({
  components: {Hook0Button},
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
