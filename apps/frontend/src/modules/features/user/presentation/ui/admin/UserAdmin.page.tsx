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

export const UserAdminPage = () => {
  const { users, isLoading, isError } = useUsers();
  const userIds = users?.items?.map(user => user.userId) ?? [];
  const { selectedIds, clearSelection, headerProps } = useFeatureSelection('users', userIds);
  const deleteUser = useDeleteUser();
  const [openDialog, setOpenDialog] = useState(false);
  const { i18n } = useLingui();

  const { goToUserSettings } = useScyllaNavigate();

  const handleDelete = async () => {
    const currentUserId = localStorage.getItem('userId');
    if (currentUserId && selectedIds.includes(currentUserId)) {
      toast.error(i18n._(ToastMessages.USER_DELETE_OWN_ACCOUNT_ERROR));
      return;
    }

    const promises = selectedIds.map(id => deleteUser.mutateAsync(id));
    await Promise.all(promises);
    clearSelection();
  };

  //todo: handle properly
  if (isLoading) return <></>;
  if (isError) return <ErrorState message='Error loading users' />;

  return (
    <div className={'flex flex-col gap-4 w-full'}>
      <FeatureHeader
        count={users?.items?.length ?? 0}
        label='User'
        {...headerProps}
        onDeleteSelection={handleDelete}
        onNew={() => setOpenDialog(true)}
        newLabel={<Trans>New user</Trans>}
      />
      <UserTable onView={goToUserSettings} data={users?.items} />
      <AddUserDialog open={openDialog} setOpen={setOpenDialog} />
    </div>
  );
};
