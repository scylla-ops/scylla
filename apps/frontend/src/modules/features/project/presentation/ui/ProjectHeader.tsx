import { Button } from '@shadcn';
import { AddProjectDialog } from '@/modules/features/project/presentation/ui/AddProjectDialog.tsx';
import { useState } from 'react';
import { Trans } from '@lingui/react/macro';

interface ProjectHeaderProps {
  numberOfProjects: number;
}

export const ProjectHeader = ({ numberOfProjects }: ProjectHeaderProps) => {
  const [open, setOpen] = useState(false);

  return (
    <div className={'flex flex-row justify-between items-center w-full'}>
      <div className='flex items-baseline gap-2'>
        <h1 className='text-3xl font-bold tracking-tight'>
          <span className='text-primary'>{numberOfProjects}</span>{' '}
          <span className='text-foreground'>
            Project
            {numberOfProjects > 1 ? 's' : ''}
          </span>
        </h1>
        <span className='text-sm text-muted-foreground font-medium'><Trans>in total</Trans></span>
      </div>
      <Button onClick={() => setOpen(true)} variant={'default'} className={'ml-auto'}>
        <Trans>New project</Trans>
      </Button>
      <AddProjectDialog open={open} setOpen={setOpen} />
    </div>
  );
};
