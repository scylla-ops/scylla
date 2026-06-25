import { DataTable } from '@shared/presentation/ui';
import { createUserColumns } from '@/modules/features/user/presentation/ui/admin/user-table/UserColumns.tsx';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';

interface UserTableProps {
  readonly data?: UserEntity[];
  onView: (userId: string) => void;
}

export const UserTable = ({ data, onView }: UserTableProps) => {
  const { selectedIds, select } = useSelection('users');

  return (
    <DataTable
      columns={createUserColumns({ onView })}
      data={data ?? []}
      onRowClick={row => select(row.original.userId)}
      getRowId={(row, index) => row.userId || index.toString()}
      isRowSelected={row => selectedIds.includes(row.userId)}
      alignColumnsCenter
      alignRowsCenter
    />
  );
};
