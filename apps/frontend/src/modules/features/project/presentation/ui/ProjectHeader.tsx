import { AddProjectDialog } from '@/modules/features/project/presentation/ui/AddProjectDialog.tsx';
import { useState } from 'react';
import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';

interface ProjectHeaderProps {
  numberOfProjects: number;
}

export const ProjectHeader = ({ numberOfProjects }: ProjectHeaderProps) => {
  const [open, setOpen] = useState(false);

  return (
    <>
      <FeatureHeader
        count={numberOfProjects}
        label='Project'
        onNew={() => setOpen(true)}
        newLabel={<Trans>New project</Trans>}
      />
      <AddProjectDialog open={open} setOpen={setOpen} />
    </>
  );
};
