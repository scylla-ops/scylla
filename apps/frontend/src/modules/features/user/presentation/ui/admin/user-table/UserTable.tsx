import { DataTable } from '@shared/presentation/ui';
import { createUserColumns } from '@/modules/features/user/presentation/ui/admin/user-table/UserColumns.tsx';
import type { User } from '@/modules/features/user/domain/models/user.model.ts';
import { useSelection } from '@shared/presentation/hooks/useSelection.ts';

interface UserTableProps {
  data?: User[];
}

export const UserTable = ({ data }: UserTableProps) => {
  const { selectedIds, select } = useSelection('users');

  return (
    <DataTable
      columns={createUserColumns()}
      data={data ?? []}
      onRowClick={row => select(row.original.userId)}
      getRowId={(row, index) => row.userId || index.toString()}
      isRowSelected={row => selectedIds.includes(row.userId)}
    />
  );
};
