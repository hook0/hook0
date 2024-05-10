<script setup lang="ts">
import { onMounted, onUpdated, ref } from 'vue';
import { useRoute } from 'vue-router';

import Hook0CardContentLine from '@/components/Hook0CardContentLine.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0Card from '@/components/Hook0Card.vue';
import { UUID } from '@/http';
import Hook0Text from '@/components/Hook0Text.vue';
import Hook0CardContentLines from '@/components/Hook0CardContentLines.vue';
import { getDeserializedBiscuit } from '@/utils/biscuit_auth.ts';

const route = useRoute();

const biscuit_token = ref<null | string>(null);
const organization_id = ref<null | UUID>(null);

function _forceLoad() {
  organization_id.value = route.params.organization_id as UUID;
  let biscuit = getDeserializedBiscuit(route.params.biscuit_token as string);
  biscuit_token.value = biscuit;
  console.log(biscuit_token.value);
}

function _load() {
  if (organization_id.value !== route.params.organization_id) {
    _forceLoad();
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
  <div>
    <Hook0Card>
      <Hook0CardHeader>
        <template #header> Service Token </template>
        <template #subtitle>
          <div class="text-sm text-gray-500">
            This is your service token. You can use it to authenticate your services with Hook0.
          </div>
        </template>
      </Hook0CardHeader>
      <Hook0CardContent>
        <Hook0CardContentLines>
          <Hook0CardContentLine>
            <template #label> Service Token </template>
            <template #content>
              <Hook0Text>{{ biscuit_token }}</Hook0Text>
            </template>
          </Hook0CardContentLine>
        </Hook0CardContentLines>
      </Hook0CardContent>
    </Hook0Card>
  </div>
</template>
