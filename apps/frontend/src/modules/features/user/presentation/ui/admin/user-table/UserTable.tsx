import { DataTable } from '@shared/presentation/ui';
import { createUserColumns } from '@/modules/features/user/presentation/ui/admin/user-table/UserColumns.tsx';
import type { User } from '@/modules/features/user/domain/models/user.model.ts';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';

interface UserTableProps {
  readonly data?: User[];
  onView: (userId: string) => void;
}

export const UserTable = ({ data, onView }: UserTableProps) => {
  const { selectedIds, select } = useSelection('users');

  return (
    <DataTable
      columns={createUserColumns({ onView })}
      data={data ?? []}
      alignCenter
      onRowClick={row => select(row.original.userId)}
      getRowId={(row, index) => row.userId || index.toString()}
      isRowSelected={row => selectedIds.includes(row.userId)}
    />
  );
};
