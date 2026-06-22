import { AddProjectDialog } from '@/modules/features/project/presentation/ui/AddProjectDialog.tsx';
import { useState } from 'react';
import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { useDeleteProject } from '@/modules/features/project/presentation/hooks/use-delete-project.ts';

interface ProjectHeaderProps {
  numberOfProjects: number;
  projectIds: string[];
}

export const ProjectHeader = ({ numberOfProjects, projectIds }: ProjectHeaderProps) => {
  const [open, setOpen] = useState(false);
  const deleteProject = useDeleteProject();
  const { headerProps } = useFeatureSelection('projects', projectIds, {
    deleteItem: id => deleteProject.mutateAsync(id),
  });

  return (
    <>
      <FeatureHeader
        count={numberOfProjects}
        label='Project'
        {...headerProps}
        onNew={() => setOpen(true)}
        newLabel={<Trans>New project</Trans>}
      />
      <AddProjectDialog open={open} setOpen={setOpen} />
    </>
  );
};
