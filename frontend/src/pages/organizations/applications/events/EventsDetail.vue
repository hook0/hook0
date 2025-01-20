<script setup lang="ts">
import { onMounted, onUpdated, ref } from 'vue';
import { useRoute } from 'vue-router';

import Hook0Button from '@/components/Hook0Button.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import { UUID } from '@/http';
import Hook0Text from '@/components/Hook0Text.vue';
import * as EventsService from './EventsService';
import { EventWithPayload } from './EventsService';
import Hook0Code from '@/components/Hook0Code.vue';
import Hook0DateTime from '@/components/Hook0DateTime.vue';
import Hook0Loader from '@/components/Hook0Loader.vue';
import Hook0Error from '@/components/Hook0Error.vue';

const route = useRoute();

const event$ = ref<Promise<Array<EventWithPayload>>>();
const event_id = ref<null | UUID>(null);
const application_id = ref<null | UUID>(null);

function _load() {
  if (
    event_id.value !== route.params.event_id ||
    application_id.value !== route.params.application_id
  ) {
    event_id.value = route.params.event_id as UUID;
    application_id.value = route.params.application_id as UUID;

    event$.value = EventsService.get(event_id.value, application_id.value).then();
  }
}

onMounted(() => {
  _load();
});

onUpdated(() => {
  _load();
});
</script>

<template>
  <Promised :promise="event$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <Hook0Loader></Hook0Loader>
    </template>

    <template #rejected="error">
      <Hook0Error :error="error"></Hook0Error>
    </template>

    <template #default="event">
      <div>
        <Hook0Card>
          <Hook0CardHeader>
            <template #header>
              Event of type <Hook0Text class="code">{{ event.event_type_name }}</Hook0Text>
            </template>
            <template #subtitle>
              <Hook0Text class="block">
                <Hook0Text class="label pr-1">Event ID:</Hook0Text>
                <Hook0Text class="code">{{ event.event_id }}</Hook0Text>
              </Hook0Text>

              <Hook0Text class="block">
                <Hook0Text class="label pr-1">Occurred At:</Hook0Text>
                <Hook0DateTime :value="event.occurred_at"></Hook0DateTime>
              </Hook0Text>

              <Hook0Text class="block">
                <Hook0Text class="label pr-1">Received At:</Hook0Text>
                <Hook0DateTime :value="event.received_at"></Hook0DateTime>
              </Hook0Text>

              <Hook0Text class="block">
                <Hook0Text class="label pr-1">Source IP:</Hook0Text>
                <Hook0Text class="code">{{ event.ip }}</Hook0Text>
              </Hook0Text>
            </template>
          </Hook0CardHeader>
        </Hook0Card>

        <Hook0Card>
          <Hook0CardHeader>
            <template #header> Metadata </template>
            <template #subtitle>
              <Hook0Button href="https://documentation.hook0.com/docs/metadata" class="label pr-1"
                >Learn more…</Hook0Button
              >
            </template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <template v-if="event.metadata !== null && Object.keys(event.metadata).length > 0">
              <Hook0CardContentLine v-for="(value, key) in event.metadata" :key="key">
                <template #label>{{ key }}</template>
                <template #content>
                  <Hook0Text class="code">{{ value }}</Hook0Text>
                </template>
              </Hook0CardContentLine>
            </template>
            <template v-else>
              <Hook0CardContentLine>
                <template #label>No metadata</template>
              </Hook0CardContentLine>
            </template>
          </Hook0CardContent>
        </Hook0Card>
        <Hook0Card>
          <Hook0CardHeader>
            <template #header>Labels</template>
            <template #subtitle></template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <Hook0CardContentLine v-for="(value, key) in event.labels" :key="key">
              <template #label>{{ key }}</template>
              <template #content>
                <Hook0Text class="code">{{ value }}</Hook0Text>
              </template>
            </Hook0CardContentLine>
          </Hook0CardContent>
        </Hook0Card>

        <Hook0Card>
          <Hook0CardHeader>
            <template #header> Payload </template>
            <template #subtitle></template>
          </Hook0CardHeader>
          <Hook0CardContent>
            <Hook0CardContentLine>
              <template #label> Payload Content Type </template>
              <template #content>
                <Hook0Text class="code">{{ event.payload_content_type }}</Hook0Text>
              </template>
            </Hook0CardContentLine>
            <Hook0CardContentLine>
              <template #label> Payload (decoded) </template>
              <template #content>
                <Hook0Code :code="event.payload_decoded" />
              </template>
            </Hook0CardContentLine>
            <Hook0CardContentLine>
              <template #label> Payload (raw) </template>
              <template #content>
                <Hook0Code :code="event.payload" />
              </template>
            </Hook0CardContentLine>
          </Hook0CardContent>
        </Hook0Card>
      </div>
    </template>
  </Promised>
</template>
