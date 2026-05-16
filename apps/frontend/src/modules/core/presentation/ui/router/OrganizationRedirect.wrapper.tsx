import { Navigate } from 'react-router-dom';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';
import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';

/**
 * Redirects the user to their current (or first) organization's projects page.
 * Used as the index route inside the authenticated layout.
 */
export const OrganizationRedirectWrapper = () => {
  const storedOrgName = useContextStore(state => state.organization.name);
  const { organizations, isLoading } = useOrganizations();

  if (isLoading) return null;

  const orgName = storedOrgName ?? organizations?.[0]?.name;

  if (orgName) {
    return <Navigate to={`/${slugifyOrgName(orgName)}/projects`} replace />;
  }

  // No orgs at all — Layout will show the onboarding screen
  return null;
};
