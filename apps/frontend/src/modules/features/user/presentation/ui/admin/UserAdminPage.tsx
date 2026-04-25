import { UserTable } from '@/modules/features/user/presentation/ui/admin/user-table/UserTable.tsx';
import { useUsers } from '@/modules/features/user/presentation/hooks/use-users.ts';
import { FeatureHeader } from '@shared/presentation/ui';
import { useSelection } from '@shared/presentation/hooks/useSelection.ts';
import { AddUserDialog } from '@/modules/features/user/presentation/ui/admin/AddUserDialog.tsx';
import { useState } from 'react';
import { Trans } from '@lingui/react/macro';
import { ErrorState } from '@shared/presentation/ui/ErrorState.tsx';

export const UserAdminPage = () => {
  const { users, isLoading, isError } = useUsers();
  const { selectedIds, clearSelection } = useSelection('users');
  const [openDialog, setOpenDialog] = useState(false);

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
        onNew={() => setOpenDialog(true)}
        newLabel={<Trans>New user</Trans>}
      />
      <UserTable data={users?.users} />
      <AddUserDialog open={openDialog} setOpen={setOpenDialog} />
    </div>
  );
};
