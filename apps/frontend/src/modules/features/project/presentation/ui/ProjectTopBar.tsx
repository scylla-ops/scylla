import { Button } from '@shadcn';
import { AddProjectDialog } from '@/modules/features/project/presentation/ui/AddProjectDialog.tsx';
import { useState } from 'react';

export const ProjectTopBar = () => {
  const [open, setOpen] = useState(false);

  return (
    <div className={'flex items-center'}>
      <Button onClick={() => setOpen(true)} variant={'default'} className={'ml-auto'}>
        New project
      </Button>
      <AddProjectDialog open={open} setOpen={setOpen} />
    </div>
  );
};
