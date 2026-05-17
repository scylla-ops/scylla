import { AddProjectDialog } from '@/modules/features/project/presentation/ui/AddProjectDialog.tsx';
import { useState } from 'react';
import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { useDeleteProject } from '@/modules/features/project/presentation/hooks/use-delete-project.ts';

interface ProjectHeaderProps {
  numberOfProjects: number;
}

export const ProjectHeader = ({ numberOfProjects }: ProjectHeaderProps) => {
  const [open, setOpen] = useState(false);
  const { selectedIds, clearSelection } = useSelection('projects');
  const deleteProject = useDeleteProject();

  const handleDelete = async () => {
    const promises = selectedIds.map(id => deleteProject.mutateAsync(id));
    await Promise.all(promises);
    clearSelection();
  };

  return (
    <>
      <FeatureHeader
        count={numberOfProjects}
        label='Project'
        selectedCount={selectedIds.length}
        onClearSelection={clearSelection}
        onDeleteSelection={handleDelete}
        onNew={() => setOpen(true)}
        newLabel={<Trans>New project</Trans>}
      />
      <AddProjectDialog open={open} setOpen={setOpen} />
    </>
  );
};
