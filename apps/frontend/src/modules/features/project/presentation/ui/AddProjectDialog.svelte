<script lang="ts">
  import { i18n } from '@lingui/core';
  import { useContextStore } from '@platform/context';
  import { createMutation } from '@platform/query';
  import {
    FormDialog,
    FormItemType,
    type FormItem,
    type FormValues,
  } from '@shared/presentation/ui-svelte';
  import { toRune } from '@shared/presentation/stores/to-rune.svelte.ts';
  import { toast } from '@shared/presentation/utils/toast.ts';
  import { ToastMessages } from '@shared/utils/toast-messages.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { projectMutations } from '../project.queries.ts';
  import { projectMessages } from './project.messages.ts';

  interface Props {
    open: boolean;
    setOpen: (open: boolean) => void;
  }

  let { open, setOpen }: Props = $props();

  const context = toRune(useContextStore);
  const organizationId = $derived(context().organization.id);

  const createProject = createMutation(() => projectMutations.create());

  const items: readonly FormItem<'name' | 'description'>[] = $derived([
    {
      id: 'name',
      label: t(projectMessages.projectName),
      placeholder: t(projectMessages.projectNamePlaceholder),
      type: FormItemType.Input,
      inputType: 'text',
    },
    {
      id: 'description',
      label: t(projectMessages.description),
      placeholder: t(projectMessages.descriptionPlaceholder),
      type: FormItemType.Input,
      inputType: 'text',
    },
  ]);

  const handleSubmit = ({ name, description }: FormValues<'name' | 'description'>) => {
    // A project belongs to an organization; without one there is nothing to
    // create it under, and the toast says so rather than failing on the wire.
    if (!name.trim() || !organizationId) {
      toast.error(i18n._(ToastMessages.PROJECT_NAME_REQUIRED_ERROR));
      return;
    }

    createProject.mutate(
      { name, organizationId, description: description.trim() || undefined },
      { onSuccess: () => setOpen(false) },
    );
  };
</script>

<FormDialog
  {open}
  onOpenChange={setOpen}
  title={t(projectMessages.createTitle)}
  description={t(projectMessages.createDescription)}
  {items}
  isPending={createProject.isPending}
  submitLabel={t(projectMessages.createSubmit)}
  onSubmit={handleSubmit}
/>
