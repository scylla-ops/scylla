import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';
import type { ComponentType, ReactNode } from 'react';
import { useContextStore } from '@/modules/shared/presentation/stores/useContext.ts';
import { ContextItem } from '@/modules/layout/presentation/ui/context-selector/ContextItem.tsx';
import { Building2 } from 'lucide-react';

interface OrganizationListProps {
  Wrapper: ComponentType<{ children: ReactNode; onSelect?: () => void }>;
}

export const OrganizationList = ({ Wrapper }: OrganizationListProps) => {
  const { organizations } = useOrganizations();
  const setOrganization = useContextStore(state => state.setOrganization);

  if (!organizations) return <p>Loading...</p>;

  return (
    <>
      {organizations.map(organisation => (
        <Wrapper
          key={organisation.organizationId}
          onSelect={() => {
            setOrganization(organisation.organizationId, organisation.name);
          }}
        >
          <ContextItem name={organisation.name} icon={Building2} />
        </Wrapper>
      ))}
    </>
  );
};
