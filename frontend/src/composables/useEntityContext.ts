import { computed, watch } from 'vue';
import { useRoute } from 'vue-router';
import { useContextStore } from '@/stores/context';
import { useUiStore } from '@/stores/ui';
import { useOrganizationDetail } from '@/pages/organizations/useOrganizationQueries';
import { useApplicationDetail } from '@/pages/organizations/applications/useApplicationQueries';

/**
 * Composable that syncs entity names (organization, application) to the context store.
 *
 * This composable watches route params for organization_id and application_id,
 * fetches the entity details via TanStack Query (which provides caching),
 * and updates the context store with the entity names.
 *
 * It also tracks recent workspaces (org/app combinations) for quick switching.
 *
 * Initialize once in App.vue or Root.vue to enable entity name display
 * in Header, Breadcrumbs, and other navigation components.
 */
export function useEntityContext() {
  const route = useRoute();
  const contextStore = useContextStore();
  const uiStore = useUiStore();

  const organizationId = computed(() => (route.params.organization_id as string) || '');
  const applicationId = computed(() => (route.params.application_id as string) || '');

  // Use TanStack Query to fetch entity details (leverages caching)
  const { data: organization, isLoading: orgLoading } = useOrganizationDetail(organizationId);
  const { data: application, isLoading: appLoading } = useApplicationDetail(applicationId);

  // Sync organization name to context store
  watch(
    [organization, organizationId],
    ([org, id]) => {
      if (id && org) {
        contextStore.setOrganizationWithName(id, org.name);
      } else if (!id) {
        contextStore.setOrganizationWithName(null, null);
      }
    },
    { immediate: true }
  );

  // Sync application name to context store
  watch(
    [application, applicationId],
    ([app, id]) => {
      if (id && app) {
        contextStore.setApplicationWithName(id, app.name);
      } else if (!id) {
        contextStore.setApplicationWithName(null, null);
      }
    },
    { immediate: true }
  );

  // Track recent workspaces when both org and app names are available
  watch([organization, application, organizationId, applicationId], ([org, app, orgId, appId]) => {
    if (orgId && org) {
      // Add to recent workspaces (org with optional app)
      uiStore.addRecentWorkspace(orgId, org.name, appId || null, app?.name ?? null);
    }
  });

  return {
    organizationId,
    applicationId,
    organizationName: computed(() => organization.value?.name ?? null),
    applicationName: computed(() => application.value?.name ?? null),
    isLoading: computed(() => orgLoading.value || appLoading.value),
  };
}
