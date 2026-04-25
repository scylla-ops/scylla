import { DataTable } from '@shared/presentation/ui';
import { createUserColumns } from '@/modules/features/user/presentation/ui/admin/user-table/UserColumns.tsx';
import type { User } from '@/modules/features/user/domain/models/user.model.ts';

interface UserTableProps {
  data?: User[];
}

export const UserTable = ({ data }: UserTableProps) => {
  return <DataTable columns={createUserColumns()} data={data ?? []} />;
};
