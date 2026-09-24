<script lang="ts">
  import { i18n } from '@lingui/core';
  import { can, Permission } from '@platform/authz';
  import { scyllaNavigate } from '@platform/context';
  import { createMutation, createQuery } from '@platform/query';
  import { ErrorState, FeatureHeader } from '@shared/presentation/ui';
  import { createFeatureSelection } from '@shared/presentation/state/feature-selection.svelte.ts';
  import { toast } from '@shared/presentation/utils/toast.ts';
  import { ToastMessages } from '@shared/utils/toast-messages.ts';
  import { ScyllaError } from '@shared/utils/scylla-result.ts';
  import { t } from '@shared/presentation/utils/i18n-svelte.svelte.ts';
  import { userMutations, userQueries } from '../../user.queries.ts';
  import { userMessages } from '../user.messages.ts';
  import AddUserDialog from './AddUserDialog.svelte';
  import UserTable from './user-table/UserTable.svelte';

  const usersQuery = createQuery(() => userQueries.list());
  const users = $derived(usersQuery.data?.items ?? []);

  const deleteUser = createMutation(() => userMutations.remove());

  const selection = createFeatureSelection('users', () => users.map(user => user.userId));

  let openDialog = $state(false);

  // A system resource: no target to check.
  const canCreate = $derived(can(Permission.CREATE_USER));
  const canDelete = $derived(can(Permission.DELETE_USER));

  /** Deleting your own account is refused before any call. Throwing stops `FeatureHeader` from reporting a success. */
  const handleDelete = async () => {
    const currentUserId = localStorage.getItem('userId');
    if (currentUserId && selection.selectedIds.includes(currentUserId)) {
      const errorMessage = i18n._(ToastMessages.USER_DELETE_OWN_ACCOUNT_ERROR);
      toast.error(errorMessage);
      throw new ScyllaError(errorMessage);
    }

    await Promise.all(selection.selectedIds.map(id => deleteUser.mutateAsync(id)));
    selection.clearSelection();
  };
</script>

{#if usersQuery.isLoading}
  <!-- Nothing while loading: the table would only flash. -->
{:else if usersQuery.isError}
  <ErrorState message={t(userMessages.loadError)} />
{:else}
  <div class="flex w-full flex-col gap-4">
    <FeatureHeader
      count={users.length}
      label={t(userMessages.user)}
      pluralLabel={t(userMessages.users)}
      {...selection.headerProps}
      onDeleteSelection={handleDelete}
      onNew={() => (openDialog = true)}
      newLabel={t(userMessages.newUser)}
      canNew={canCreate}
      newDeniedReason={t(userMessages.createDenied)}
      {canDelete}
      deleteDeniedReason={t(userMessages.deleteDenied)}
    />
    <UserTable data={users} onView={scyllaNavigate.goToUserSettings} />
    <AddUserDialog open={openDialog} setOpen={open => (openDialog = open)} />
  </div>
{/if}
