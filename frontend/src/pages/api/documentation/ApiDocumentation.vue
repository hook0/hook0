<script setup lang="ts">
import SwaggerUI from 'swagger-ui';
import 'swagger-ui/dist/swagger-ui.css';
import { useRoute } from 'vue-router';
import { onMounted, onUpdated, ref } from 'vue';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import featureFlags from '@/feature-flags';
import { getAccessToken } from '@/iam';

const route = useRoute();
const swaggerUI = ref<null | SwaggerUI>(null);
const container = ref<null | HTMLDivElement>(null);

function _load() {
  const application_id = route.params.application_id;
  const organization_id = route.params.organization_id;

  swaggerUI.value = SwaggerUI({
    url:
      featureFlags.getOrElse('API_ENDPOINT', import.meta.env.VITE_API_ENDPOINT ?? '') +
      '/swagger.json',
    domNode: container.value,

    docExpansion: 'list',

    showExtensions: true,
    showCommonExtensions: true,

    // if set to true, it triggers infinite redirect loop with keycloak
    deepLinking: false,

    tagsSorter: (tag1, tag2) => {
      const order = [
        'Hook0',
        'Organizations Management',
        'Applications Management',
        'Events Management',
        'Subscriptions Management',
      ];
      return order.indexOf(tag1) - order.indexOf(tag2);
    },

    operationsSorter: 'alpha',

    displayOperationId: true,

    parameterMacro: (_operation: Readonly<unknown>, parameter: Readonly<unknown>) => {
      if (
        organization_id &&
        'name' in parameter &&
        parameter.name === 'organization_id' &&
        'schema' in parameter &&
        typeof parameter.schema === 'object' &&
        parameter.schema !== null &&
        'default' in parameter.schema
      ) {
        parameter.schema.default = organization_id;
      }

      if (
        application_id &&
        'name' in parameter &&
        parameter.name === 'application_id' &&
        'schema' in parameter &&
        typeof parameter.schema === 'object' &&
        parameter.schema !== null &&
        'default' in parameter.schema
      ) {
        parameter.schema.default = application_id;
      }
    },

    requestInterceptor: (req: SwaggerUI.Request) => {
      const accessToken = getAccessToken().value;
      return Object.assign(
        {},
        req,
        accessToken ? { Headers: { Authorization: `Bearer ${accessToken}` } } : {}
      );
    },

    // try out
    displayRequestDuration: true,
    tryItOutEnabled: true,
    // \try out

    syntaxHighlight: {
      activate: true,
    },

    onComplete: () => {
      console.log('Swagger UI launched');
    },
  });
}

onMounted(() => {
  _load();
});

onUpdated(() => {
  _load();
});
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header> Hook0 API Documentation </template>
      <template #subtitle> Automate everything! </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <div ref="container">
        <!-- swagger ui documentation -->
      </div>
    </Hook0CardContent>
    <Hook0CardFooter> </Hook0CardFooter>
  </Hook0Card>
</template>
