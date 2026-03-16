import { AvailableOrganizationItem } from '@/modules/features/organization/presentation/ui/AvailableOrganizationItem.tsx';
import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';
import type { ComponentType, ReactNode } from 'react';
import { useOrganizationStore } from '@/modules/features/organization/presentation/stores/useOrganizationStore.ts';

interface OrganizationListProps {
  Wrapper: ComponentType<{ children: ReactNode; onSelect?: () => void }>;
}

export const OrganizationList = ({ Wrapper }: OrganizationListProps) => {
  const { organizations } = useOrganizations();
  const setOrganiationName = useOrganizationStore(state => state.setCurrentOrganizationName);

  return (
    <>
      {organizations?.organizations.map(organisation => (
        <Wrapper
          key={organisation.organizationId}
          onSelect={() => setOrganiationName(organisation.name)}
        >
          <AvailableOrganizationItem name={organisation.name} />
        </Wrapper>
      ))}
    </>
  );
};
