import { DataTable } from '@shared/presentation/ui';
import { createUserColumns } from '@/modules/features/user/presentation/ui/admin/user-table/UserColumns.tsx';
import type { UserEntity } from '@/modules/features/user/domain/entities/user.entity.ts';
import { useSelection } from '@shared/presentation/hooks/use-selection.ts';
import { useLingui } from '@lingui/react/macro';
import type { SelectionActionsProps } from '@shared/presentation/ui/controls/SelectionActions.tsx';

interface UserTableProps {
  readonly data?: UserEntity[];
  onView: (userId: string) => void;
  /** Owned by the page: deleting users has a rule the table has no business knowing. */
  selection?: SelectionActionsProps;
}

export const UserTable = ({ data, onView, selection }: UserTableProps) => {
  const { selectedIds, select } = useSelection('users');
  const { t } = useLingui();

  return (
    <DataTable
      columns={createUserColumns({ onView })}
      data={data ?? []}
      onRowClick={row => select(row.original.userId)}
      getRowId={(row, index) => row.userId || index.toString()}
      isRowSelected={row => selectedIds.includes(row.userId)}
      searchable
      searchPlaceholder={t`Search a user…`}
      searchValues={user => [user.username, user.userId]}
      selection={selection}
      alignColumnsCenter
      alignRowsCenter
    />
  );
};
