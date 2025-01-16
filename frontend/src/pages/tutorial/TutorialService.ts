import { ref } from 'vue';
import { RouteLocationRaw } from 'vue-router';

export const progressItems = ref([
  { icon: 'sitemap', title: 'Organization' },
  { icon: 'rocket', title: 'Application' },
  { icon: 'folder-tree', title: 'Event Type' },
  { icon: 'link', title: 'Subscription' },
  { icon: 'file-lines', title: 'Event' },
]);

export interface Step {
  title: string;
  details: string;
  isActive: boolean;
  icon?: string;
  route?: RouteLocationRaw;
}
