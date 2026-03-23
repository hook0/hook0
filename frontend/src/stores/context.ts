import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { UUID } from '@/http';

export const useContextStore = defineStore('context', () => {
  const organizationId = ref<UUID | null>(null);
  const organizationName = ref<string | null>(null);
  const applicationId = ref<UUID | null>(null);
  const applicationName = ref<string | null>(null);

  const hasOrganization = computed(() => organizationId.value !== null);
  const hasApplication = computed(() => applicationId.value !== null);

  function setOrganization(id: UUID | null): void {
    organizationId.value = id;
    if (!id) {
      organizationName.value = null;
      applicationId.value = null;
      applicationName.value = null;
    }
  }

  function setOrganizationWithName(id: UUID | null, name: string | null): void {
    organizationId.value = id;
    organizationName.value = name;
    if (!id) {
      applicationId.value = null;
      applicationName.value = null;
    }
  }

  function setApplication(id: UUID | null): void {
    applicationId.value = id;
    if (!id) {
      applicationName.value = null;
    }
  }

  function setApplicationWithName(id: UUID | null, name: string | null): void {
    applicationId.value = id;
    applicationName.value = name;
  }

  function updateFromRoute(params: Record<string, string | string[]>): void {
    const newOrgId = (params.organization_id as UUID) ?? null;
    const newAppId = (params.application_id as UUID) ?? null;

    // Only clear names if ID changed
    if (newOrgId !== organizationId.value) {
      organizationName.value = null;
      applicationName.value = null;
    }
    if (newAppId !== applicationId.value) {
      applicationName.value = null;
    }

    organizationId.value = newOrgId;
    applicationId.value = newAppId;
  }

  function clear(): void {
    organizationId.value = null;
    organizationName.value = null;
    applicationId.value = null;
    applicationName.value = null;
  }

  return {
    organizationId,
    organizationName,
    applicationId,
    applicationName,
    hasOrganization,
    hasApplication,
    setOrganization,
    setOrganizationWithName,
    setApplication,
    setApplicationWithName,
    updateFromRoute,
    clear,
  };
});
