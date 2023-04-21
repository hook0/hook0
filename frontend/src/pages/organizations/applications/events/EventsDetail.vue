<template>
  <Promised :promise="event$">
    <!-- Use the "pending" slot to display a loading message -->
    <template #pending>
      <hook0-loader></hook0-loader>
    </template>

    <template #rejected="error">
      <hook0-error :error="error"></hook0-error>
    </template>

    <template #default="event">
      <div>
        <hook0-card>
          <hook0-card-header>
            <template #header>
              {{ event.event_type_name }}
            </template>
            <template #subtitle>
              <hook0-text class="block">
                <hook0-text class="label pr-1">Received at</hook0-text>
                <hook0-date-time :value="event.received_at"></hook0-date-time>
              </hook0-text>

              <hook0-text class="block">
                <hook0-text class="label pr-1">Occurred at</hook0-text>
                <hook0-date-time :value="event.occurred_at"></hook0-date-time>
              </hook0-text>
            </template>
          </hook0-card-header>
          <hook0-card-content>
            <hook0-card-content-line>
              <template #label> Source Ip </template>
              <template #content>
                <hook0-text class="code">{{ event.ip }}</hook0-text>
              </template>
            </hook0-card-content-line>
          </hook0-card-content>
        </hook0-card>

        <hook0-card>
          <hook0-card-header>
            <template #header> Metadata </template>
            <template #subtitle>
              <hook0-button href="https://documentation.hook0.com/docs/metadata" class="label pr-1"
                >Learn more…</hook0-button
              >
            </template>
          </hook0-card-header>
          <hook0-card-content>
            <hook0-card-content-line :key="key" v-for="(key, value) in event.metadata">
              <template #label>{{ key }}</template>
              <template #content>
                <hook0-text class="code">{{ value }}</hook0-text>
              </template>
            </hook0-card-content-line>
          </hook0-card-content>
        </hook0-card>
        <hook0-card>
          <hook0-card-header>
            <template #header>Labels</template>
            <template #subtitle> </template>
          </hook0-card-header>
          <hook0-card-content>
            <hook0-card-content-line :key="key" v-for="(key, value) in event.labels">
              <template #label>{{ key }}</template>
              <template #content>
                <hook0-text class="code">{{ value }}</hook0-text>
              </template>
            </hook0-card-content-line>
          </hook0-card-content>
        </hook0-card>

        <hook0-card>
          <hook0-card-header>
            <template #header> Payload </template>
            <template #subtitle> </template>
          </hook0-card-header>
          <hook0-card-content>
            <hook0-card-content-line>
              <template #label> Payload Content type </template>
              <template #content>
                <hook0-text class="code">{{ event.payload_content_type }}</hook0-text>
              </template>
            </hook0-card-content-line>
            <hook0-card-content-line>
              <template #label> Payload (decoded) </template>
              <template #content>
                <hook0-code :code="event.payload_decoded" />
              </template>
            </hook0-card-content-line>
            <hook0-card-content-line>
              <template #label> Payload (raw) </template>
              <template #content>
                <hook0-code :code="event.payload" />
              </template>
            </hook0-card-content-line>
          </hook0-card-content>
        </hook0-card>
      </div>
    </template>
  </Promised>
</template>

<script lang="ts">
import { AxiosError } from 'axios';
import * as EventsService from './EventsService';
import { EventWithPayload } from './EventsService';
import { Options, Vue } from 'vue-class-component';
import { routes } from '@/routes';
import { isAxiosError, Problem, UUID } from '@/http';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Code from '@/components/Hook0Code.vue';
import Hook0DateTime from '@/components/Hook0DateTime.vue';

@Options({
  components: { Hook0Text, Hook0Code, Hook0DateTime },
})
export default class EventsDetail extends Vue {
  event_id: UUID | null = null;
  application_id: UUID | null = null;

  routes = routes;

  event$: Promise<EventWithPayload> = new Promise(() => {
    //
  });

  mounted() {
    this._load();
  }

  updated() {
    this._load();
  }

  _load() {
    if (
      this.event_id !== this.$route.params.event_id ||
      this.application_id !== this.$route.params.application_id
    ) {
      this.event_id = this.$route.params.event_id as UUID;
      this.application_id = this.$route.params.application_id as UUID;

      this.event$ = EventsService.get(this.event_id, this.application_id).then();
    }
  }
}
</script>

<style scoped></style>
