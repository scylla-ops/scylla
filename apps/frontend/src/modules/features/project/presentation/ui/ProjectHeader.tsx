import { AddProjectDialog } from '@/modules/features/project/presentation/ui/AddProjectDialog.tsx';
import { useState } from 'react';
import { Trans } from '@lingui/react/macro';
import { FeatureHeader } from '@shared/presentation/ui';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { useDeleteProject } from '@/modules/features/project/presentation/hooks/use-delete-project.ts';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useCan } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';

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

  // Creating a project is an organization-level capability; deleting one is
  // checked per project, so this only reflects the current context.
  const canCreate = useCan(Permission.CREATE_PROJECT);
  const canDelete = useCan(Permission.DELETE_PROJECT);

  return (
    <>
      <FeatureHeader
        count={numberOfProjects}
        label={<Trans>Project</Trans>}
        pluralLabel={<Trans>Projects</Trans>}
        {...headerProps}
        onNew={() => setOpen(true)}
        newLabel={<Trans>New project</Trans>}
        canNew={canCreate}
        newDeniedReason={<Trans>You don't have permission to create projects.</Trans>}
        canDelete={canDelete}
        deleteDeniedReason={<Trans>You don't have permission to delete projects.</Trans>}
      />
      <AddProjectDialog open={open} setOpen={setOpen} />
    </>
  );
};
