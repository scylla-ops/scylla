import { DataTable } from '@shared/presentation/ui';
import { createUserColumns } from '@/modules/features/users_admin/presentation/user-table/columns.tsx';

export const UserTable = () => {
  return <DataTable columns={createUserColumns()} data={[]} />;
};
