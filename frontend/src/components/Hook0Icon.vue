<script setup lang="ts">
import { computed, type Component } from 'vue';
import * as icons from 'lucide-vue-next';

interface Props {
  name: string;
  size?: number;
  strokeWidth?: number;
}

const props = withDefaults(defineProps<Props>(), {
  size: 20,
  strokeWidth: 2,
});

// Map legacy FontAwesome names to Lucide equivalents
const iconNameMap: Record<string, string> = {
  spinner: 'Loader2',
  'circle-notch': 'Loader2',
  link: 'Link',
  copy: 'Copy',
  plus: 'Plus',
  minus: 'Minus',
  'fa-minus': 'Minus',
  'fa-plus': 'Plus',
  trash: 'Trash2',
  edit: 'Pencil',
  check: 'Check',
  'check-circle': 'CheckCircle',
  times: 'X',
  'times-circle': 'XCircle',
  search: 'Search',
  cog: 'Settings',
  gear: 'Settings',
  user: 'User',
  users: 'Users',
  home: 'Home',
  'file-lines': 'FileText',
  file: 'FileText',
  bell: 'Bell',
  envelope: 'Mail',
  eye: 'Eye',
  'eye-slash': 'EyeOff',
  lock: 'Lock',
  unlock: 'Unlock',
  key: 'Key',
  globe: 'Globe',
  server: 'Server',
  database: 'Database',
  code: 'Code',
  terminal: 'Terminal',
  'chevron-down': 'ChevronDown',
  'chevron-up': 'ChevronUp',
  'chevron-left': 'ChevronLeft',
  'chevron-right': 'ChevronRight',
  'arrow-right': 'ArrowRight',
  'arrow-left': 'ArrowLeft',
  'external-link': 'ExternalLink',
  download: 'Download',
  upload: 'Upload',
  refresh: 'RefreshCw',
  sync: 'RefreshCw',
  warning: 'AlertTriangle',
  'exclamation-triangle': 'AlertTriangle',
  info: 'Info',
  'info-circle': 'Info',
  'question-circle': 'HelpCircle',
  clock: 'Clock',
  calendar: 'Calendar',
  filter: 'Filter',
  sort: 'ArrowUpDown',
  play: 'Play',
  pause: 'Pause',
  stop: 'Square',
  power: 'Power',
  'sign-out': 'LogOut',
  'sign-in': 'LogIn',
  shield: 'Shield',
  'shield-check': 'ShieldCheck',
  building: 'Building2',
  box: 'Package',
  tag: 'Tag',
  tags: 'Tags',
  layers: 'Layers',
  zap: 'Zap',
  activity: 'Activity',
  'bar-chart': 'BarChart3',
  webhook: 'Webhook',
  send: 'Send',
  inbox: 'Inbox',
  list: 'List',
  grid: 'LayoutGrid',
  menu: 'Menu',
  'more-horizontal': 'MoreHorizontal',
  'more-vertical': 'MoreVertical',
  clipboard: 'Clipboard',
  'clipboard-check': 'ClipboardCheck',
  'toggle-on': 'ToggleRight',
  'toggle-off': 'ToggleLeft',
  'arrows-rotate': 'RefreshCw',
  pen: 'Pencil',
  robot: 'Bot',
  book: 'BookOpen',
};

function toPascalCase(str: string): string {
  return str
    .split('-')
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join('');
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const iconsMap = icons as unknown as Record<string, Component>;

const iconComponent = computed<Component | null>(() => {
  const name = props.name;

  // Check direct mapping first
  const mapped = iconNameMap[name];
  if (mapped && iconsMap[mapped]) {
    return iconsMap[mapped];
  }

  // Try PascalCase conversion
  const pascalName = toPascalCase(name);
  if (iconsMap[pascalName]) {
    return iconsMap[pascalName];
  }

  // Try exact name
  if (iconsMap[name]) {
    return iconsMap[name];
  }

  return null;
});
</script>

<template>
  <component
    :is="iconComponent"
    v-if="iconComponent"
    :size="size"
    :stroke-width="strokeWidth"
    aria-hidden="true"
    v-bind="$attrs"
  />
</template>
