import { UserTable } from '@/modules/features/user/presentation/ui/admin/user-table/UserTable.tsx';
import { useUsers } from '@/modules/features/user/presentation/hooks/use-users.ts';
import { useDeleteUser } from '@/modules/features/user/presentation/hooks/use-delete-user.ts';
import { FeatureHeader } from '@shared/presentation/ui';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { AddUserDialog } from '@/modules/features/user/presentation/ui/admin/AddUserDialog.tsx';
import { useState } from 'react';
import { Trans } from '@lingui/react/macro';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';
import { useScyllaNavigate } from '@shared/presentation/hooks/use-scylla-navigate.ts';

export const UserAdminPage = () => {
  const { users, isLoading, isError } = useUsers();
  const { selectedIds, clearSelection } = useSelection('users');
  const deleteUser = useDeleteUser();
  const [openDialog, setOpenDialog] = useState(false);

  const { goToUserSettings } = useScyllaNavigate();

  const handleDelete = async () => {
    const promises = selectedIds.map(id => deleteUser.mutateAsync(id));
    await Promise.all(promises);
    clearSelection();
  };

  //todo: handle properly
  if (isLoading) return <></>;
  if (isError) return <ErrorState message='Error loading users' />;

  return (
    <div className={'flex flex-col gap-4 w-full p-2'}>
      <FeatureHeader
        count={users?.users?.length ?? 0}
        label='User'
        selectedCount={selectedIds.length}
        onClearSelection={clearSelection}
        onDeleteSelection={handleDelete}
        onNew={() => setOpenDialog(true)}
        newLabel={<Trans>New user</Trans>}
      />
      <UserTable onView={goToUserSettings} data={users?.users} />
      <AddUserDialog open={openDialog} setOpen={setOpenDialog} />
    </div>
  );
};
