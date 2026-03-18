import { computed } from 'vue';
import { useRouter } from 'vue-router';
import { routes } from '@/routes';
import { useContextStore } from '@/stores/context';
import { useOrganizationList } from '@/pages/organizations/useOrganizationQueries';
import { useApplicationList } from '@/pages/organizations/applications/useApplicationQueries';

export function useOrgAppSwitcher() {
  const router = useRouter();
  const ctx = useContextStore();

  const currentOrgId = computed(() => ctx.organizationId);
  const currentOrgName = computed(() => ctx.organizationName);
  const currentAppId = computed(() => ctx.applicationId);
  const currentAppName = computed(() => ctx.applicationName);

  const { data: orgs, isLoading: orgsLoading } = useOrganizationList();

  const orgIdForApps = computed(() => currentOrgId.value || '');
  const { data: apps, isLoading: appsLoading } = useApplicationList(orgIdForApps);

  const isAppLevel = computed(() => !!currentOrgId.value && !!currentAppId.value);
  const isOrgLevel = computed(() => !!currentOrgId.value && !currentAppId.value);

  function switchOrg(orgId: string) {
    void router.push({
      name: routes.ApplicationsList,
      params: { organization_id: orgId },
    });
  }

  function switchApp(orgId: string, appId: string) {
    void router.push({
      name: routes.ApplicationsDashboard,
      params: { organization_id: orgId, application_id: appId },
    });
  }

  function goToOrgSettings() {
    if (currentOrgId.value) {
      void router.push({
        name: routes.OrganizationsDetail,
        params: { organization_id: currentOrgId.value },
      });
    }
  }

  function goToCreateOrg() {
    void router.push({ name: routes.OrganizationsNew });
  }

  function goToCreateApp() {
    if (currentOrgId.value) {
      void router.push({
        name: routes.ApplicationsNew,
        params: { organization_id: currentOrgId.value },
      });
    }
  }

  function goToOrgDashboard() {
    if (currentOrgId.value) {
      void router.push({
        name: routes.ApplicationsList,
        params: { organization_id: currentOrgId.value },
      });
    }
  }

  return {
    currentOrgId,
    currentOrgName,
    currentAppId,
    currentAppName,
    orgs,
    orgsLoading,
    apps,
    appsLoading,
    isAppLevel,
    isOrgLevel,
    switchOrg,
    switchApp,
    goToOrgSettings,
    goToCreateOrg,
    goToCreateApp,
    goToOrgDashboard,
  };
}
