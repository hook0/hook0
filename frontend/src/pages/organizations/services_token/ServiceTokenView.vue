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
import { push } from 'notivue';

const route = useRoute();

const biscuit_token = ref<null | string>(null);
const organization_id = ref<null | UUID>(null);

function _forceLoad() {
  organization_id.value = route.params.organization_id as UUID;
  biscuit_token.value = route.params.biscuit_token as string;
  let biscuit = getDeserializedBiscuit(biscuit_token.value);
  if (typeof biscuit === 'string') {
    biscuit_token.value = biscuit;
  } else {
    push.error({
      title: 'Error',
      message: biscuit.toString(),
      duration: 5000,
    });
  }
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
  <Hook0Card>
    <Hook0CardHeader>
      <Hook0Text>Service Token</Hook0Text>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLines>
        <Hook0CardContentLine>
          <Hook0Text>Organization ID</Hook0Text>
          <Hook0Text>{{ organization_id }}</Hook0Text>
        </Hook0CardContentLine>
        <Hook0CardContentLine>
          <Hook0Text>Biscuit Token</Hook0Text>
          <Hook0Text>{{ biscuit_token }}</Hook0Text>
        </Hook0CardContentLine>
      </Hook0CardContentLines>
    </Hook0CardContent>
  </Hook0Card>
</template>
