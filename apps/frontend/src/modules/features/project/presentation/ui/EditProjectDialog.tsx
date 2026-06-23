import { Trans, useLingui } from '@lingui/react/macro';
import { FormDialog } from '@shared/presentation/ui';
import {
  type FormChange,
  type FormItem,
  FormItemType,
} from '@shared/presentation/structs/scylla-form.struct.ts';
import { useUpdateProject } from '@/modules/features/project/presentation/hooks/use-update-project.ts';
import type { ProjectEntity } from '@/modules/features/project/domain/entities/project.entity.ts';

interface EditProjectDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
  project: ProjectEntity;
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
    {
      id: 'description',
      label: t`Description`,
      placeholder: project.description || t`Add a description...`,
      type: FormItemType.Input,
      inputType: 'text',
    },
  ];

  const handleSubmit = (values: FormChange[]) => {
    const name = values.find(v => v.id === 'name')?.value;
    const description = values.find(v => v.id === 'description')?.value;

    updateProject.mutate(
      {
        projectId: project.id,
        name: name?.trim() || undefined,
        description: description?.trim() || undefined,
      },
      { onSuccess: () => setOpen(false) },
    );
  };

  return (
    <FormDialog
      open={open}
      onOpenChange={setOpen}
      title={<Trans>Edit project</Trans>}
      description={<Trans>Update the project name and description.</Trans>}
      items={items}
      isPending={updateProject.isPending}
      submitLabel={<Trans>Save</Trans>}
      pendingLabel={<Trans>Saving...</Trans>}
      onSubmit={handleSubmit}
    />
  );
}
