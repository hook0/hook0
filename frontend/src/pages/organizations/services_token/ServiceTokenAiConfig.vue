<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { useClipboardCopy } from '@/composables/useClipboardCopy';

import Hook0Card from '@/components/Hook0Card.vue';
import Hook0CardHeader from '@/components/Hook0CardHeader.vue';
import Hook0CardContent from '@/components/Hook0CardContent.vue';
import Hook0Button from '@/components/Hook0Button.vue';
import Hook0Tabs from '@/components/Hook0Tabs.vue';
import Hook0Stack from '@/components/Hook0Stack.vue';
import Hook0Code from '@/components/Hook0Code.vue';

type Props = {
  token: string;
};

const props = defineProps<Props>();

const { t } = useI18n();
const clipboardCopy = useClipboardCopy();

type AiTabId = 'claude' | 'cursor' | 'windsurf' | 'cline';

type McpConfigDef = {
  file: string;
  wrapInMcpServers: boolean;
};

const MCP_CONFIGS: Record<AiTabId, McpConfigDef> = {
  claude: { file: 'claude_desktop_config.json', wrapInMcpServers: true },
  cursor: { file: '.cursor/mcp.json', wrapInMcpServers: false },
  windsurf: { file: '~/.codeium/windsurf/mcp_config.json', wrapInMcpServers: true },
  cline: { file: 'cline_mcp_settings.json', wrapInMcpServers: false },
};

const aiTabs: Array<{ id: AiTabId; label: string }> = [
  { id: 'claude', label: 'Claude' },
  { id: 'cursor', label: 'Cursor' },
  { id: 'windsurf', label: 'Windsurf' },
  { id: 'cline', label: 'Cline' },
];

const activeTab = defineModel<string>({ default: 'claude' });

function buildMcpConfig(token: string, def: McpConfigDef): string {
  const inner = { command: 'hook0-mcp', env: { HOOK0_API_TOKEN: token } };
  const obj = def.wrapInMcpServers ? { mcpServers: { hook0: inner } } : { hook0: inner };
  return JSON.stringify(obj, null, 2);
}

const configs = computed(() => {
  const token = props.token || t('serviceTokens.tokenPlaceholder');
  return Object.fromEntries(
    Object.entries(MCP_CONFIGS).map(([id, def]) => [id, buildMcpConfig(token, def)])
  ) as Record<AiTabId, string>;
});

function copyConfig() {
  const tabId = activeTab.value as AiTabId;
  clipboardCopy(configs.value[tabId]);
}
</script>

<template>
  <Hook0Card>
    <Hook0CardHeader>
      <template #header>{{ t('serviceTokens.aiIntegrationTitle') }}</template>
    </Hook0CardHeader>
    <Hook0CardContent>
      <div class="ai-config-section">
        <Hook0Tabs v-model="activeTab" :tabs="aiTabs">
          <template v-for="tab in aiTabs" :key="tab.id" #[tab.id]>
            <Hook0Stack direction="column" gap="md">
              <span class="ai-config-section__hint">{{
                t('serviceTokens.addToConfig', { file: MCP_CONFIGS[tab.id].file })
              }}</span>
              <Hook0Code :code="configs[tab.id]" />
            </Hook0Stack>
          </template>
        </Hook0Tabs>
        <div class="ai-config-section__footer">
          <Hook0Button variant="primary" size="sm" type="button" @click="copyConfig">
            {{ t('serviceTokens.copyConfig') }}
          </Hook0Button>
        </div>
      </div>
    </Hook0CardContent>
  </Hook0Card>
</template>

<style scoped>
.ai-config-section {
  padding: 0 1.25rem 1.25rem;
}

.ai-config-section__hint {
  font-size: 0.875rem;
  color: var(--color-text-secondary);
}

.ai-config-section__footer {
  margin-top: 1rem;
}

</style>
