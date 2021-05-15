<template>
  <hook0-card>
    <hook0-card-header>
      <template #header>
        Hook0 API Documentation
      </template>
      <template #subtitle>
        Automate everything!
      </template>

    </hook0-card-header>
    <hook0-card-content>
      <div ref="container">
        <!-- swagger ui documentation -->
      </div>
    </hook0-card-content>
    <hook0-card-footer>
    </hook0-card-footer>
  </hook0-card>
</template>

<script lang="ts">
import SwaggerUI from 'swagger-ui';
import "swagger-ui/dist/swagger-ui.css";

import {Options, Vue} from 'vue-class-component';
import Hook0Card from "@/components/Hook0Card.vue";
import Hook0CardHeader from "@/components/Hook0CardHeader.vue";
import Hook0CardFooter from "@/components/Hook0CardFooter.vue";
import Hook0CardContent from "@/components/Hook0CardContent.vue";
import Hook0CardContentLine from "@/components/Hook0CardContentLine.vue";

@Options({
  components: {
    Hook0CardContentLine,
    Hook0CardContent,
    Hook0CardFooter,
    Hook0CardHeader,
    Hook0Card
  },
})
export default class ApiDocumentation extends Vue {
  private swaggerUI!: any;

  // annotate refs type.
  // The symbol `!` (definite assignment assertion) is needed to get rid of compilation error.
  // noinspection JSUnusedGlobalSymbols
  $refs!: {
    container: HTMLDivElement
  }

  data() {
    return {}
  }

  mounted() {
    this.swaggerUI = SwaggerUI({
      url: "/api/v1/swagger.json",
      domNode: this.$refs.container,

      presets: [
        SwaggerUI.presets.apis,
      ],
      plugins: [
        SwaggerUI.plugins.DownloadUrl
      ],

      docExpansion: "list",

      showExtensions: true,
      showCommonExtensions: true,

      // if set to true, it triggers infinite redirect loop with keycloak
      deepLinking: false,

      tagsSorter: (tag) => {
        const order = ['Applications Management', 'Events Management', 'Subscriptions Management', 'Identity and Access Management'];
        return order.indexOf(tag);
      },

      apisSorter: "alpha",
      operationsSorter: "alpha",

      displayOperationId: false,

      // try out
      displayRequestDuration: true,
      tryItOutEnabled: true,
      // \try out

      syntaxHighlight: {
        activate: true,
        theme: "aaa"
      },

      onComplete: () => {
        console.log('Swagger UI launched');
      }
    })
  }
};

</script>

<style scoped>
</style>
