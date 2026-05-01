import { useCreateProject } from '@/modules/features/project/presentation/hooks/useCreateProject.ts';
import { useOrganizations } from '@/modules/features/organization/presentation/hooks/useOrganizations.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { Trans, useLingui } from '@lingui/react/macro';
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
  const { t } = useLingui();
  const navigate = useNavigate();
  const setOrganization = useContextStore(state => state.setOrganization);
  const createProject = useCreateProject();
  const { organizations } = useOrganizations();

  const items: FormItem[] = [
    {
      id: 'organizationId',
      label: t`Organization`,
      placeholder: t`Select an organization`,
      type: FormItemType.Select,
      options: (organizations ?? []).map(org => ({
        label: org.name,
        value: org.organizationId,
      })),
    },
    {
      id: 'name',
      label: t`Project name`,
      placeholder: t`e.g., My project`,
      type: FormItemType.Input,
      inputType: 'text',
    },
  ];

  const handleSubmit = (values: FormChange[]) => {
    const name = values.find(v => v.id === 'name')?.value;
    const organizationId = values.find(v => v.id === 'organizationId')?.value;

    if (!name?.trim() || !organizationId) {
      toast.error(t`Project name is required and you must select an organization.`);
      return;
    }

    const selectedOrganization = organizations?.find(org => org.organizationId === organizationId);

    createProject.mutate(
      { name, organizationId },
      {
        onSuccess: () => {
          setOpen(false);
          if (selectedOrganization) {
            setOrganization(organizationId, selectedOrganization.name);
          }
          navigate('/projects');
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
