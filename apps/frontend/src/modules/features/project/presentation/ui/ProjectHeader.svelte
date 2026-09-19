<script lang="ts">
  import { can, Permission } from '@platform/authz';
  import { createMutation } from '@platform/query';
  import { FeatureHeader } from '@shared/presentation/ui-svelte';
  import { createFeatureSelection } from '@shared/presentation/state/feature-selection.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { projectMutations } from '../project.queries.ts';
  import AddProjectDialog from './AddProjectDialog.svelte';
  import { projectMessages } from './project.messages.ts';

  interface Props {
    numberOfProjects: number;
    projectIds: string[];
  }

  let { numberOfProjects, projectIds }: Props = $props();

  let open = $state(false);

  const deleteProject = createMutation(() => projectMutations.remove());

  const selection = createFeatureSelection('projects', () => projectIds, {
    deleteItem: id => deleteProject.mutateAsync(id),
  });

  // Creating a project is an organization-level capability; deleting one is
  // checked per project, so this only reflects the current context.
  const canCreate = $derived(can(Permission.CREATE_PROJECT));
  const canDelete = $derived(can(Permission.DELETE_PROJECT));
</script>

<FeatureHeader
  count={numberOfProjects}
  label={t(projectMessages.project)}
  pluralLabel={t(projectMessages.projects)}
  {...selection.headerProps}
  onNew={() => (open = true)}
  newLabel={t(projectMessages.newProject)}
  canNew={canCreate}
  newDeniedReason={t(projectMessages.createDenied)}
  {canDelete}
  deleteDeniedReason={t(projectMessages.deleteDenied)}
/>
<AddProjectDialog {open} setOpen={next => (open = next)} />
