import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';
import type { ComponentType, ReactNode } from 'react';
import { useContextStore } from '@/modules/shared/presentation/stores/useContext.ts';
import { ContextItem } from '@/modules/layout/presentation/ui/context-selector/ContextItem.tsx';
import { Skeleton } from '@/modules/shared/presentation/ui/shadcn/skeleton.tsx';
import { Building2 } from 'lucide-react';
import { useNavigate } from 'react-router-dom';

interface OrganizationListProps {
  Wrapper: ComponentType<{ children: ReactNode; onSelect?: () => void; className?: string }>;
}

export const OrganizationList = ({ Wrapper }: OrganizationListProps) => {
  const { organizations } = useOrganizations();
  const setOrganization = useContextStore(state => state.setOrganization);
  const navigate = useNavigate();

  if (!organizations)
    return (
      <>
        {Array.from({ length: 3 }).map((_, i) => (
          <Wrapper key={i} className='group'>
            <div className='flex items-center gap-3 px-1 py-1'>
              <Skeleton className='h-8 w-8 rounded-md' />
              <Skeleton className='h-4 w-24' />
            </div>
          </Wrapper>
        ))}
      </>
    );

  return (
    <>
      {organizations.map(organisation => (
        <Wrapper
          className={'group hover:bg-slate-50 transition-colors'}
          key={organisation.organizationId}
          onSelect={() => {
            setOrganization(organisation.organizationId, organisation.name);
            navigate('/projects');
          }}
        >
          <ContextItem name={organisation.name} icon={Building2} />
        </Wrapper>
      ))}
    </>
  );
};
