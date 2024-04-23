<script setup lang="ts">
import { ref } from 'vue';

import { Problem, UUID } from '@/http';
import { Alert } from '@/components/Hook0Alert';
import { routes } from '@/routes';
import Hook0ListItem from '@/components/Hook0ListItem.vue';
import Hook0List from '@/components/Hook0List.vue';
import { EventTypePost } from './EventTypeService';
import * as EventTypeService from './EventTypeService';
import { useRoute, useRouter } from 'vue-router';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardFooter from '@/components/Hook0CardFooter.vue';
import Hook0Input from '@/components/Hook0Input.vue';
import Hook0Alert from '@/components/Hook0Alert.vue';
import Hook0Button from '@/components/Hook0Button.vue';

const router = useRouter();
const route = useRoute();

const event_type = ref<EventTypePost>({
  application_id: '',
  service: '',
  resource_type: '',
  verb: '',
});
const alert = ref<Alert>({
  visible: false,
  type: 'alert',
  title: '',
  description: '',
});

function create(e: Event) {
  e.preventDefault();
  e.stopImmediatePropagation();

  alert.value.visible = false; // reset alert

  event_type.value.application_id = route.params.application_id as UUID;

  EventTypeService.create(event_type.value).then(async (_resp) => {
    await router.push({
      name: routes.EventTypesList,
    });
  }, displayError);
}

function displayError(err: Problem) {
  console.error(err);
  alert.value.visible = true;

  alert.value.type = err.status >= 500 ? 'alert' : 'warning';
  alert.value.title = err.title;
  alert.value.description = err.detail;
}
</script>

<template>
  <form ref="form" @submit="create">
    <Hook0Card>
      <Hook0CardHeader>
        <template #header> Create new event type </template>
        <template #subtitle> Each event sent through a webhook must have an event type. </template>
      </Hook0CardHeader>

      <Hook0CardContentLine>
        <template #label>
          <Hook0Text>Event Type</Hook0Text>
        </template>
        <template #content>
          <div class="flex flex-row justify-start items-baseline">
            <Hook0Input
              v-model="event_type.service"
              type="text"
              placeholder="billing"
              required
              class="flex-grow-1"
            >
            </Hook0Input>
            <Hook0Text class="bold flex-grow-0">.</Hook0Text>
            <Hook0Input
              v-model="event_type.resource_type"
              type="text"
              placeholder="invoice"
              required
              class="flex-grow-1"
            >
            </Hook0Input>
            <Hook0Text class="bold flex-grow-0">.</Hook0Text>
            <Hook0Input
              v-model="event_type.verb"
              type="text"
              placeholder="created"
              required
              class="flex-grow-1"
            >
            </Hook0Input>
          </div>
        </template>
      </Hook0CardContentLine>
      <Hook0CardContent v-if="alert.visible">
        <Hook0Alert
          :type="alert.type"
          :title="alert.title"
          :description="alert.description"
        ></Hook0Alert>
      </Hook0CardContent>
      <Hook0CardFooter>
        <Hook0Button class="secondary" type="button" @click="$router.back()">Cancel</Hook0Button>
        <Hook0Button class="primary" type="button" @click="create($event)"
          >Create event type
        </Hook0Button>
      </Hook0CardFooter>
    </Hook0Card>

    <Hook0Card>
      <Hook0CardContent>
        <Hook0CardContentLine type="full-width">
          <template #content>
            <Hook0Text class="block"
              >An event is something that has happened. In the past.</Hook0Text
            >
            <Hook0Text class="mt-2">Event should be in the form of:</Hook0Text>
            <Hook0Text class="code"> &lt;service&gt;.&lt;resourceType&gt;.&lt;verb&gt; </Hook0Text>
          </template>
        </Hook0CardContentLine>
        <Hook0CardContentLine type="columns">
          <template #content>
            <div>
              <!--            https://cloud.google.com/iam/docs/permissions-reference -->
              <Hook0Text class="mt-4 block bold">
                <Hook0Text class="code">&lt;service&gt;</Hook0Text>
                examples
              </Hook0Text>

              <Hook0List>
                <Hook0ListItem>
                  <template #left>billing</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>chat</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>contacts</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>connectors</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>file</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>iam</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>iap</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>integrations</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>logging</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>monitoring</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>storage</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>workflows</template>
                </Hook0ListItem>
              </Hook0List>
            </div>

            <div>
              <Hook0Text class="mt-4 block bold">
                <Hook0Text class="code">&lt;resourceType&gt;</Hook0Text>
                examples
              </Hook0Text>
              <Hook0List>
                <Hook0ListItem>
                  <template #left>project</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>action</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>comment</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>collaborator</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>teammember</template>
                </Hook0ListItem>
              </Hook0List>
            </div>

            <div>
              <Hook0Text class="mt-4 block bold">
                <Hook0Text class="code">&lt;verb&gt;</Hook0Text>
                examples
              </Hook0Text>
              <Hook0List>
                <Hook0ListItem>
                  <template #left>created</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>updated</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>deleted</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>copied</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>versioned</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>executed</template>
                </Hook0ListItem>
                <Hook0ListItem>
                  <template #left>completed</template>
                </Hook0ListItem>
              </Hook0List>
            </div>
          </template>
        </Hook0CardContentLine>
      </Hook0CardContent>
      <Hook0CardFooter> </Hook0CardFooter>
    </Hook0Card>
  </form>
</template>
