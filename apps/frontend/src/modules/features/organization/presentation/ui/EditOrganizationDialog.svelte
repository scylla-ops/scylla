<script lang="ts">
  import { createMutation } from '@platform/query';
  import {
    FormDialog,
    FormItemType,
    type FormItem,
    type FormValues,
  } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { organizationMutations } from '../organization.queries.ts';
  import { organizationMessages } from './organization.messages.ts';

  interface Props {
    open: boolean;
    setOpen: (open: boolean) => void;
    organization: { id: string; name: string; description?: string };
  }

  let { open, setOpen, organization }: Props = $props();

  const updateOrganization = createMutation(() => organizationMutations.update());

  const items: readonly FormItem<'name' | 'description'>[] = $derived([
    {
      id: 'name',
      label: t(organizationMessages.organizationName),
      placeholder: organization.name,
      type: FormItemType.Input,
      inputType: 'text',
    },
    {
      id: 'description',
      label: t(organizationMessages.description),
      placeholder: organization.description || t(organizationMessages.addDescription),
      type: FormItemType.Input,
      inputType: 'text',
    },
  ]);

  // Both fields are optional on the wire: an empty one means "leave it alone",
  // which is why they are placeholders rather than default values.
  const handleSubmit = ({ name, description }: FormValues<'name' | 'description'>) => {
    updateOrganization.mutate(
      {
        organizationId: organization.id,
        name: name.trim() || undefined,
        description: description.trim() || undefined,
      },
      { onSuccess: () => setOpen(false) },
    );
  };
</script>

<FormDialog
  {open}
  onOpenChange={setOpen}
  title={t(organizationMessages.editTitle)}
  description={t(organizationMessages.editDescription)}
  {items}
  isPending={updateOrganization.isPending}
  submitLabel={t(organizationMessages.save)}
  pendingLabel={t(organizationMessages.saving)}
  onSubmit={handleSubmit}
/>
