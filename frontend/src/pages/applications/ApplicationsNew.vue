<template>
  <div>
    <div class="max-w-7xl mx-auto px-4 sm:px-6 md:px-8">
      <h1 class="text-2xl font-semibold text-gray-900">Create a new application</h1>
    </div>
    <div class="max-w-7xl mx-auto px-4 sm:px-6 md:px-8 h-96">
      <div class="py-4">
        PLOPPPPPPPPPPPP
      </div>
    </div>
  </div>
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
export default class ApplicationNew extends Vue {
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

<style scoped>
</style>
