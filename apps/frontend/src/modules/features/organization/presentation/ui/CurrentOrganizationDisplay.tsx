import { Building2, ChevronsUpDown } from 'lucide-react';
import { useOrganizationStore } from '@/modules/features/organization/presentation/stores/useOrganizationStore.ts';

type OrganizationDisplayButtonProps = {
  description: string;
};

export const CurrentOrganizationDisplay = ({ description }: OrganizationDisplayButtonProps) => {
  const name = useOrganizationStore(state => state.currentOrganizationName);

  return (
    <>
      <div className='bg-sidebar-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg'>
        <Building2 className='size-4' />
      </div>

      <div className='grid flex-1 text-left text-sm leading-tight'>
        <span className='truncate font-medium'>{name}</span>
        <span className='truncate text-xs text-muted-foreground'>{description}</span>
      </div>
      <ChevronsUpDown className='ml-auto' />
    </>
  );
};
