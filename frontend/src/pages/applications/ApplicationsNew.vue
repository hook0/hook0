<template>
  <form @submit="submit">
    <hook0-card>
      <hook0-card-header>
        <template v-slot:header>
          Create new application
        </template>
        <template v-slot:subtitle>
          An application emit events that are consumed by customers through webhooks
        </template>

      </hook0-card-header>
      <hook0-card-content>
        <hook0-card-content-line>
          <template v-slot:label>
            Application Name
          </template>
          <template v-slot:content>
            <hook0-input
              type="text"
              v-model.lazy.trim="application.name"
              placeholder="my awesome api - production"
            >
              <template v-slot:helpText></template>
            </hook0-input>
          </template>
        </hook0-card-content-line>
      </hook0-card-content>
      <hook0-card-footer>
        <hook0-button class="secondary" type="button" @click="$router.back()">Cancel</hook0-button>
        <hook0-button class="primary" type="submit">Create</hook0-button>
      </hook0-card-footer>
    </hook0-card>
  </form>

</template>

<script lang="ts">
import {Application, list} from './ApplicationService';
import {Options, Vue} from 'vue-class-component';
import Hook0Button from "@/components/Hook0Button.vue";
import {routes} from "@/routes";
import Hook0Input from "@/components/Hook0Input.vue";
import {create} from './ApplicationService';
import Hook0Card from "@/components/Hook0Card.vue";
import Hook0CardHeader from "@/components/Hook0CardHeader.vue";
import Hook0CardFooter from "@/components/Hook0CardFooter.vue";
import Hook0CardContent from "@/components/Hook0CardContent.vue";
import Hook0CardContentLine from "@/components/Hook0CardContentLine.vue";

@Options({
  components: {
    Hook0CardContentLine, Hook0CardContent, Hook0CardFooter, Hook0CardHeader, Hook0Card, Hook0Input, Hook0Button
  },
  props: {
    msg: String
  },
})
export default class ApplicationNew extends Vue {
  private applications$ !: Promise<Array<Application>>;

  data() {
    return {
      routes: routes,
      application: {
        name: ''
      },
      applications$: new Promise(() => {
      }),
    }
  }

  mounted() {
    this.applications$ = list(this.$route.query.organization_id as string);
  }

  submit() {
    create({
      name: this.application.name,
      organization_id: this.$route.query.organization_id
    }).then((resp) => {
      // move back to /applications
      console.log(resp);
      //this.$router.back();
    }, (error) => {
      console.log(error);
    })
  }
};
</script>

<style scoped>
</style>
