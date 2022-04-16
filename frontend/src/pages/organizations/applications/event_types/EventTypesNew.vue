<template>
  <form @submit="create" ref="form">
    <hook0-card>
      <hook0-card-header>
        <template #header>
          Create new event type
        </template>
        <template #subtitle>
          Each event sent through a webhook must have an event type.
        </template>

      </hook0-card-header>
      <hook0-card-content>
        <hook0-card-content-line>
          <template #label>
            <hook0-text>Event Type</hook0-text>
          </template>
          <template #content>
            <div class="flex flex-row justify-start items-baseline">
              <hook0-input
                type="text"
                v-model="event_type.service"
                placeholder="billing"
                required
                class="flex-grow-1"
              >
              </hook0-input>
              <hook0-text class="bold flex-grow-0">.</hook0-text>
              <hook0-input
                type="text"
                v-model="event_type.resource_type"
                placeholder="invoice"
                required
                class="flex-grow-1"
              >
              </hook0-input>
              <hook0-text class="bold flex-grow-0">.</hook0-text>
              <hook0-input
                type="text"
                v-model="event_type.verb"
                placeholder="created"
                required
                class="flex-grow-1"
              >
              </hook0-input>
            </div>
          </template>
        </hook0-card-content-line>
        <hook0-card-content-line>
          <template #content>
            <hook0-text class="block">An event is something that has happened. In the past.</hook0-text>
            <hook0-text class="mt-2">Event should be in the form of:</hook0-text>
            <hook0-text class="bold">&lt;service&gt;.&lt;resourceType&gt;.&lt;verb_past_tense&gt;
            </hook0-text>

            <!--            https://cloud.google.com/iam/docs/permissions-reference -->
            <hook0-text class="mt-4 block bold">Service type examples</hook0-text>

            <hook0-list>
              <hook0-list-item>
                <template #left>billing</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>chat</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>contacts</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>connectors</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>file</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>iam</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>iap</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>integrations</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>logging</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>monitoring</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>storage</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>workflows</template>
              </hook0-list-item>
            </hook0-list>

            <hook0-text class="mt-4 block bold">Resource type examples</hook0-text>
            <hook0-list>
              <hook0-list-item>
                <template #left>project</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>action</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>comment</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>collaborator</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>teammember</template>
              </hook0-list-item>
            </hook0-list>

            <hook0-text class="mt-4 block bold">Verb examples</hook0-text>
            <hook0-list>
              <hook0-list-item>
                <template #left>created</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>updated</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>deleted</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>copied</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>versioned</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>executed</template>
              </hook0-list-item>
              <hook0-list-item>
                <template #left>completed</template>
              </hook0-list-item>
            </hook0-list>


          </template>
        </hook0-card-content-line>

      </hook0-card-content>

      <hook0-card-content v-if="alert.visible">
        <hook0-alert :type="alert.type" :title="alert.title" :description="alert.description"></hook0-alert>
      </hook0-card-content>
      <hook0-card-footer>
        <hook0-button class="secondary" type="button" @click="$router.back()">Cancel</hook0-button>
        <hook0-button class="primary" type="button" @click="create($event)">Create event type
        </hook0-button>
      </hook0-card-footer>
    </hook0-card>
  </form>

</template>

<script lang="ts">
import {AxiosError} from 'axios';
import {Options, Vue} from 'vue-class-component';
import {routes} from "@/routes";
import {isAxiosError, Problem, UUID} from "@/http";
import {Alert} from "@/components/Hook0Alert";
import {definitions} from "@/types";
import * as EventTypeService from './EventTypeService';
import Hook0ListItem from "@/components/Hook0ListItem.vue";
import Hook0List from "@/components/Hook0List.vue";

export type EventTypePost = definitions['EventTypePost'];

@Options({
  components: {
    Hook0List,
    Hook0ListItem,
  }
})
export default class EventTypesNew extends Vue {
  routes = routes;

  event_type: EventTypePost = {application_id: "", service: "", resource_type: "", verb: ""};

  alert: Alert = {
    visible: false,
    type: 'alert',
    title: '',
    description: '',
  };

  create(e: Event) {
    e.preventDefault();
    e.stopImmediatePropagation();

    this.alert.visible = false; // reset alert
    
    this.event_type.application_id = this.$route.params.application_id as UUID;

    EventTypeService.create(this.event_type).then(async (_resp: any) => {
      await this.$router.push({
        name: routes.EventTypesList,
      });
    }, this.displayError.bind(this))
  }

  displayError(err: AxiosError | unknown) {
    console.error(err);
    this.alert.visible = true;

    if (isAxiosError(err) && err.response) {
      const problem: Problem = err.response.data as Problem;
      this.alert.type = problem.status >= 500 ? 'alert' : 'warning';
      this.alert.title = problem.title;
      this.alert.description = problem.detail;
    } else {
      this.alert.type = 'alert';
      this.alert.title = "An error occurred";
      this.alert.description = String(err);
    }
  }
};

</script>

<style scoped>
</style>
