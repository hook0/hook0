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
      <hook0-card>
        <hook0-card-header>
          <template #header>
            {{ event.event_type_name }}
          </template>
          <template #subtitle>
            <hook0-text class="block">
              <hook0-text class="label pr-1">Received at</hook0-text>
              <hook0-text>{{ event.received_at }}</hook0-text>
            </hook0-text>

            <hook0-text class="block">
              <hook0-text class="label pr-1">Occurred at</hook0-text>
              <hook0-text>{{ event.occurred_at }}</hook0-text>
            </hook0-text>
          </template>

        </hook0-card-header>
        <hook0-card-content>
          <hook0-card-content-line>
            <template #label>
              Payload
            </template>
            <template #content>
              {{ event.payload }}
            </template>
          </hook0-card-content-line>
          <hook0-card-content-line>
            <template #label>
              Payload Content type
            </template>
            <template #content>
              {{ event.payload_content_type_name }}
            </template>
          </hook0-card-content-line>
          <hook0-card-content-line>
            <template #label>
              Ip
            </template>
            <template #content>
              {{ event.ip }}
            </template>
          </hook0-card-content-line>
        </hook0-card-content>
        <hook0-card-footer>
        </hook0-card-footer>
      </hook0-card>
    </template>

  </Promised>
</template>

<script lang="ts">
import {AxiosError} from 'axios';
import * as EventsService from './EventsService';
import {EventWithPayload} from './EventsService';
import {Options, Vue} from 'vue-class-component';
import {routes} from "@/routes";
import {isAxiosError, Problem, UUID} from "@/http";
import {Alert} from "@/components/Hook0Alert";
import Hook0Text from "@/components/Hook0Text.vue";

@Options({
  components: {Hook0Text},
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
    if (this.event_id !== this.$route.params.event_id ||
      this.application_id !== this.$route.params.application_id) {

      this.event_id = this.$route.params.event_id as UUID;
      this.application_id = this.$route.params.application_id as UUID;

      this.event$ = EventsService.get(this.event_id, this.application_id).then();
    }
  }
};
</script>

<style scoped>
</style>