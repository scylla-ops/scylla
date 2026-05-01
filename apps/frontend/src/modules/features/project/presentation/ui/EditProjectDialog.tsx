import { Trans, useLingui } from '@lingui/react/macro';
import { FormDialog } from '@shared/presentation/ui';
import { type FormChange, type FormItem, FormItemType } from '@shared/presentation/models/scylla-form.model.ts';
import { useUpdateProject } from '@/modules/features/project/presentation/hooks/use-update-project.ts';
import type { Project } from '@/modules/features/project/domain/models/project.model.ts';

interface EditProjectDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  project: Project;
}

export function EditProjectDialog({ open, setOpen, project }: EditProjectDialogProps) {
  const { t } = useLingui();
  const updateProject = useUpdateProject();

  const items: FormItem[] = [
    {
      id: 'name',
      label: t`Project name`,
      placeholder: project.name,
      type: FormItemType.Input,
      inputType: 'text',
    },
  ];

  const handleSubmit = (values: FormChange[]) => {
    const name = values.find(v => v.id === 'name')?.value;
    if (!name?.trim()) return;

    updateProject.mutate(
      { projectId: project.id, name },
      { onSuccess: () => setOpen(false) },
    );
  };

  return (
    <FormDialog
      open={open}
      onOpenChange={setOpen}
      title={<Trans>Edit project</Trans>}
      description={<Trans>Update the project name.</Trans>}
      items={items}
      isPending={updateProject.isPending}
      submitLabel={<Trans>Save</Trans>}
      pendingLabel={<Trans>Saving...</Trans>}
      onSubmit={handleSubmit}
    />
  );
}

