import { AvailableOrganizationItem } from '@/modules/features/organization/presentation/ui/AvailableOrganizationItem.tsx';
import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';
import type { ComponentType, ReactNode } from 'react';
import { useOrganizationStore } from '@/modules/features/organization/presentation/stores/useOrganizationStore.ts';
import { useContextStore } from '@/modules/shared/presentation/stores/useContext.ts';

interface OrganizationListProps {
  Wrapper: ComponentType<{ children: ReactNode; onSelect?: () => void }>;
}

export const OrganizationList = ({ Wrapper }: OrganizationListProps) => {
  const { organizations } = useOrganizations();
  const setOrganiationName = useOrganizationStore(state => state.setCurrentOrganizationName);
  const setOrganizationId = useContextStore(state => state.setOrganizationId);

  if (!organizations) return <p>Loading...</p>;

  return (
    <>
      {organizations.map(organisation => (
        <Wrapper
          key={organisation.organizationId}
          onSelect={() => {
            setOrganizationId(organisation.organizationId);
            setOrganiationName(organisation.name);
          }}
        >
          <AvailableOrganizationItem name={organisation.name} />
        </Wrapper>
      ))}
    </>
  );
};
