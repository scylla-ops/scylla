import { useEffect } from 'react';
import { Outlet, useNavigate, useParams } from 'react-router-dom';
import { useContextStore } from '@platform/context/use-context.store.ts';
import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';
import { slugifyOrgName } from '@shared/utils/slug.ts';

/**
 * Synchronizes the :organizationSlug URL param with the Zustand context store.
 * Resolves the organization by matching the slug against known org names.
 * Falls back to the first organization if the slug doesn't match any.
 */
export const OrganizationSyncWrapper = () => {
  const { organizationSlug } = useParams<{ organizationSlug: string }>();
  const setOrganization = useContextStore(state => state.setOrganization);
  const currentOrgId = useContextStore(state => state.organization.id);
  const { organizations, isLoading } = useOrganizations();
  const navigate = useNavigate();

  useEffect(() => {
    if (!organizationSlug || isLoading || !organizations) return;

    const org = organizations.find(o => slugifyOrgName(o.name) === organizationSlug);

    if (org) {
      if (org.id !== currentOrgId) {
        setOrganization(org.id, org.name);
      }
    } else {
      // Slug doesn't match any org — fallback to first available
      //todo: navigate to a not found page ?
      const fallback = organizations[0];
      if (fallback) {
        void navigate(`/${slugifyOrgName(fallback.name)}/dashboard`, {
          replace: true,
        });
        setOrganization(fallback.id, fallback.name);
      }
    }
  }, [organizationSlug, isLoading, organizations, currentOrgId, setOrganization, navigate]);

  return <Outlet />;
};
