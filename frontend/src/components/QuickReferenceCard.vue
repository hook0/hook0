<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { BookOpen } from 'lucide-vue-next';

import Hook0Card from './Hook0Card.vue';
import Hook0CardHeader from './Hook0CardHeader.vue';
import Hook0CardContent from './Hook0CardContent.vue';
import Hook0CardContentLine from './Hook0CardContentLine.vue';
import Hook0CopyField from './Hook0CopyField.vue';
import Hook0Stack from './Hook0Stack.vue';

type Props = {
  title: string;
  subtitle: string;
  organizationId: string;
  applicationId?: string;
};

withDefaults(defineProps<Props>(), {
  applicationId: undefined,
});

const { t } = useI18n();
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>
        <Hook0Stack direction="row" align="center" gap="sm">
          <BookOpen :size="18" aria-hidden="true" />
          {{ title }}
        </Hook0Stack>
      </template>
      <template #subtitle>
        {{ subtitle }}
      </template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <Hook0CardContentLine type="split">
        <template #label>{{ t('common.organizationId') }}</template>
        <template #content>
          <Hook0CopyField :value="organizationId" />
        </template>
      </Hook0CardContentLine>
      <Hook0CardContentLine v-if="applicationId" type="split">
        <template #label>{{ t('common.applicationId') }}</template>
        <template #content>
          <Hook0CopyField :value="applicationId" />
        </template>
      </Hook0CardContentLine>
    </Hook0CardContent>
  </Hook0Card>
</template>
