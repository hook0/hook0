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

  const { data: orgs } = useOrganizationList();

  const orgIdForApps = computed(() => currentOrgId.value || '');
  const { data: apps } = useApplicationList(orgIdForApps);

  const isAppLevel = computed(() => !!currentOrgId.value && !!currentAppId.value);

  const currentOrgPlan = computed(() => {
    const org = (orgs.value ?? []).find((o) => o.organization_id === currentOrgId.value);
    return org?.plan ?? null;
  });

  /** Navigate to the dashboard of the given organization. */
  function switchOrg(orgId: string) {
    void router.push({
      name: routes.OrganizationsDashboard,
      params: { organization_id: orgId },
    });
  }

  /** Navigate to the dashboard of the given application. */
  function switchApp(orgId: string, appId: string) {
    void router.push({
      name: routes.ApplicationsDashboard,
      params: { organization_id: orgId, application_id: appId },
    });
  }

  /** Navigate to organization settings. Uses provided orgId or falls back to current. */
  function goToOrgSettings(orgId?: string) {
    const id = orgId ?? currentOrgId.value;
    if (id) {
      void router.push({
        name: routes.OrganizationsDetail,
        params: { organization_id: id },
      });
    }
  }

  /** Navigate to application settings. */
  function goToAppSettings(orgId: string, appId: string) {
    void router.push({
      name: routes.ApplicationsDetail,
      params: { organization_id: orgId, application_id: appId },
    });
  }

  /** Navigate to the new organization creation page. */
  function goToCreateOrg() {
    void router.push({ name: routes.OrganizationsNew });
  }

  /** Navigate to the new application creation page for the current org. */
  function goToCreateApp() {
    if (currentOrgId.value) {
      void router.push({
        name: routes.ApplicationsNew,
        params: { organization_id: currentOrgId.value },
      });
    }
  }

  /** Navigate to the current organization's dashboard. */
  function goToOrgDashboard() {
    if (currentOrgId.value) {
      void router.push({
        name: routes.OrganizationsDashboard,
        params: { organization_id: currentOrgId.value },
      });
    }
  }

  return {
    currentOrgId,
    currentOrgName,
    currentAppId,
    currentAppName,
    currentOrgPlan,
    orgs,
    apps,
    isAppLevel,
    switchOrg,
    switchApp,
    goToOrgSettings,
    goToAppSettings,
    goToCreateOrg,
    goToCreateApp,
    goToOrgDashboard,
  };
}
