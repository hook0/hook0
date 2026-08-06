import { useRouter } from 'vue-router';
import { routes } from '@/routes';
import * as OrganizationService from '@/pages/organizations/OrganizationService';
import * as ApplicationService from '@/pages/organizations/applications/ApplicationService';

/**
 * Route the user to the right landing page after authentication — whether they
 * just logged in or just verified their email. A brand-new account with no
 * organization (or a single organization with no application yet) lands on the
 * onboarding wizard; anyone else lands on Home. Kept in one place so the login
 * and email-verification flows always agree on where a new user lands.
 */
export function usePostAuthNavigation() {
  const router = useRouter();

  function navigateAfterAuth(): Promise<unknown> {
    return OrganizationService.list().then((organizations) => {
      if (organizations.length === 0) {
        return router.push({ name: routes.Tutorial });
      }
      if (organizations.length === 1) {
        return ApplicationService.list(organizations[0].organization_id).then((applications) => {
          const destination = applications.length === 0 ? routes.Tutorial : routes.Home;
          return router.push({ name: destination });
        });
      }
      return router.push({ name: routes.Home });
    });
  }

  return { navigateAfterAuth };
}
