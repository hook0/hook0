import { createRouter, createWebHistory } from 'vue-router';
import routes from '@/routes';

const router = createRouter({
  // Provide the history implementation to use
  history: createWebHistory(),
  routes,
});

router.afterEach((to) => {
  const title = (to.meta?.title as string | undefined) ?? null;
  document.title = title ? `${title} — Hook0` : 'Hook0';
});

export default router;
