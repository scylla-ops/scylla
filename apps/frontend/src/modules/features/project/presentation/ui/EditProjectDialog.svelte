<script lang="ts">
  import { createMutation } from '@platform/query';
  import {
    FormDialog,
    FormItemType,
    type FormItem,
    type FormValues,
  } from '@shared/presentation/ui';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import type { ProjectEntity } from '../../domain/entities/project.entity.ts';
  import { projectMutations } from '../project.queries.ts';
  import { projectMessages } from './project.messages.ts';

  interface Props {
    open: boolean;
    setOpen: (open: boolean) => void;
    project: ProjectEntity;
  }

  let { open, setOpen, project }: Props = $props();

  const updateProject = createMutation(() => projectMutations.update());

  // Prefilled, unlike the organization dialog: a project's current values are
  // the starting point here, so Save is enabled on an untouched form.
  const items: readonly FormItem<'name' | 'description'>[] = $derived([
    {
      id: 'name',
      label: t(projectMessages.projectName),
      placeholder: project.name,
      defaultValue: project.name,
      type: FormItemType.Input,
      inputType: 'text',
    },
    {
      id: 'description',
      label: t(projectMessages.description),
      placeholder: project.description || t(projectMessages.addDescription),
      defaultValue: project.description,
      type: FormItemType.Input,
      inputType: 'text',
    },
  ]);

  const handleSubmit = ({ name, description }: FormValues<'name' | 'description'>) => {
    updateProject.mutate(
      {
        projectId: project.id,
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
  title={t(projectMessages.editTitle)}
  description={t(projectMessages.editDescription)}
  {items}
  isPending={updateProject.isPending}
  submitLabel={t(projectMessages.save)}
  pendingLabel={t(projectMessages.saving)}
  onSubmit={handleSubmit}
/>
