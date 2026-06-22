import { useCreateProject } from '@/modules/features/project/presentation/hooks/useCreateProject.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { Trans, useLingui } from '@lingui/react/macro';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { FormDialog } from '@shared/presentation/ui';
import {
  type FormChange,
  type FormItem,
  FormItemType,
} from '@shared/presentation/models/scylla-form.model.ts';
import { useNavigate } from 'react-router-dom';
import { useContextStore } from '@shared/presentation/stores/use-context.store.ts';

interface AddProjectDialogProps {
  open: boolean;
  setOpen: (open: boolean) => void;
}

export function AddProjectDialog({ open, setOpen }: AddProjectDialogProps) {
  const { t, i18n } = useLingui();
  const navigate = useNavigate();
  const organizationId = useContextStore(state => state.organization.id);
  const createProject = useCreateProject();

  const items: FormItem[] = [
    {
      id: 'name',
      label: t`Project name`,
      placeholder: t`e.g., My project`,
      type: FormItemType.Input,
      inputType: 'text',
    },
    {
      id: 'description',
      label: t`Description`,
      placeholder: t`e.g., A short description of the project`,
      type: FormItemType.Input,
      inputType: 'text',
    },
  ];

  const handleSubmit = (values: FormChange[]) => {
    const name = values.find(v => v.id === 'name')?.value;
    const description = values.find(v => v.id === 'description')?.value;

    if (!name?.trim() || !organizationId) {
      toast.error(i18n._(ToastMessages.PROJECT_NAME_REQUIRED_ERROR));
      return;
    }

    createProject.mutate(
      { name, organizationId, description: description?.trim() || undefined },
      {
        onSuccess: () => {
          setOpen(false);
          void navigate('/projects');
        },
      },
    );
  };

  return (
    <FormDialog
      open={open}
      onOpenChange={setOpen}
      title={<Trans>Create a new project</Trans>}
      description={
        <Trans>Enter a name for your new project and select the organization it belongs to.</Trans>
      }
      items={items}
      isPending={createProject.isPending}
      submitLabel={<Trans>Create Project</Trans>}
      onSubmit={handleSubmit}
    />
  );
}
