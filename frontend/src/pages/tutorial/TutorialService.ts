import { ref } from 'vue';

export const progressItems = ref([
  { icon: 'sitemap', description: 'Organization' },
  { icon: 'rocket', description: 'Application' },
  { icon: 'folder-tree', description: 'Event Type' },
  { icon: 'link', description: 'Subscription' },
  { icon: 'file-lines', description: 'Event' },
]);

export interface Step {
  title: string;
  details: string;
  isActive: boolean;
  icon?: string;
  action?: () => void;
}
