import { UserTable } from '@/modules/features/user/presentation/ui/admin/user-table/UserTable.tsx';
import { useUsers } from '@/modules/features/user/presentation/hooks/use-users.ts';
import { useDeleteUser } from '@/modules/features/user/presentation/hooks/use-delete-user.ts';
import { FeatureHeader } from '@shared/presentation/ui';
import { useFeatureSelection } from '@shared/presentation/hooks/use-feature-selection.ts';
import { AddUserDialog } from '@/modules/features/user/presentation/ui/admin/AddUserDialog.tsx';
import { useState } from 'react';
import { Trans, useLingui } from '@lingui/react/macro';
import { ErrorState } from '@shared/presentation/ui/feedback/ErrorState.tsx';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';
import { toast } from '@shared/presentation/utils/toast.ts';
import { ToastMessages } from '@shared/utils/toast-messages.ts';
import { ScyllaError } from '@shared/utils/scylla-result.ts';
import { Permission } from '@/modules/features/permission/domain/structs/permission.struct.ts';
import { useCan } from '@/modules/features/permission/presentation/hooks/use-authorization.ts';

export const UserAdminPage = () => {
  const { users, isLoading, isError } = useUsers();
  const userIds = users?.items?.map(user => user.userId) ?? [];
  const { selectedIds, clearSelection, headerProps } = useFeatureSelection('users', userIds);
  const deleteUser = useDeleteUser();
  const [openDialog, setOpenDialog] = useState(false);
  const { i18n } = useLingui();

  const { goToUserSettings } = useScyllaNavigate();

  // Users are a system-level resource — no org/project target to check against.
  const canCreate = useCan(Permission.CREATE_USER);
  const canDelete = useCan(Permission.DELETE_USER);

  const handleDelete = async () => {
    const currentUserId = localStorage.getItem('userId');
    if (currentUserId && selectedIds.includes(currentUserId)) {
      const errorMessage = i18n._(ToastMessages.USER_DELETE_OWN_ACCOUNT_ERROR);
      toast.error(errorMessage);
      throw new ScyllaError(errorMessage);
    }

    const promises = selectedIds.map(id => deleteUser.mutateAsync(id));
    await Promise.all(promises);
    clearSelection();
  };

  //todo: handle properly
  if (isLoading) return <></>;
  if (isError) return <ErrorState message={<Trans>Error loading users</Trans>} />;

  return (
    <div className={'flex flex-col gap-4 w-full'}>
      <FeatureHeader
        count={users?.items?.length ?? 0}
        label={<Trans>User</Trans>}
        pluralLabel={<Trans>Users</Trans>}
        {...headerProps}
        onDeleteSelection={handleDelete}
        onNew={() => setOpenDialog(true)}
        newLabel={<Trans>New user</Trans>}
        canNew={canCreate}
        newDeniedReason={<Trans>You don't have permission to create users.</Trans>}
        canDelete={canDelete}
        deleteDeniedReason={<Trans>You don't have permission to delete users.</Trans>}
      />
      <UserTable onView={goToUserSettings} data={users?.items} />
      <AddUserDialog open={openDialog} setOpen={setOpenDialog} />
    </div>
  );
};
