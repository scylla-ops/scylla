<script lang="ts">
  import { can, Permission } from '@platform/authz';
  import { createMutation } from '@platform/query';
  import { FeatureHeader } from '@shared/presentation/ui-svelte';
  import { createFeatureSelection } from '@shared/presentation/state/feature-selection.svelte.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { secretMutations } from '../../secret.queries.ts';
  import { secretMessages } from '../secret.messages.ts';

  interface Props {
    activeCount: number;
    secretIds: string[];
    projectId: string;
    onAddSecret?: () => void;
  }

  let { activeCount, secretIds, projectId, onAddSecret }: Props = $props();

  const deleteSecret = createMutation(() => secretMutations.remove(projectId));

  // A getter, not the array: the ids are server data that arrives late, and
  // capturing them once would freeze "select all" on the first, empty render.
  const selection = createFeatureSelection('secrets', () => secretIds, {
    deleteItem: id => deleteSecret.mutateAsync(id),
  });

  const canCreate = $derived(can(Permission.CREATE_SECRET, { projectId }));
  const canDelete = $derived(can(Permission.DELETE_SECRET, { projectId }));
</script>

<FeatureHeader
  count={activeCount}
  label={t(secretMessages.secret)}
  pluralLabel={t(secretMessages.secrets)}
  onNew={onAddSecret}
  newLabel={t(secretMessages.newSecret)}
  canNew={canCreate}
  newDeniedReason={t(secretMessages.createDenied)}
  {canDelete}
  deleteDeniedReason={t(secretMessages.deleteDenied)}
  {...selection.headerProps}
/>
