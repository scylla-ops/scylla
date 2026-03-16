import { ChevronsUpDown, ProjectorIcon } from 'lucide-react';
import { useProjectStore } from '@/modules/features/project/presentation/stores/useProjectStore.ts';

type CurrentProjectDisplayProps = {
  description: string;
};

export const CurrentOrganizationDisplay = ({ description }: CurrentProjectDisplayProps) => {
  const name = useProjectStore(state => state.currentProjectName);

  return (
    <>
      <div className='bg-primary text-sidebar-primary-foreground flex aspect-square size-8 items-center justify-center rounded-lg'>
        <ProjectorIcon className='size-4' />
      </div>

      <div className='grid flex-1 text-left text-sm leading-tight'>
        <span className='truncate font-medium'>{name}</span>
        <span className='truncate text-xs text-muted-foreground'>{description}</span>
      </div>
      <ChevronsUpDown className='ml-auto' />
    </>
  );
};

export default CurrentOrganizationDisplay;
