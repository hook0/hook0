/**
 * RBAC-ready permission composable.
 *
 * For now, always returns true for all permission checks.
 * When backend RBAC ships, this will call an API endpoint
 * to determine actual permissions.
 */
export function usePermissions() {
  return {
    canView: (_resource: string) => true,
    canCreate: (_resource: string) => true,
    canEdit: (_resource: string) => true,
    canDelete: (_resource: string) => true,
  };
}
